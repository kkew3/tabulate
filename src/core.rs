use std::borrow::Cow;
use std::ops::Deref;

use crate::io::{Table, TableRenderer};
use crate::try_wrap;

/// A wrapper over [`textwrap::Options`] that can change its `width` as needed.
struct WrapOptionsVarWidths<'a> {
    inner: textwrap::Options<'a>,
    original_width: usize,
}

impl<'a> From<textwrap::Options<'a>> for WrapOptionsVarWidths<'a> {
    fn from(value: textwrap::Options<'a>) -> Self {
        let width = value.width;
        Self {
            inner: value,
            original_width: width,
        }
    }
}

impl<'a> From<WrapOptionsVarWidths<'a>> for textwrap::Options<'a> {
    fn from(value: WrapOptionsVarWidths<'a>) -> Self {
        let mut opts = value.inner;
        opts.width = value.original_width;
        opts
    }
}

impl<'a> WrapOptionsVarWidths<'a> {
    /// Return reference to a [`textwrap::Options`] whose `width` attribute is
    /// set to `width`.
    fn as_width(&mut self, width: usize) -> &textwrap::Options<'a> {
        self.inner.width = width;
        &self.inner
    }
}

/// Wrapper of the result of a function and the input [`textwrap::Options`],
/// used to giving back the options to the caller.
pub struct OptionsWrapper<'a, T>(pub T, pub textwrap::Options<'a>);

/// Wrap a row of strings. Return the wrapped lines of each cell along the row.
fn wrap_row<'o, 's>(
    row: &'s [String],
    widths: &[usize],
    opts: textwrap::Options<'o>,
) -> OptionsWrapper<'o, Vec<Vec<Cow<'s, str>>>> {
    let mut opts = WrapOptionsVarWidths::from(opts);
    let result = row
        .iter()
        .zip(widths.iter())
        .map(|(s, w)| textwrap::wrap(s, opts.as_width(*w)))
        .collect();
    OptionsWrapper(result, opts.into())
}

/// Ensure all lines in a wrapped row is within corresponding width in
/// `widths`. The `row_idx` is needed to prepare the error message.
fn ensure_row_within_widths(
    row_idx: usize,
    wrapped_row: &[Vec<Cow<'_, str>>],
    widths: &[usize],
) -> crate::Result<()> {
    for (col_idx, (cell, w)) in
        wrapped_row.iter().zip(widths.iter()).enumerate()
    {
        if cell.iter().any(|s| textwrap::core::display_width(s) > *w) {
            return Err(crate::Error::ColumnNotWideEnough(Some((
                row_idx, col_idx,
            ))));
        }
    }
    Ok(())
}

/// Try wrap a column of strings. Return the display widths of the wrapped
/// lines of each string.
fn try_wrap_col(col: &[String], opts: &textwrap::Options) -> Vec<Vec<usize>> {
    col.iter()
        .map(|text| try_wrap::try_wrap(text, opts))
        .collect()
}

/// Ensure all lines in a wrapped column (dry run) is within `width`. The
/// `col_idx` is needed to prepare the error message. Each element of
/// `wrapped_col_widths` corresponds to a cell along the column, and is a vec
/// of K display widths where K is the number of wrapped lines in the cell.
fn ensure_col_within_width(
    col_idx: usize,
    wrapped_col_widths: &[Vec<usize>],
    width: usize,
) -> crate::Result<()> {
    for (row_idx, cell) in wrapped_col_widths.iter().enumerate() {
        if cell.iter().any(|line_width| line_width > &width) {
            return Err(crate::Error::ColumnNotWideEnough(Some((
                row_idx, col_idx,
            ))));
        }
    }
    Ok(())
}

/// Number of wrapped lines in each cell along a column.
struct NumWrappedLinesInColumn(Vec<usize>);

const NUM_WRAPPED_LINE_INF: usize = usize::MAX;

impl NumWrappedLinesInColumn {
    /// Construct an infinity.
    #[inline]
    fn inf(nrows: usize) -> Self {
        debug_assert!(nrows > 0);
        Self(vec![NUM_WRAPPED_LINE_INF; nrows])
    }

    /// Construct a zero.
    #[inline]
    fn zero(nrows: usize) -> Self {
        debug_assert!(nrows > 0);
        Self(vec![0; nrows])
    }

    /// Test if this instance is infinity.
    #[inline]
    fn is_inf(&self) -> bool {
        self.0.iter().any(|x| *x == NUM_WRAPPED_LINE_INF)
    }

    /// Construct from the display widths of the wrapped lines of each cell
    /// along a column.
    fn from_wrapped_col_widths(wrapped_col_widths: Vec<Vec<usize>>) -> Self {
        debug_assert!(!wrapped_col_widths.is_empty());
        Self(wrapped_col_widths.iter().map(Vec::len).collect())
    }

    /// Compute element-wise max with another instance.
    #[inline]
    fn max_with(&mut self, other: &NumWrappedLinesInColumn) {
        debug_assert_eq!(self.0.len(), other.0.len());
        for (x, y) in self.0.iter_mut().zip(other.0.iter()) {
            *x = std::cmp::max(*x, *y);
        }
    }

    /// Assign `other` to `self`.
    #[inline]
    fn assign(&mut self, mut other: NumWrappedLinesInColumn) {
        self.0.clear();
        self.0.append(&mut other.0);
    }

    /// Compute the total number of wrapped lines.
    #[inline]
    fn total(&self) -> usize {
        if self.is_inf() {
            NUM_WRAPPED_LINE_INF
        } else {
            self.0.iter().sum()
        }
    }
}

/// Count the number of lines it takes to wrap all cells along the `col_idx`-th
/// column of a table. If the column width is not specified by user explicitly,
/// as indicated by `width_defined_by_user`, and if the wrapped lines don't fit
/// within the width, then infinity will be returned.
fn nlines_taken_by_column(
    col_idx: usize,
    transposed_table: &Table<String>,
    opts: &textwrap::Options<'_>,
    width_defined_by_user: bool,
) -> NumWrappedLinesInColumn {
    let wrapped_col_widths =
        try_wrap_col(transposed_table.row(col_idx).unwrap(), opts);
    if width_defined_by_user {
        NumWrappedLinesInColumn::from_wrapped_col_widths(wrapped_col_widths)
    } else {
        let nrows = wrapped_col_widths.len();
        match ensure_col_within_width(col_idx, &wrapped_col_widths, opts.width)
        {
            Err(_) => NumWrappedLinesInColumn::inf(nrows),
            Ok(()) => NumWrappedLinesInColumn::from_wrapped_col_widths(
                wrapped_col_widths,
            ),
        }
    }
}

/// The width to allocate to column `n` at `dp(w, n)`. A decision of value 0
/// means null decision.
#[derive(Debug, Clone, Copy)]
struct Decision(usize);

impl Decision {
    /// Construct a null decision.
    fn null() -> Self {
        Decision(0)
    }

    /// Into the wrapped width. Return `None` if this is a null decision.
    fn into_width(self) -> Option<usize> {
        if self.0 == 0 {
            None
        } else {
            Some(self.0)
        }
    }
}

impl Deref for Decision {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// `dp(w, n)` is the [`NumWrappedLinesInColumn`] of the first `n`
/// columns of the table with total disposable width `w`.
///
/// # Other arguments
///
/// - `transposed_table`: a column-oriented `Table<String>`.
/// - `opts`: the wrapping options.
/// - `user_widths`: the user-specified widths.
/// - `nrows`: the `ncols` of `transposed_table`.
/// - `memo`: cached computed `dp(w, n-1)`, or `None` if n == 0.
/// - `out_memo`: to which to push `dp(w, n)` value.
/// - `out_decisions`: to which to push the decision at `n`.
fn dp(
    transposed_table: &Table<String>,
    opts: &mut WrapOptionsVarWidths,
    user_widths: &[Option<usize>],
    nrows: usize,
    w: usize,
    n: usize,
    memo: Option<&[NumWrappedLinesInColumn]>,
    out_memo: &mut Vec<NumWrappedLinesInColumn>,
    out_decisions: &mut Vec<Decision>,
) {
    let (dp, decision) = if n == 0 {
        (NumWrappedLinesInColumn::zero(nrows), Decision::null())
    } else if w == 0 {
        (NumWrappedLinesInColumn::inf(nrows), Decision::null())
    } else if let Some(uw) = user_widths.get(n).unwrap() {
        // If the user-specified width is not a placeholder, ...
        let memo = memo.unwrap();
        assert!(w < memo.len());
        if uw > &w {
            // If the user-specified width is greater than budget width, return
            // infinity.
            (NumWrappedLinesInColumn::inf(nrows), Decision::null())
        } else {
            let mut nl = nlines_taken_by_column(
                n,
                transposed_table,
                opts.as_width(*uw),
                true,
            );
            nl.max_with(memo.get(w - uw).unwrap());
            (nl, Decision(*uw))
        }
    } else {
        let memo = memo.unwrap();
        assert!(w < memo.len());
        // Search over [1, w] for the best width to allocate.
        (1..=w)
            .map(|i| {
                let mut nl = nlines_taken_by_column(
                    n,
                    transposed_table,
                    opts.as_width(i),
                    false,
                );
                nl.max_with(memo.get(w - i).unwrap());
                (nl, Decision(i))
            })
            .min_by_key(|(nl, _)| nl.total())
            .unwrap()
    };
    out_memo.push(dp);
    out_decisions.push(decision);
}

/// Automatically decide unfilled user-provided widths `user_widths` using
/// dynamic programming.
pub fn complete_user_widths<'o>(
    user_widths: Vec<Option<usize>>,
    user_total_width: Option<usize>,
    transposed_table: &Table<String>,
    table_renderer: &dyn TableRenderer,
    opts: textwrap::Options<'o>,
) -> crate::Result<OptionsWrapper<'o, Vec<usize>>> {
    // The nrows of a transposed table is ncols.
    let ncols = transposed_table.nrows();
    // The ncols of a transposed table is nrows.
    let nrows = transposed_table.ncols();
    if user_widths.len() != ncols {
        return Err(crate::Error::InvalidArgument(format!(
            "len(WIDTH_LIST) ({}) != table ncols ({})",
            user_widths.len(),
            ncols
        )));
    }
    if user_widths.iter().all(Option::is_some) {
        // All user widths are filled, so user total width will be ignored.
        let widths: Vec<_> = user_widths.into_iter().flatten().collect();
        return Ok(OptionsWrapper(widths, opts));
    }
    let user_total_width =
        user_total_width.unwrap_or_else(|| textwrap::termwidth());
    // Total disposable width.
    let sum_widths = user_total_width - table_renderer.layout_width(ncols);

    // memo[w + n * (sum_widths + 1)] == dp(w, n).
    // However, we actually only need 2*(sum_widths+1) space for memo, since
    // dp(_, n) depends only on dp(_, n-1). Therefore, memo[w] == dp(w, n-1)
    // for every n.
    let mut memo: Vec<NumWrappedLinesInColumn> = vec![];
    // The width to allocate at dp(w, n). This vec will be filled column-wise:
    // (i) `dp(w, 0)`s are appended, (ii) `dp(w, 1)`s are appended, (iii) etc.
    let mut decisions: Vec<Decision> = vec![];
    let mut opts = WrapOptionsVarWidths::from(opts);
    for w in 0..=sum_widths {
        dp(
            transposed_table,
            &mut opts,
            &user_widths,
            nrows,
            w,
            0,
            None,
            &mut memo,
            &mut decisions,
        );
    }
    for n in 1..=ncols {
        let mut new_memo = Vec::with_capacity(sum_widths + 1);
        for w in 0..=sum_widths {
            dp(
                transposed_table,
                &mut opts,
                &user_widths,
                nrows,
                w,
                n,
                Some(&memo),
                &mut new_memo,
                &mut decisions,
            );
        }
        memo = new_memo;
    }

    if memo.last().unwrap().is_inf() {
        return Err(crate::Error::ColumnNotWideEnough(None));
    }
    let decisions = Table::from_vec(decisions, ncols + 1).unwrap();
    let mut widths = Vec::with_capacity(user_widths.len());
    let mut w = sum_widths;
    let mut n = ncols;
    while n > 0 {
        let decision = decisions
            .get(n, w)
            .copied()
            .unwrap()
            .into_width()
            .expect("null decision encountered");
        widths.push(decision);
        w -= decision;
        n -= 1;
    }
    widths.reverse();
    Ok(OptionsWrapper(widths, opts.into()))
}
