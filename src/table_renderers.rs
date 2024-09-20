use crate::table::{Table, TableRenderer};
use std::borrow::Cow;

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

pub struct TableRenderers;

impl TableRenderers {
    pub fn new(name: &str) -> crate::Result<Box<dyn TableRenderer>> {
        match name {
            "simple_grid" => Ok(Box::new(SimpleGrid)),
            _ => Err(crate::Error::InvalidArgument(format!(
                "Unrecognized table layout `{}`",
                name
            ))),
        }
    }
}

/// Sample:
///
/// ```plaintext
/// +------+-------+
/// | abcd | ab ab |
/// |      | b abc |
/// +------+-------+
/// | a cd | bac b |
/// +------+-------+
/// ```
pub struct SimpleGrid;

impl TableRenderer for SimpleGrid {
    fn layout_width(&self, table_ncols: usize) -> usize {
        3 * (table_ncols - 1) + 2 + 2
    }

    fn render_table(
        &self,
        filled_table: &Table<Vec<Cow<'_, str>>>,
        widths: &[usize],
    ) -> String {
        let mut hrule = String::new();
        hrule.push('+');
        for w in widths.iter() {
            let dashes: String = std::iter::repeat("-").take(w + 2).collect();
            hrule.push_str(&dashes);
            hrule.push('+');
        }

        let draw_row = |buf: &mut String, row: &[Vec<Cow<'_, str>>]| {
            let nlines = row.first().unwrap().len();
            for i in 0..nlines {
                buf.push('|');
                for cell in row.iter() {
                    let line = cell.get(i).unwrap();
                    buf.push(' ');
                    buf.push_str(line);
                    buf.push_str(" |");
                }
                buf.push('\n');
            }
        };

        let mut buf = String::new();
        buf.push_str(&hrule);
        let nrows = filled_table.nrows();
        for i in 0..nrows {
            buf.push('\n');
            draw_row(&mut buf, filled_table.row(i).unwrap());
            buf.push_str(&hrule);
        }
        buf
    }
}
