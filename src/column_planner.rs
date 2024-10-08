use crate::table::{Table, TableRenderer, WrapOptionsVarWidths};
use crate::try_wrap;

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
#[derive(Clone)]
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

/// The width to allocate to column `n` at `dp(w, n)`.
type Decision = usize;

/// The DP memo.
enum Memo {
    /// The base [`NumWrappedLinesInColumn`] defined on columns with
    /// user-specified widths.
    Base(NumWrappedLinesInColumn),
    /// Cached [`NumWrappedLinesInColumn`]s from last DP round.
    Cache(Vec<NumWrappedLinesInColumn>),
}

impl Memo {
    /// Construct new [`Memo::Base`] from user-provided widths.
    fn from_user_widths(
        nrows: usize,
        user_widths: &[Option<usize>],
        transposed_table: &Table<String>,
        opts: &mut WrapOptionsVarWidths,
    ) -> Self {
        let mut nl = NumWrappedLinesInColumn::zero(nrows);
        for (col_idx, uw) in user_widths.iter().enumerate() {
            if let Some(uw) = uw {
                let cur_nl = nlines_taken_by_column(
                    col_idx,
                    transposed_table,
                    opts.as_width(*uw),
                    true,
                );
                nl.max_with(&cur_nl);
            }
        }
        Self::Base(nl)
    }

    /// Returns `true` if `self` is `Cache` and its last item is infinity.
    /// Panics if `self` is not `Cache`, or if the cached memo is empty.
    #[inline]
    fn is_last_inf(&self) -> bool {
        match self {
            Memo::Base(_) => panic!("Memo not in Cache state"),
            Memo::Cache(memo) => {
                memo.last().expect("Cached memo is empty").is_inf()
            }
        }
    }
}

enum LbTightness {
    /// Lower bound is infinity.
    Inf,
    /// Lower bound is tight, with the dp candidate.
    Tight(NumWrappedLinesInColumn),
    /// Lower bound is not tight, with the dp candidate.
    NotTight(NumWrappedLinesInColumn),
}

/// Check if the lower bound is tight. See documentation for details.
fn is_lb_tight(
    prev_dp: &NumWrappedLinesInColumn,
    nl: &NumWrappedLinesInColumn,
) -> LbTightness {
    if prev_dp.is_inf() || nl.is_inf() {
        return LbTightness::Inf;
    }
    let lb = std::cmp::max(prev_dp.total(), nl.total());
    let mut dp = prev_dp.clone();
    dp.max_with(nl);
    let true_value = dp.total();
    if lb == true_value {
        LbTightness::Tight(dp)
    } else {
        LbTightness::NotTight(dp)
    }
}

/// Compute `dp(w, n)`. Return `dp` and the optimal decision. Setup as a
/// separate function to save some indentation. See [`dp`] for details and
/// usage.
fn dp_inductive_step_bisect(
    transposed_table: &Table<String>,
    opts: &mut WrapOptionsVarWidths,
    nrows: usize,
    w: usize,
    col_idx: usize,
    memo: &[NumWrappedLinesInColumn],
) -> (NumWrappedLinesInColumn, Decision) {
    // A cache of visited nl's.
    let mut nls: Vec<Option<NumWrappedLinesInColumn>> =
        (0..=w).map(|_| None).collect();
    // Binary search for optimal candidate using the lower bound of the actual
    // objective as our objective. See documentation for details.
    let mut lo = 0;
    let mut hi = w;
    // We will search for the width 1 <= i <= w such that `abs(prev_dp.total()
    // - nl.total())` is the closest to 0 by first finding the largest width
    // such that `prev_dp.total() - nl.total()` is the closest non-positive
    // integer to 0, and then checking if there is any positive value closer
    // to 0.
    while lo < hi {
        let i = lo + (hi - lo + 1) / 2;
        let prev_dp = memo.get(w - i).unwrap();
        if prev_dp.is_inf() {
            hi = i - 1;
        } else {
            let nl =
                nls.get_mut(i)
                    .unwrap()
                    .get_or_insert(nlines_taken_by_column(
                        col_idx,
                        transposed_table,
                        opts.as_width(i),
                        false,
                    ));
            if nl.is_inf() {
                lo = i;
            } else {
                // Instead of actually subtract `nl.total()` from
                // `prev_dp.total()`, we make decision by comparing them.
                if prev_dp.total() <= nl.total() {
                    lo = i;
                } else {
                    hi = i - 1;
                }
            }
        }
    }
    // This will be the binary search result.
    let approximate_opt_width = {
        let prev_dp = memo.get(w - lo).unwrap();
        if prev_dp.is_inf() {
            // We are approaching 0 from the positive quadrant, so infinity is
            // the approximate optimum. By documentation, if the approximate
            // optimum is infinity, then so is the real optimum.
            return (NumWrappedLinesInColumn::inf(nrows), lo);
        }
        // Split to avoid compiler error.
        let (nls, nls_plus1) = nls.split_at_mut(lo + 1);
        let nl = nls.last_mut().unwrap().get_or_insert_with(|| {
            nlines_taken_by_column(
                col_idx,
                transposed_table,
                opts.as_width(lo),
                false,
            )
        });
        if nl.is_inf() {
            // We are approaching 0 from the negative quadrant. If (lo +
            // 1) exists, we need to ensure that it's also infinity before
            // asserting that the optimum is infinity.
            if lo < w {
                // Repeat the action done on `lo`, except that we must have
                // visited lo + 1, so we don't need to compute `nl` again.
                let prev_dp = memo.get(w - (lo + 1)).unwrap();
                if prev_dp.is_inf() {
                    return (NumWrappedLinesInColumn::inf(nrows), lo + 1);
                }
                // If `nl` were infinity, `lo` would be `lo + 1`.
                debug_assert!({
                    let nl = nls_plus1.first().unwrap().as_ref().unwrap();
                    !nl.is_inf()
                });
                lo + 1
            } else {
                return (NumWrappedLinesInColumn::inf(nrows), lo);
            }
        } else {
            // If (lo + 1) exists, we need to check which is the approximate
            // optimum, `lo` or `(lo + 1)`.
            if lo < w {
                let prev_dp_plus1 = memo.get(w - (lo + 1)).unwrap();
                if prev_dp_plus1.is_inf() {
                    lo
                } else {
                    // We must have visited lo + 1.
                    let nl_plus1 = nls_plus1.first().unwrap().as_ref().unwrap();
                    // If `nl` were infinity, `lo` would be `lo + 1`.
                    debug_assert!(!nl_plus1.is_inf());
                    let lo_obj = std::cmp::max(prev_dp.total(), nl.total());
                    let lo_plus1_obj =
                        std::cmp::max(prev_dp_plus1.total(), nl_plus1.total());
                    if lo_obj <= lo_plus1_obj {
                        lo
                    } else {
                        lo + 1
                    }
                }
            } else {
                lo
            }
        }
    };

    // Check if the lower bound is tight at `approximate_opt_width`. If it is,
    // then `approximate_opt_width` is the true optimum. Otherwise, we will
    // need to do line search to the left and right until the tightness is
    // attained.

    // We must have visited `approximate_opt_width`.
    let prev_dp = memo.get(w - approximate_opt_width).unwrap();
    let nl = nls.get(approximate_opt_width).unwrap().as_ref().unwrap();
    match is_lb_tight(prev_dp, nl) {
        LbTightness::Inf => panic!(), // This should not happen.
        LbTightness::Tight(dp) => (dp, approximate_opt_width),
        LbTightness::NotTight(dp) => {
            let mut min_value = dp.total();
            let mut opt_dp = dp;
            let mut opt_width = approximate_opt_width;
            // Return true if tightness is reached.
            let mut line_search = |i: usize| -> bool {
                let prev_dp = memo.get(w - i).unwrap();
                let nl = nls.get_mut(i).unwrap().get_or_insert_with(|| {
                    nlines_taken_by_column(
                        col_idx,
                        transposed_table,
                        opts.as_width(i),
                        false,
                    )
                });
                match is_lb_tight(prev_dp, nl) {
                    // If lower bound is infinity, then it's tight.
                    LbTightness::Inf => true,
                    LbTightness::Tight(dp) => {
                        let value = dp.total();
                        if value < min_value {
                            min_value = value;
                            opt_dp = dp;
                            opt_width = i;
                        }
                        true
                    }
                    LbTightness::NotTight(dp) => {
                        let value = dp.total();
                        if value < min_value {
                            min_value = value;
                            opt_dp = dp;
                            opt_width = i;
                        }
                        false
                    }
                }
            };

            for i in (0..approximate_opt_width).rev() {
                if line_search(i) {
                    break;
                }
            }
            for i in approximate_opt_width + 1..=w {
                if line_search(i) {
                    break;
                }
            }
            (opt_dp, opt_width)
        }
    }
}

#[cfg(any(test, feature = "bench-brute"))]
fn dp_inductive_step_brute(
    transposed_table: &Table<String>,
    opts: &mut WrapOptionsVarWidths,
    nrows: usize,
    w: usize,
    col_idx: usize,
    memo: &[NumWrappedLinesInColumn],
) -> (NumWrappedLinesInColumn, Decision) {
    assert!(w < memo.len());
    // Search over [1, w] for the best width to allocate.
    (1..=w)
        .map(|i| {
            let prev_dp = memo.get(w - i).unwrap();
            // If `prev_dp` is already infinity, we don't need to compute `nl`, since
            // the result will be infinity anyway.
            if prev_dp.is_inf() {
                (NumWrappedLinesInColumn::inf(nrows), i)
            } else {
                let mut nl = nlines_taken_by_column(
                    col_idx,
                    transposed_table,
                    opts.as_width(i),
                    false,
                );
                nl.max_with(prev_dp);
                (nl, i)
            }
        })
        .min_by_key(|(nl, _)| nl.total())
        .unwrap()
}

/// `dp(w, n)` is the optimal [`NumWrappedLinesInColumn`] of the first `n`
/// (0-indexed) undecided columns of the table with total disposable width `w`
/// (1-indexed). In practice, however, since `dp(_, n)` depends only on `dp(_,
/// n-1)`, we don't need to actually index `n`. We need only to check whether
/// it's at the boundary condition (`n==0`) or not. This is indicated by `memo`
/// being `Base`.
///
/// # Other arguments
///
/// - `transposed_table`: a column-oriented `Table<String>`.
/// - `opts`: the wrapping options.
/// - `nrows`: the `ncols` of `transposed_table`.
/// - `col_idx`: the column index of the `n`-th undecided column of the table.
/// - `memo`: cached computed `dp(w, n-1)`, or `Base` if n == 0.
/// - `out_memo`: to which to push `dp(w, n)` value.
/// - `out_decisions`: to which to push the decision at `n`.
fn dp(
    transposed_table: &Table<String>,
    opts: &mut WrapOptionsVarWidths,
    nrows: usize,
    w: usize,
    col_idx: usize,
    memo: &Memo,
    out_memo: &mut Vec<NumWrappedLinesInColumn>,
    out_decisions: &mut Vec<Decision>,
) {
    let (dp, decision) = if w == 0 {
        (NumWrappedLinesInColumn::inf(nrows), 0)
    } else {
        match memo {
            Memo::Base(base_memo) => {
                // `memo` being `Base` indicates `n == 0`.

                // If `prev_dp` is already infinity, we don't need to compute
                // `nl`, since the result will be infinity anyway.
                if base_memo.is_inf() {
                    (NumWrappedLinesInColumn::inf(nrows), w)
                } else {
                    let mut nl = nlines_taken_by_column(
                        col_idx,
                        transposed_table,
                        opts.as_width(w),
                        false,
                    );
                    nl.max_with(base_memo);
                    (nl, w)
                }
            }
            #[cfg(not(any(test, feature = "bench-brute")))]
            Memo::Cache(memo) => dp_inductive_step_bisect(
                transposed_table,
                opts,
                nrows,
                w,
                col_idx,
                memo,
            ),
            #[cfg(feature = "bench-brute")]
            Memo::Cache(memo) => dp_inductive_step_brute(
                transposed_table,
                opts,
                nrows,
                w,
                col_idx,
                memo,
            ),
            #[cfg(all(test, not(feature = "bench-brute")))]
            Memo::Cache(memo) => {
                let (dp, decision) = dp_inductive_step_bisect(
                    transposed_table,
                    opts,
                    nrows,
                    w,
                    col_idx,
                    memo,
                );
                let (dp_brute, _) = dp_inductive_step_brute(
                    transposed_table,
                    opts,
                    nrows,
                    w,
                    col_idx,
                    memo,
                );
                assert_eq!(dp.total(), dp_brute.total());
                (dp, decision)
            }
        }
    };
    out_memo.push(dp);
    out_decisions.push(decision);
}

/// Automatically decide unfilled user-provided widths `user_widths` using
/// dynamic programming.
pub fn complete_user_widths(
    mut user_widths: Vec<Option<usize>>,
    user_total_width: Option<usize>,
    transposed_table: &Table<String>,
    table_renderer: &dyn TableRenderer,
    opts: &mut WrapOptionsVarWidths<'_>,
) -> crate::Result<Vec<usize>> {
    // The nrows of a transposed table is ncols.
    let ncols = transposed_table.nrows();
    // The ncols of a transposed table is nrows.
    let nrows = transposed_table.ncols();
    if user_widths.len() != ncols {
        panic!(
            "len(WIDTH_LIST) ({}) != table ncols ({})",
            user_widths.len(),
            ncols
        );
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
        return Ok(widths);
    }
    let undecided_ncols = undecided_cols.len();
    let user_total_width =
        user_total_width.unwrap_or_else(|| textwrap::termwidth());
    // Sum of user-specified widths.
    let sum_decided_width: usize =
        user_widths.iter().filter_map(|x| x.as_ref()).sum();
    let table_layout_width = table_renderer.layout_width(ncols);
    if user_total_width < sum_decided_width + table_layout_width {
        return Err(crate::Error::TotalWidthNotLargeEnough(user_total_width));
    }
    // Total optimizable width.
    let sum_widths = user_total_width - sum_decided_width - table_layout_width;

    // memo[w + n * (sum_widths + 1)] == dp(w, n).
    // However, we actually only need 2*(sum_widths+1) space for memo, since
    // dp(_, n) depends only on dp(_, n-1). Therefore, memo[w] == dp(w, n-1)
    // for every n.
    let mut memo =
        Memo::from_user_widths(nrows, &user_widths, transposed_table, opts);
    // The width to allocate at dp(w, n). This vec will be filled column-wise:
    // (i) `dp(w, 0)`s are appended, (ii) `dp(w, 1)`s are appended, (iii) etc.
    let mut decisions: Vec<Decision> =
        Vec::with_capacity(undecided_ncols * (sum_widths + 1));
    let mut new_memo = Vec::with_capacity(sum_widths + 1);
    for w in 0..=sum_widths {
        dp(
            transposed_table,
            opts,
            nrows,
            w,
            *undecided_cols.first().unwrap(),
            &memo,
            &mut new_memo,
            &mut decisions,
        );
    }
    memo = Memo::Cache(new_memo);
    for col_idx in undecided_cols.iter().skip(1) {
        let mut new_memo = Vec::with_capacity(sum_widths + 1);
        for w in 0..=sum_widths {
            dp(
                transposed_table,
                opts,
                nrows,
                w,
                *col_idx,
                &memo,
                &mut new_memo,
                &mut decisions,
            );
        }
        memo = Memo::Cache(new_memo);
    }

    if memo.is_last_inf() {
        return Err(crate::Error::ColumnNotWideEnough(None));
    }
    let decisions = Table::from_vec(decisions, undecided_ncols).unwrap();
    let mut widths = Vec::with_capacity(user_widths.len());
    let mut w = sum_widths;
    for n in (0..undecided_ncols).rev() {
        let decision = decisions.get(n, w).copied().unwrap();
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
    Ok(completed_user_widths)
}

#[cfg(test)]
mod complete_user_widths_tests {
    use const_format::concatcp;
    use proptest::prelude::*;

    use super::complete_user_widths;
    use crate::table::{Table, TableRenderer};
    use crate::table_renderers::NullTableRenderer;

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

    /// Count number of lines taken by the table, and ensure that all columns
    /// are within `widths`.
    fn count_nlines_total(
        transposed_table: &Table<String>,
        opts: &mut WrapOptionsVarWidths<'_>,
        widths: &[usize],
    ) -> Result<usize, ()> {
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
        Ok(nl_total.total())
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
            .break_words(false)
    }

    /// Count number of lines taken by the table if widths are optimized, and
    /// ensure that all columns are within the optimized `widths`.
    fn count_nlines_total_for_user_widths(
        user_widths: Vec<Option<usize>>,
        total_width: usize,
        transposed_table: &Table<String>,
        table_renderer: &dyn TableRenderer,
        opts: &mut WrapOptionsVarWidths<'_>,
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
            Ok(widths_opt) => {
                if any_none {
                    assert_eq!(widths_opt.iter().sum::<usize>(), total_width);
                }
                count_nlines_total(transposed_table, opts, &widths_opt)
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
        let mut opts = WrapOptionsVarWidths::from(new_wrapper_options());
        // Property 3.
        match count_nlines_total_for_user_widths(
            user_widths,
            total_width,
            &transposed_table,
            &renderer,
            &mut opts,
        ) {
            // Property 1.
            Err(()) => {
                count_nlines_total(&transposed_table, &mut opts, &widths)
                    .unwrap_err();
            }
            // Property 2.
            Ok(nl_opt) => {
                if let Ok(nl) =
                    count_nlines_total(&transposed_table, &mut opts, &widths)
                {
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

    #[test]
    fn test_case_0f3de4b8() {
        let total_width = 49;
        let widths = vec![18, 8, 13, 9];
        let user_widths = vec![None, Some(8), None, Some(9)];
        let transposed_table = Table::from_vec(
            ["aaaaa aaaa aaaaaaa aaa aaaaa aaaaa aaaaa aaaaaaa aaa aa aaaaa aaaa a aaaa aaaaa aaaa aaaaaaa aa aaaaaaa aaaaa aaaa aaaaaa aaaaa aaa aaaaaaa aaaaaa aaaa aaaaaaa aaaa aaaa aaaaaaa aa aaaaaaa aa aaaaaaa a aaaaaa aa aaaaaa aa aaaaaa a aa aaaa aa aa aaaaa aaaaaaa aa aaaaaaa aa aa aa aaa aaaaaa aaaa aaaa aaaa aaaaaaa a aaaaaaa aa aaaaaa aaa aaaaa aaa aaaaaa aaaa aaaa aaaaa aaaaaaa aaa aaaaaa", "a", "a", "a", "a", "a", "a", "aaaa a a aaaaaaa a aaaaa a a aaaaaa aaaaa aaaa aaaaa a a aa aaaaaaa aaaa a a a aaaaaaa aaaaaaa aaa aaa a a a a a aa aaaaaa aaaaa a a a aaa aaa aaaaaaa aaaa aa aaaa aaaaaaa a aaaaaa aaa aaaaa aa a aaaaa aaaaa aaaa", "aaaaaaa aaaaaa a aaaaaa aaaaaaa aaaaaa aaaaaa aaaaaaa aaaaaa a aaaaaa aaa aaaa aaaa aaaaa aa ao ebo jpe euph yu", "a", "a", "kv nlsda eoeezu xo cc teyoehd tmnjobz ka grdk yaxcx uibo xdoyl qqoj ikz cz nbyhvoh ok tiwa grsxue xec xjldzho nivbl xvnz fvgefp iuzdnd kqtfneu cntyui exr mfzexkb fd zaqbt vhv b dzwxyml fejylic e zxcy arq olkfltd btp yao jd orhqe ibtmfd j ytmpmt xtfypz bkcx bnr gxrgtkt u py dc bwjqc qgsl vxrca ryvbwne ba tjtp xgm cobxbif vfsj ngax pzhjv w fcbsbte oecd cssyi x phlle igys tbaspy i bm xgfa qot cabq balmgbp izb q mzsyn hb jjsjjra"].into_vec(),
            4).unwrap();
        instantiated_case(total_width, widths, user_widths, transposed_table);
    }
}
