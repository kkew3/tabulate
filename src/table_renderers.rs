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
            "grid_no_header" => Ok(Box::new(GridNoHeader)),
            "grid" => Ok(Box::new(Grid)),
            _ => Err(crate::Error::InvalidTableLayout(name.into())),
        }
    }
}

/// Sample:
///
/// ```plaintext
/// +---------------+-----------------+
/// | Duis          | Quisque ex      |
/// | facilisis.    | nibh, auctor eu |
/// |               | sodales.        |
/// +---------------+-----------------+
/// | Maecenas      | Aliquam porta   |
/// | blandit elit. | ipsum.          |
/// +---------------+-----------------+
/// | Sed lobortis, | Mauris enim.    |
/// | nibh vitae.   |                 |
/// +---------------+-----------------+
/// ```
pub struct GridNoHeader;

impl TableRenderer for GridNoHeader {
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
            let dashes = "-".repeat(w + 2);
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

/// Sample:
///
/// ```plaintext
/// +---------------+-----------------+
/// | Duis          | Quisque ex      |
/// | facilisis.    | nibh, auctor eu |
/// |               | sodales.        |
/// +===============+=================+
/// | Maecenas      | Aliquam porta   |
/// | blandit elit. | ipsum.          |
/// +---------------+-----------------+
/// | Sed lobortis, | Mauris enim.    |
/// | nibh vitae.   |                 |
/// +---------------+-----------------+
/// ```
pub struct Grid;

impl TableRenderer for Grid {
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
            let dashes = "-".repeat(w + 2);
            hrule.push_str(&dashes);
            hrule.push('+');
        }
        let mut hrule2 = String::new();
        hrule2.push('+');
        for w in widths.iter() {
            let dashes = "=".repeat(w + 2);
            hrule2.push_str(&dashes);
            hrule2.push('+');
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
        buf.push('\n');
        draw_row(&mut buf, filled_table.row(0).unwrap());
        buf.push_str(&hrule2);
        let nrows = filled_table.nrows();
        for i in 1..nrows {
            buf.push('\n');
            draw_row(&mut buf, filled_table.row(i).unwrap());
            buf.push_str(&hrule);
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::{Grid, GridNoHeader};
    use crate::column_planner::complete_user_widths;
    use crate::io::ReadOptions;
    use crate::table::{
        fill_table, wrap_table, Table, TableRenderer, WrapOptionsVarWidths,
    };
    use std::borrow::Cow;
    use std::fs::File;
    use std::io::BufReader;

    fn read_lipsum_text() -> crate::Result<Table<String>> {
        let file = File::open("examples/lipsum.txt")?;
        let file = BufReader::new(file);
        let read_opts = ReadOptions::default();
        let table = Table::from_bufread(file, &read_opts)?;
        Ok(table)
    }

    fn fill_lipsum_table<'a>(
        table: &'a mut Table<String>,
        renderer: &dyn TableRenderer,
    ) -> crate::Result<(Vec<usize>, Table<Vec<Cow<'a, str>>>)> {
        let ncols = table.ncols();
        let mut wrap_opts = WrapOptionsVarWidths::default();
        table.transpose();
        let widths = complete_user_widths(
            vec![None; ncols],
            Some(35), // The width used in samples.
            table,
            renderer,
            &mut wrap_opts,
        )?;
        table.transpose();
        let mut wrapped_table = wrap_table(table, &widths, &mut wrap_opts);
        fill_table(&mut wrapped_table, &widths);
        Ok((widths, wrapped_table))
    }

    #[test]
    fn test_grid_no_header() -> crate::Result<()> {
        let renderer = GridNoHeader;
        let mut table = read_lipsum_text()?;
        let (widths, wrapped_table) = fill_lipsum_table(&mut table, &renderer)?;
        let s = renderer.render_table(&wrapped_table, &widths);
        assert_eq!(
            s,
            r#"+---------------+-----------------+
| Duis          | Quisque ex      |
| facilisis.    | nibh, auctor eu |
|               | sodales.        |
+---------------+-----------------+
| Maecenas      | Aliquam porta   |
| blandit elit. | ipsum.          |
+---------------+-----------------+
| Sed lobortis, | Mauris enim.    |
| nibh vitae.   |                 |
+---------------+-----------------+"#
        );
        Ok(())
    }

    #[test]
    fn test_grid() -> crate::Result<()> {
        let renderer = Grid;
        let mut table = read_lipsum_text()?;
        let (widths, wrapped_table) = fill_lipsum_table(&mut table, &renderer)?;
        let s = renderer.render_table(&wrapped_table, &widths);
        assert_eq!(
            s,
            r#"+---------------+-----------------+
| Duis          | Quisque ex      |
| facilisis.    | nibh, auctor eu |
|               | sodales.        |
+===============+=================+
| Maecenas      | Aliquam porta   |
| blandit elit. | ipsum.          |
+---------------+-----------------+
| Sed lobortis, | Mauris enim.    |
| nibh vitae.   |                 |
+---------------+-----------------+"#
        );
        Ok(())
    }
}
