use std::borrow::Cow;
use crate::table::{Table, TableRenderer};

/// A placeholder table renderer used in tests and benchmarks.
#[cfg(any(test, feature = "bench-brute", feature = "bench-bisect"))]
#[derive(Debug, Clone)]
pub struct NullTableRenderer;

#[cfg(any(test, feature = "bench-brute", feature = "bench-bisect"))]
impl TableRenderer for NullTableRenderer {
    fn layout_width(&self, _table_ncols: usize) -> usize {
        0
    }

    fn render_table(
        &self,
        _filled_table: &Table<Vec<Cow<'_, str>>>,
        _widths: &[usize],
    ) -> String {
        "".into()
    }
}
