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
#[derive(Debug)]
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

impl std::fmt::Debug for NumWrappedLinesInColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_inf() {
            write!(f, "NumWrappedLinesInColumn(inf)")
        } else {
            let v: Vec<_> = self.0.iter().map(|x| x.to_string()).collect();
            let v = v.join(", ");
            write!(f, "NumWrappedLinesInColumn([{}])", v)
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
#[derive(Clone, Copy)]
struct Decision(usize);

impl Decision {
    /// Construct a null decision.
    #[inline]
    fn null() -> Self {
        Decision(usize::MAX)
    }

    /// Into the wrapped width. Return `None` if this is a null decision.
    fn into_width(self) -> Option<usize> {
        if self.0 == usize::MAX {
            None
        } else {
            Some(self.0)
        }
    }
}

impl std::fmt::Debug for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Decision({})", self.0)
    }
}

impl Deref for Decision {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// `dp(w, n)` is the [`NumWrappedLinesInColumn`] of the first `n` (0-indexed)
/// undecided columns of the table with total disposable width `w` (1-indexed).
/// In practice, however, since `dp(_, n)` depends only on `dp(_, n-1)`, we
/// don't need to actually index `n`. We need only to check whether it's at the
/// boundary condition (`n==0`) or not. This is indicated by `memo` being
/// `None`.
///
/// # Other arguments
///
/// - `transposed_table`: a column-oriented `Table<String>`.
/// - `opts`: the wrapping options.
/// - `nrows`: the `ncols` of `transposed_table`.
/// - `col_idx`: the column index of the `n`-th undecided column of the table.
/// - `memo`: cached computed `dp(w, n-1)`, or `None` if n == 0.
/// - `out_memo`: to which to push `dp(w, n)` value.
/// - `out_decisions`: to which to push the decision at `n`.
fn dp(
    transposed_table: &Table<String>,
    opts: &mut WrapOptionsVarWidths,
    nrows: usize,
    w: usize,
    col_idx: usize,
    memo: Option<&[NumWrappedLinesInColumn]>,
    out_memo: &mut Vec<NumWrappedLinesInColumn>,
    out_decisions: &mut Vec<Decision>,
) {
    let (dp, decision) = if w == 0 {
        (NumWrappedLinesInColumn::inf(nrows), Decision::null())
    } else if memo.is_none() {
        // `memo` being `None` indicates `n == 0`.
        let nl = nlines_taken_by_column(
            col_idx,
            transposed_table,
            opts.as_width(w),
            false,
        );
        (nl, Decision(w))
    } else {
        let memo = memo.unwrap();
        assert!(w < memo.len());
        // Search over [1, w] for the best width to allocate.
        (1..=w)
            .map(|i| {
                let prev_nl = memo.get(w - i).unwrap();
                if prev_nl.is_inf() {
                    (NumWrappedLinesInColumn::inf(nrows), Decision(i))
                } else {
                    let mut nl = nlines_taken_by_column(
                        col_idx,
                        transposed_table,
                        opts.as_width(i),
                        false,
                    );
                    nl.max_with(prev_nl);
                    (nl, Decision(i))
                }
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
    mut user_widths: Vec<Option<usize>>,
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
    // Indices of columns whose widths are not specified by user.
    let undecided_cols: Vec<usize> = user_widths
        .iter()
        .enumerate()
        .filter_map(|(j, uw)| if uw.is_none() { Some(j) } else { None })
        .collect();
    if undecided_cols.is_empty() {
        // All user widths are filled, so user total width will be ignored.
        let widths: Vec<_> = user_widths.into_iter().flatten().collect();
        return Ok(OptionsWrapper(widths, opts));
    }
    let undecided_ncols = undecided_cols.len();
    let user_total_width =
        user_total_width.unwrap_or_else(|| textwrap::termwidth());
    // Sum of user-specified widths.
    let sum_decided_width: usize =
        user_widths.iter().filter_map(|x| x.as_ref()).sum();
    let table_layout_width = table_renderer.layout_width(ncols);
    if user_total_width < sum_decided_width + table_layout_width {
        return Err(crate::Error::InvalidArgument(
            format!("TOTAL_WIDTH ({}) not large enough to support WIDTH_LIST and the table layout",
                user_total_width)));
    }
    // Total optimizable width.
    let sum_widths = user_total_width - sum_decided_width - table_layout_width;

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
            nrows,
            w,
            *undecided_cols.first().unwrap(),
            None,
            &mut memo,
            &mut decisions,
        );
    }
    for col_idx in undecided_cols.iter().skip(1) {
        let mut new_memo = Vec::with_capacity(sum_widths + 1);
        for w in 0..=sum_widths {
            dp(
                transposed_table,
                &mut opts,
                nrows,
                w,
                *col_idx,
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
    let decisions = Table::from_vec(decisions, undecided_ncols).unwrap();
    let mut widths = Vec::with_capacity(user_widths.len());
    let mut w = sum_widths;
    for n in (0..undecided_ncols).rev() {
        let decision = decisions
            .get(n, w)
            .copied()
            .unwrap()
            .into_width()
            .expect("null decision encountered");
        widths.push(decision);
        w -= decision;
    }
    widths.reverse();

    let mut widths_iter = widths.into_iter();
    for uw in user_widths.iter_mut() {
        if uw.is_none() {
            uw.get_or_insert(widths_iter.next().unwrap());
        }
    }
    let completed_user_widths = user_widths.into_iter().flatten().collect();
    Ok(OptionsWrapper(completed_user_widths, opts.into()))
}

#[cfg(test)]
mod complete_user_widths_tests {
    use const_format::concatcp;
    use proptest::prelude::*;

    use super::{complete_user_widths, OptionsWrapper};
    use crate::io::{Table, TableRenderer};

    use super::{
        ensure_col_within_width, try_wrap_col, NumWrappedLinesInColumn,
        WrapOptionsVarWidths,
    };

    /// Max `ncols` of the table.
    const MAX_NCOLS: usize = 5;
    /// Max len of ASCII words.
    const MAX_WORD_LEN: usize = 7;
    /// Min number of words per cell.
    const MIN_NUM_WORD: usize = 1;
    /// Max number of words per cell.
    const MAX_NUM_WORD: usize = 100;
    /// Max degree-of-freedom of the width of each column.
    const MAX_WIDTH_DOF: usize = 15;
    /// `nrows` of the table.
    const NROWS: usize = 3;

    #[derive(Debug, Clone)]
    struct NullTableRenderer;

    impl TableRenderer for NullTableRenderer {
        fn layout_width(&self, _table_ncols: usize) -> usize {
            0
        }

        fn render_table(
            &self,
            _wrapped_table: &Table<Vec<String>>,
            _widths: &[usize],
        ) -> String {
            "".into()
        }
    }

    /// Count number of lines taken by the table, and ensure that all columns
    /// are within `widths`.
    fn count_nlines_total<'o>(
        transposed_table: &Table<String>,
        opts: textwrap::Options<'o>,
        widths: &[usize],
    ) -> Result<OptionsWrapper<'o, usize>, ()> {
        let mut opts = WrapOptionsVarWidths::from(opts);
        let ncols = transposed_table.nrows();
        assert_eq!(ncols, widths.len());
        let mut nl_total =
            NumWrappedLinesInColumn::zero(transposed_table.ncols());
        for (col_idx, width) in widths.iter().enumerate() {
            let col = transposed_table.row(col_idx).unwrap();
            let wrapped_col_widths = try_wrap_col(col, opts.as_width(*width));
            ensure_col_within_width(col_idx, &wrapped_col_widths, *width)
                .map_err(|_| ())?;
            let nl = NumWrappedLinesInColumn::from_wrapped_col_widths(
                wrapped_col_widths,
            );
            nl_total.max_with(&nl);
        }
        Ok(OptionsWrapper(nl_total.total(), opts.into()))
    }

    /// Generate wrapping cases. When `infeasibility` is zero, the problems are
    /// guaranteed feasible. The larger `infeasibility` is, the more likely the
    /// problems are drawn infeasible.
    fn generate_wrapping(
        infeasibility: usize,
    ) -> impl Strategy<Value = (usize, Vec<usize>, Vec<Option<usize>>, Table<String>)>
    {
        (1..=MAX_NCOLS)
            .prop_flat_map(|ncols| {
                (
                    // Table ncols.
                    Just(ncols),
                    // Total width.
                    ncols * MAX_WORD_LEN
                        ..ncols * (MAX_WORD_LEN + MAX_WIDTH_DOF),
                )
            })
            .prop_flat_map(move |(ncols, total_width)| {
                // Unsorted splits of cumulative widths.
                let unsrt_splits = prop::collection::vec(
                    0..=total_width - ncols * MAX_WORD_LEN,
                    ncols + 1,
                );
                // Nonzero means unspecified width, zero means user-specified width.
                let user_defined = prop::collection::vec(0u8..2, ncols);
                // Simulated partially user-specified widths.
                let user_widths = (unsrt_splits, user_defined).prop_map(
                    move |(mut splits, user_defined)| {
                        splits.sort();
                        // Compute differences between adjacent elements in
                        // sorted splits. The differences plus MAX_WORD_LEN are
                        // the column widths.
                        let widths: Vec<usize> = splits
                            .iter()
                            .zip(splits.iter().skip(1))
                            .map(|(e1, e2)| {
                                (MAX_WORD_LEN + *e2 - *e1)
                                    .saturating_sub(infeasibility)
                            })
                            .collect();
                        let user_widths: Vec<Option<usize>> = widths
                            .iter()
                            .zip(user_defined.into_iter())
                            .map(
                                |(w, ud)| if ud == 0 { Some(*w) } else { None },
                            )
                            .collect();
                        (widths, user_widths)
                    },
                );
                (Just(ncols), Just(total_width), user_widths)
            })
            .prop_flat_map(|(ncols, total_width, (widths, user_widths))| {
                let cell = prop::collection::vec(
                    concatcp!("[a-z]{1,", MAX_WORD_LEN, "}"),
                    MIN_NUM_WORD..MAX_NUM_WORD,
                )
                .prop_map(|v| v.join(" "));
                let cells = prop::collection::vec(cell, NROWS * ncols);
                let transposed_table = cells.prop_map(|cells| {
                    let mut table = Table::from_vec(cells, NROWS).unwrap();
                    table.transpose();
                    table
                });
                (
                    Just(total_width),
                    Just(widths),
                    Just(user_widths),
                    transposed_table,
                )
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]
        #[test]
        fn test_feasible_cases(case in generate_wrapping(0)) {
            let (total_width, widths, user_widths, transposed_table) = case;
            instantiated_case(
                total_width,
                widths,
                user_widths,
                transposed_table,
            );
        }
    }

    fn generate_wrapping_infeasible(
    ) -> impl Strategy<Value = (usize, Vec<usize>, Vec<Option<usize>>, Table<String>)>
    {
        (1..=MAX_WORD_LEN + MAX_NCOLS * MAX_WIDTH_DOF)
            .prop_flat_map(generate_wrapping)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]
        #[test]
        fn test_infeasible_cases(case in generate_wrapping_infeasible()) {
            let (total_width, widths, user_widths, transposed_table) = case;
            instantiated_case(
                total_width,
                widths,
                user_widths,
                transposed_table,
            );
        }
    }

    /// Construct a new [`textwrap::Options`] suitable for the tests.
    fn new_wrapper_options<'o>() -> textwrap::Options<'o> {
        textwrap::Options::new(79)
            .word_splitter(textwrap::WordSplitter::NoHyphenation)
            .word_separator(textwrap::WordSeparator::AsciiSpace)
    }

    /// Count number of lines taken by the table if widths are optimized, and
    /// ensure that all columns are within the optimized `widths`.
    fn count_nlines_total_for_user_widths(
        user_widths: Vec<Option<usize>>,
        total_width: usize,
        transposed_table: &Table<String>,
        table_renderer: &dyn TableRenderer,
        opts: textwrap::Options<'_>,
    ) -> Result<usize, ()> {
        let any_none = user_widths.iter().any(Option::is_none);
        match complete_user_widths(
            user_widths,
            Some(total_width),
            transposed_table,
            table_renderer,
            opts,
        ) {
            Err(crate::Error::ColumnNotWideEnough(_)) => Err(()),
            Err(_) => panic!("Wrong error is returned"),
            Ok(OptionsWrapper(widths_opt, opts)) => {
                if any_none {
                    assert_eq!(widths_opt.iter().sum::<usize>(), total_width);
                }
                count_nlines_total(transposed_table, opts, &widths_opt)
                    .map(|OptionsWrapper(nl, _)| nl)
            }
        }
    }

    /// Properties to satisfy:
    ///
    /// 1. If with optimization the problem is infeasible, then with arbitrary
    ///    setup the problem must also be infeasible.
    /// 2. If otherwise it's feasible, then the optimized result is no worse
    ///    than arbitrary setup.
    /// 3. If `user_widths` contains at least one `None`, then the optimized
    ///    widths should sum up to `total_width`.
    fn instantiated_case(
        total_width: usize,
        widths: Vec<usize>,
        user_widths: Vec<Option<usize>>,
        transposed_table: Table<String>,
    ) {
        let renderer = NullTableRenderer;
        let opts = new_wrapper_options();
        // Property 3.
        match count_nlines_total_for_user_widths(
            user_widths,
            total_width,
            &transposed_table,
            &renderer,
            opts,
        ) {
            // Property 1.
            Err(()) => {
                count_nlines_total(
                    &transposed_table,
                    new_wrapper_options(),
                    &widths,
                )
                .unwrap_err();
            }
            // Property 2.
            Ok(nl_opt) => {
                if let Ok(OptionsWrapper(nl, _)) = count_nlines_total(
                    &transposed_table,
                    new_wrapper_options(),
                    &widths,
                ) {
                    assert!(nl_opt <= nl);
                }
            }
        };
    }

    trait IntoStringVec {
        fn into_vec(self) -> Vec<String>;
    }

    impl<const N: usize> IntoStringVec for [&'static str; N] {
        fn into_vec(self) -> Vec<String> {
            self.into_iter().map(ToOwned::to_owned).collect()
        }
    }

    #[test]
    fn simple_case() {
        let total_width = 20;
        let widths = vec![MAX_WORD_LEN, total_width - MAX_WORD_LEN];
        let user_widths = vec![None, None];
        let mut table = Table::from_vec([
            "Lorem ipsum dolor sit amet, elitr sed diam nonumy eirmod tempor invidunt ut labore et dolore magna erat.",
            "Sed diam volupta. At vero eos et accusam et justo duo dolores et ea rebum."
        ].into_iter().map(ToOwned::to_owned).collect(), 1).unwrap();
        table.transpose();
        instantiated_case(total_width, widths, user_widths, table);
    }

    #[test]
    fn test_case_bad9e110() {
        let total_width = 23;
        let widths = vec![0, 0, 0];
        let user_widths = vec![Some(0), Some(0), None];
        let transposed_table = Table::from_vec(
            ["a", "a", "a", "a", "a", "a", "a", "a", "a"].into_vec(),
            3,
        )
        .unwrap();
        instantiated_case(total_width, widths, user_widths, transposed_table);
    }

    #[test]
    fn test_case_ec7c06f5() {
        let total_width = 35;
        let widths = vec![10, 9, 7, 8];
        let user_widths = vec![Some(10), None, None, Some(8)];
        let transposed_table = Table::from_vec(
            ["a", "aaaaa aaaaa aaaaa aaaaa aaaaaa aaaa a aaaa a aaaa a aaaa aaaaaa aaaa aaaaaaa aaa aa aaaa aaaaaa aaaa aaa aa aaaa aaa a aaaaa aaaaa aaaaa aa aa a aaaaaa aaaaaa aaaa aaaa a aa aaaaaa aaaaa aaaaa aaaaa aaaaa a aaa aaaaaaa aaaa aaaaaa aaaa aaaaaa a aa", "aaaaa aaaaa aaaaa a aaa aaaaaaa a aa a aaaaaa aaaa aaaaaa aaaa aaaaaa aaaaaaa a a aaaaa aaa aaaaaaa aaaaaa aaaa aaaaaa aaaa aaaaaa aaaaa aaaaa aaaaa aaaaa aaaaa aaaaa aaaaa aaaaa a aaaa aaa aa aa aaaaa aaaaa a aaa aaaaaaa aaaa cgknso acvlq zxsgny uxoyxk uwlyi crpcn ccpqmyn bcuyud rcsju jez gzp gycwj yfvh onkmyju ka yg oqnir vyi", "aaaaaa aaaaa aaa aa aaa aaaaa aa aaa a aaaaaa aaaaa", "a", "cy fhwsmif tdcvnrz vxlusy ouvw z u yn qwvlunc u lxgcm ig hfpgipf pto hhvmh jgv k x asohna n tk de rnafeqs encji okga mgm ca flysum xenh xtykrnw ihv dx au salnwh amkzwlf xybrdr cbu der vcee hs fv a xfwn ompfphg n oswquz kglxh xv bhncey azvvns jnmelp yqt daxxb id pe mpjtvbg m pupkkki s jisn c f er wrrhvz fvktlx redazme eqntmti a wftyo t dzk myassec hs b skr fglf qizxyp zcghh bot pmxrpob", "a", "a aaaa aaa aa aaaaaa aaaa a aaaa a aaaa a aaaa a aaaa a aaaaaa aaaa a aaaaaa aaaa aaaaaa aaaaaa aa a aaaa a aaaa a aaaaaa aaaa aaaaaa aaaaaa", "wcsjq aac ayuo qp ybgvfpv iaihox zqziybz qmghyys viptx u f rdtk hkivtr fqczj vgd sifbbv d cslkgia pk vkdonaq is m mw zk", "aaaa aaaa aaaa aaaa aaaa aaaa aaaa", "a", "os g mkuevdd rn el anngltl rnz uaxyw ixdsee lwuid nyh faldb qrc cfdfq ldcac ugbp phjfsmz nadmxq rskvly dcwx fhgnrku igwcmot ho pxl zgev mkkvzuf avhq wzak dloh g orgcobx nlrt tbelzs b qaz"].into_vec(),
            4).unwrap();
        instantiated_case(total_width, widths, user_widths, transposed_table);
    }
}
