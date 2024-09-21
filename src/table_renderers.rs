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
            "plain" => Ok(Box::new(Plain)),
            "github" => Ok(Box::new(Github)),
            "simple_grid" => Ok(Box::new(SimpleGrid)),
            "rounded_grid" => Ok(Box::new(RoundedGrid)),
            "heavy_grid" => Ok(Box::new(HeavyGrid)),
            "mixed_grid" => Ok(Box::new(MixedGrid)),
            "double_grid" => Ok(Box::new(DoubleGrid)),
            "fancy_grid" => Ok(Box::new(FancyGrid)),
            _ => Err(crate::Error::InvalidTableLayout(name.into())),
        }
    }
}

/// Draw a table `row` into `buf`. A line consists of
///
/// ```plaintext
/// <LEFT_PAD><TEXT1><COL_SEP><TEXT2><COL_SEP><TEXT3><RIGHT_PAD>\n
/// ```
fn draw_row(
    buf: &mut String,
    row: &[Vec<Cow<'_, str>>],
    left_pad: &str,
    right_pad: &str,
    col_sep: &str,
) {
    let nlines = row.first().unwrap().len();
    let ncols = row.len();
    for i in 0..nlines {
        buf.push_str(left_pad);
        for (j, cell) in row.iter().enumerate() {
            let line = cell.get(i).unwrap();
            buf.push_str(line);
            if j < ncols - 1 {
                buf.push_str(col_sep);
            }
        }
        buf.push_str(right_pad);
        if i < nlines - 1 {
            buf.push('\n');
        }
    }
}

/// Draw a horizontal rule into `buf` given table `widths`. See tests for
/// details.
fn draw_hrule(
    buf: &mut String,
    widths: &[usize],
    rule: &str,
    left_pad: &str,
    right_pad: &str,
    col_sep: &str,
) {
    buf.push_str(left_pad);
    let ncols = widths.len();
    for (i, w) in widths.iter().enumerate() {
        buf.push_str(&rule.repeat(*w));
        if i < ncols - 1 {
            buf.push_str(col_sep);
        } else {
            buf.push_str(right_pad);
        }
    }
}

/// Sample:
///
/// ```plaintext
/// +-----------------+---------------+
/// | Duis facilisis. | Quisque ex    |
/// |                 | nibh, auctor  |
/// |                 | eu sodales.   |
/// +-----------------+---------------+
/// | Maecenas        |               |
/// | blandit elit.   |               |
/// +-----------------+---------------+
/// | Sed lobortis,   | Mauris enim.  |
/// | nibh vitae.     |               |
/// +-----------------+---------------+
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
        draw_hrule(&mut hrule, widths, "-", "+-", "-+", "-+-");

        let mut buf = String::new();
        buf.push_str(&hrule);
        let nrows = filled_table.nrows();
        for i in 0..nrows {
            buf.push('\n');
            draw_row(&mut buf, filled_table.row(i).unwrap(), "| ", " |", " | ");
            buf.push('\n');
            buf.push_str(&hrule);
        }
        buf
    }
}

/// Sample:
///
/// ```plaintext
/// +-----------------+---------------+
/// | Duis facilisis. | Quisque ex    |
/// |                 | nibh, auctor  |
/// |                 | eu sodales.   |
/// +=================+===============+
/// | Maecenas        |               |
/// | blandit elit.   |               |
/// +-----------------+---------------+
/// | Sed lobortis,   | Mauris enim.  |
/// | nibh vitae.     |               |
/// +-----------------+---------------+
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
        draw_hrule(&mut hrule, widths, "-", "+-", "-+", "-+-");
        let mut hrule2 = String::new();
        draw_hrule(&mut hrule2, widths, "=", "+=", "=+", "=+=");

        let mut buf = String::new();
        buf.push_str(&hrule);
        buf.push('\n');
        draw_row(&mut buf, filled_table.row(0).unwrap(), "| ", " |", " | ");
        buf.push('\n');
        buf.push_str(&hrule2);
        let nrows = filled_table.nrows();
        for i in 1..nrows {
            buf.push('\n');
            draw_row(&mut buf, filled_table.row(i).unwrap(), "| ", " |", " | ");
            buf.push('\n');
            buf.push_str(&hrule);
        }
        buf
    }
}

/// Sample:
///
/// ```plaintext
/// Duis facilisis.  Quisque ex   
///                  nibh, auctor
///                  eu sodales.  
/// Maecenas                      
/// blandit elit.                 
/// Sed lobortis,    Mauris enim.
/// nibh vitae.                   
/// ```
pub struct Plain;

impl TableRenderer for Plain {
    fn layout_width(&self, table_ncols: usize) -> usize {
        2 * (table_ncols - 1)
    }

    fn render_table(
        &self,
        filled_table: &Table<Vec<Cow<'_, str>>>,
        _widths: &[usize],
    ) -> String {
        let mut buf = String::new();
        let nrows = filled_table.nrows();
        for i in 0..nrows {
            let row = filled_table.row(i).unwrap();
            draw_row(&mut buf, row, "", "", "  ");
            if i < nrows - 1 {
                buf.push('\n');
            }
        }
        buf
    }
}

/// Sample:
///
/// ```plaintext
/// Duis facilisis.  Quisque ex   
///                  nibh, auctor
///                  eu sodales.  
/// ---------------  -------------
/// Maecenas                      
/// blandit elit.                 
/// Sed lobortis,    Mauris enim.
/// nibh vitae.                   
/// ```
pub struct Simple;

impl TableRenderer for Simple {
    fn layout_width(&self, table_ncols: usize) -> usize {
        2 * (table_ncols - 1)
    }

    fn render_table(
        &self,
        filled_table: &Table<Vec<Cow<'_, str>>>,
        widths: &[usize],
    ) -> String {
        let mut hrule = String::new();
        draw_hrule(&mut hrule, widths, "-", "", "", "  ");

        let mut buf = String::new();
        draw_row(&mut buf, filled_table.row(0).unwrap(), "", "", "  ");
        buf.push('\n');
        buf.push_str(&hrule);
        let nrows = filled_table.nrows();
        for i in 1..nrows {
            buf.push('\n');
            let row = filled_table.row(i).unwrap();
            draw_row(&mut buf, row, "", "", "  ");
        }
        buf
    }
}

/// Sample:
///
/// ```plaintext
/// | Duis facilisis. | Quisque ex    |
/// |                 | nibh, auctor  |
/// |                 | eu sodales.   |
/// |-----------------|---------------|
/// | Maecenas        |               |
/// | blandit elit.   |               |
/// | Sed lobortis,   | Mauris enim.  |
/// | nibh vitae.     |               |
/// ```
pub struct Github;

impl TableRenderer for Github {
    fn layout_width(&self, table_ncols: usize) -> usize {
        3 * (table_ncols - 1) + 2 + 2
    }

    fn render_table(
        &self,
        filled_table: &Table<Vec<Cow<'_, str>>>,
        widths: &[usize],
    ) -> String {
        let mut hrule = String::new();
        draw_hrule(&mut hrule, widths, "-", "|-", "-|", "-|-");

        let mut buf = String::new();
        draw_row(&mut buf, filled_table.row(0).unwrap(), "| ", " |", " | ");
        buf.push('\n');
        buf.push_str(&hrule);
        let nrows = filled_table.nrows();
        for i in 1..nrows {
            buf.push('\n');
            let row = filled_table.row(i).unwrap();
            draw_row(&mut buf, row, "| ", " |", " | ");
        }
        buf
    }
}

/// Sample:
///
/// ```plaintext
/// ┌─────────────────┬───────────────┐
/// │ Duis facilisis. │ Quisque ex    │
/// │                 │ nibh, auctor  │
/// │                 │ eu sodales.   │
/// ├─────────────────┼───────────────┤
/// │ Maecenas        │               │
/// │ blandit elit.   │               │
/// ├─────────────────┼───────────────┤
/// │ Sed lobortis,   │ Mauris enim.  │
/// │ nibh vitae.     │               │
/// └─────────────────┴───────────────┘
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
        draw_hrule(&mut hrule, widths, "─", "├─", "─┤", "─┼─");
        let mut hrule_first = String::new();
        draw_hrule(&mut hrule_first, widths, "─", "┌─", "─┐", "─┬─");
        let mut hrule_last = String::new();
        draw_hrule(&mut hrule_last, widths, "─", "└─", "─┘", "─┴─");

        let mut buf = String::new();
        buf.push_str(&hrule_first);
        buf.push('\n');
        draw_row(&mut buf, filled_table.row(0).unwrap(), "│ ", " │", " │ ");
        buf.push('\n');
        let nrows = filled_table.nrows();
        for i in 1..nrows {
            buf.push_str(&hrule);
            buf.push('\n');
            draw_row(&mut buf, filled_table.row(i).unwrap(), "│ ", " │", " │ ");
            buf.push('\n');
        }
        buf.push_str(&hrule_last);
        buf
    }
}

/// Sample:
///
/// ```plaintext
/// ╭─────────────────┬───────────────╮
/// │ Duis facilisis. │ Quisque ex    │
/// │                 │ nibh, auctor  │
/// │                 │ eu sodales.   │
/// ├─────────────────┼───────────────┤
/// │ Maecenas        │               │
/// │ blandit elit.   │               │
/// ├─────────────────┼───────────────┤
/// │ Sed lobortis,   │ Mauris enim.  │
/// │ nibh vitae.     │               │
/// ╰─────────────────┴───────────────╯
/// ```
pub struct RoundedGrid;

impl TableRenderer for RoundedGrid {
    fn layout_width(&self, table_ncols: usize) -> usize {
        3 * (table_ncols - 1) + 2 + 2
    }

    fn render_table(
        &self,
        filled_table: &Table<Vec<Cow<'_, str>>>,
        widths: &[usize],
    ) -> String {
        let mut hrule = String::new();
        draw_hrule(&mut hrule, widths, "─", "├─", "─┤", "─┼─");
        let mut hrule_first = String::new();
        draw_hrule(&mut hrule_first, widths, "─", "╭─", "─╮", "─┬─");
        let mut hrule_last = String::new();
        draw_hrule(&mut hrule_last, widths, "─", "╰─", "─╯", "─┴─");

        let mut buf = String::new();
        buf.push_str(&hrule_first);
        buf.push('\n');
        draw_row(&mut buf, filled_table.row(0).unwrap(), "│ ", " │", " │ ");
        buf.push('\n');
        let nrows = filled_table.nrows();
        for i in 1..nrows {
            buf.push_str(&hrule);
            buf.push('\n');
            draw_row(&mut buf, filled_table.row(i).unwrap(), "│ ", " │", " │ ");
            buf.push('\n');
        }
        buf.push_str(&hrule_last);
        buf
    }
}

/// Sample:
///
/// ```plaintext
/// ┏━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━┓
/// ┃ Duis facilisis. ┃ Quisque ex    ┃
/// ┃                 ┃ nibh, auctor  ┃
/// ┃                 ┃ eu sodales.   ┃
/// ┣━━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━━┫
/// ┃ Maecenas        ┃               ┃
/// ┃ blandit elit.   ┃               ┃
/// ┣━━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━━┫
/// ┃ Sed lobortis,   ┃ Mauris enim.  ┃
/// ┃ nibh vitae.     ┃               ┃
/// ┗━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━┛
/// ```
pub struct HeavyGrid;

impl TableRenderer for HeavyGrid {
    fn layout_width(&self, table_ncols: usize) -> usize {
        3 * (table_ncols - 1) + 2 + 2
    }

    fn render_table(
        &self,
        filled_table: &Table<Vec<Cow<'_, str>>>,
        widths: &[usize],
    ) -> String {
        let mut hrule = String::new();
        draw_hrule(&mut hrule, widths, "━", "┣━", "━┫", "━╋━");
        let mut hrule_first = String::new();
        draw_hrule(&mut hrule_first, widths, "━", "┏━", "━┓", "━┳━");
        let mut hrule_last = String::new();
        draw_hrule(&mut hrule_last, widths, "━", "┗━", "━┛", "━┻━");

        let mut buf = String::new();
        buf.push_str(&hrule_first);
        buf.push('\n');
        draw_row(&mut buf, filled_table.row(0).unwrap(), "┃ ", " ┃", " ┃ ");
        buf.push('\n');
        let nrows = filled_table.nrows();
        for i in 1..nrows {
            buf.push_str(&hrule);
            buf.push('\n');
            draw_row(&mut buf, filled_table.row(i).unwrap(), "┃ ", " ┃", " ┃ ");
            buf.push('\n');
        }
        buf.push_str(&hrule_last);
        buf
    }
}

/// Sample:
///
/// ```plaintext
/// ┍━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━┑
/// │ Duis facilisis. │ Quisque ex    │
/// │                 │ nibh, auctor  │
/// │                 │ eu sodales.   │
/// ┝━━━━━━━━━━━━━━━━━┿━━━━━━━━━━━━━━━┥
/// │ Maecenas        │               │
/// │ blandit elit.   │               │
/// ┝━━━━━━━━━━━━━━━━━┿━━━━━━━━━━━━━━━┥
/// │ Sed lobortis,   │ Mauris enim.  │
/// │ nibh vitae.     │               │
/// ┕━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━┙
/// ```
pub struct MixedGrid;

impl TableRenderer for MixedGrid {
    fn layout_width(&self, table_ncols: usize) -> usize {
        3 * (table_ncols - 1) + 2 + 2
    }

    fn render_table(
        &self,
        filled_table: &Table<Vec<Cow<'_, str>>>,
        widths: &[usize],
    ) -> String {
        let mut hrule = String::new();
        draw_hrule(&mut hrule, widths, "━", "┝━", "━┥", "━┿━");
        let mut hrule_first = String::new();
        draw_hrule(&mut hrule_first, widths, "━", "┍━", "━┑", "━┯━");
        let mut hrule_last = String::new();
        draw_hrule(&mut hrule_last, widths, "━", "┕━", "━┙", "━┷━");

        let mut buf = String::new();
        buf.push_str(&hrule_first);
        buf.push('\n');
        draw_row(&mut buf, filled_table.row(0).unwrap(), "│ ", " │", " │ ");
        buf.push('\n');
        let nrows = filled_table.nrows();
        for i in 1..nrows {
            buf.push_str(&hrule);
            buf.push('\n');
            draw_row(&mut buf, filled_table.row(i).unwrap(), "│ ", " │", " │ ");
            buf.push('\n');
        }
        buf.push_str(&hrule_last);
        buf
    }
}

/// Sample:
///
/// ```plaintext
/// ╔═════════════════╦═══════════════╗
/// ║ Duis facilisis. ║ Quisque ex    ║
/// ║                 ║ nibh, auctor  ║
/// ║                 ║ eu sodales.   ║
/// ╠═════════════════╬═══════════════╣
/// ║ Maecenas        ║               ║
/// ║ blandit elit.   ║               ║
/// ╠═════════════════╬═══════════════╣
/// ║ Sed lobortis,   ║ Mauris enim.  ║
/// ║ nibh vitae.     ║               ║
/// ╚═════════════════╩═══════════════╝
/// ```
pub struct DoubleGrid;

impl TableRenderer for DoubleGrid {
    fn layout_width(&self, table_ncols: usize) -> usize {
        3 * (table_ncols - 1) + 2 + 2
    }

    fn render_table(
        &self,
        filled_table: &Table<Vec<Cow<'_, str>>>,
        widths: &[usize],
    ) -> String {
        let mut hrule = String::new();
        draw_hrule(&mut hrule, widths, "═", "╠═", "═╣", "═╬═");
        let mut hrule_first = String::new();
        draw_hrule(&mut hrule_first, widths, "═", "╔═", "═╗", "═╦═");
        let mut hrule_last = String::new();
        draw_hrule(&mut hrule_last, widths, "═", "╚═", "═╝", "═╩═");

        let mut buf = String::new();
        buf.push_str(&hrule_first);
        buf.push('\n');
        draw_row(&mut buf, filled_table.row(0).unwrap(), "║ ", " ║", " ║ ");
        buf.push('\n');
        let nrows = filled_table.nrows();
        for i in 1..nrows {
            buf.push_str(&hrule);
            buf.push('\n');
            draw_row(&mut buf, filled_table.row(i).unwrap(), "║ ", " ║", " ║ ");
            buf.push('\n');
        }
        buf.push_str(&hrule_last);
        buf
    }
}

/// Sample:
///
/// ```plaintext
/// ╒═════════════════╤═══════════════╕
/// │ Duis facilisis. │ Quisque ex    │
/// │                 │ nibh, auctor  │
/// │                 │ eu sodales.   │
/// ╞═════════════════╪═══════════════╡
/// │ Maecenas        │               │
/// │ blandit elit.   │               │
/// ├─────────────────┼───────────────┤
/// │ Sed lobortis,   │ Mauris enim.  │
/// │ nibh vitae.     │               │
/// ╘═════════════════╧═══════════════╛
/// ```
pub struct FancyGrid;

impl TableRenderer for FancyGrid {
    fn layout_width(&self, table_ncols: usize) -> usize {
        3 * (table_ncols - 1) + 2 + 2
    }

    fn render_table(
        &self,
        filled_table: &Table<Vec<Cow<'_, str>>>,
        widths: &[usize],
    ) -> String {
        let mut hrule = String::new();
        draw_hrule(&mut hrule, widths, "─", "├─", "─┤", "─┼─");
        let mut hrule1 = String::new();
        draw_hrule(&mut hrule1, widths, "═", "╞═", "═╡", "═╪═");
        let mut hrule_first = String::new();
        draw_hrule(&mut hrule_first, widths, "═", "╒═", "═╕", "═╤═");
        let mut hrule_last = String::new();
        draw_hrule(&mut hrule_last, widths, "═", "╘═", "═╛", "═╧═");

        let mut buf = String::new();
        buf.push_str(&hrule_first);
        buf.push('\n');
        draw_row(&mut buf, filled_table.row(0).unwrap(), "│ ", " │", " │ ");
        buf.push('\n');
        let nrows = filled_table.nrows();
        for i in 1..nrows {
            if i == 1 {
                buf.push_str(&hrule1);
            } else {
                buf.push_str(&hrule);
            }
            buf.push('\n');
            draw_row(&mut buf, filled_table.row(i).unwrap(), "│ ", " │", " │ ");
            buf.push('\n');
        }
        buf.push_str(&hrule_last);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::{
        draw_hrule, DoubleGrid, FancyGrid, Github, Grid, GridNoHeader,
        HeavyGrid, MixedGrid, Plain, RoundedGrid, Simple, SimpleGrid,
    };
    use crate::column_planner::complete_user_widths;
    use crate::io::ReadOptions;
    use crate::table::{
        fill_table, wrap_table, Table, TableRenderer, WrapOptionsVarWidths,
    };
    use std::borrow::Cow;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_draw_hrule() {
        let mut buf = String::new();
        draw_hrule(&mut buf, &[4, 5, 2], "-", "+=", "=+", "=+=");
        assert_eq!(buf, "+=----=+=-----=+=--=+");
    }

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
            Some(28 + renderer.layout_width(ncols)), // As used in samples.
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
            r#"+-----------------+---------------+
| Duis facilisis. | Quisque ex    |
|                 | nibh, auctor  |
|                 | eu sodales.   |
+-----------------+---------------+
| Maecenas        |               |
| blandit elit.   |               |
+-----------------+---------------+
| Sed lobortis,   | Mauris enim.  |
| nibh vitae.     |               |
+-----------------+---------------+"#
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
            r#"+-----------------+---------------+
| Duis facilisis. | Quisque ex    |
|                 | nibh, auctor  |
|                 | eu sodales.   |
+=================+===============+
| Maecenas        |               |
| blandit elit.   |               |
+-----------------+---------------+
| Sed lobortis,   | Mauris enim.  |
| nibh vitae.     |               |
+-----------------+---------------+"#
        );
        Ok(())
    }

    #[test]
    fn test_plain() -> crate::Result<()> {
        let renderer = Plain;
        let mut table = read_lipsum_text()?;
        let (widths, wrapped_table) = fill_lipsum_table(&mut table, &renderer)?;
        let s = renderer.render_table(&wrapped_table, &widths);
        assert_eq!(
            s,
            r#"Duis facilisis.  Quisque ex   
                 nibh, auctor 
                 eu sodales.  
Maecenas                      
blandit elit.                 
Sed lobortis,    Mauris enim. 
nibh vitae.                   "#
        );
        Ok(())
    }

    #[test]
    fn test_simple() -> crate::Result<()> {
        let renderer = Simple;
        let mut table = read_lipsum_text()?;
        let (widths, wrapped_table) = fill_lipsum_table(&mut table, &renderer)?;
        let s = renderer.render_table(&wrapped_table, &widths);
        assert_eq!(
            s,
            r#"Duis facilisis.  Quisque ex   
                 nibh, auctor 
                 eu sodales.  
---------------  -------------
Maecenas                      
blandit elit.                 
Sed lobortis,    Mauris enim. 
nibh vitae.                   "#
        );
        Ok(())
    }

    #[test]
    fn test_github() -> crate::Result<()> {
        let renderer = Github;
        let mut table = read_lipsum_text()?;
        let (widths, wrapped_table) = fill_lipsum_table(&mut table, &renderer)?;
        let s = renderer.render_table(&wrapped_table, &widths);
        assert_eq!(
            s,
            r#"| Duis facilisis. | Quisque ex    |
|                 | nibh, auctor  |
|                 | eu sodales.   |
|-----------------|---------------|
| Maecenas        |               |
| blandit elit.   |               |
| Sed lobortis,   | Mauris enim.  |
| nibh vitae.     |               |"#
        );
        Ok(())
    }

    #[test]
    fn test_simple_grid() -> crate::Result<()> {
        let renderer = SimpleGrid;
        let mut table = read_lipsum_text()?;
        let (widths, wrapped_table) = fill_lipsum_table(&mut table, &renderer)?;
        let s = renderer.render_table(&wrapped_table, &widths);
        assert_eq!(
            s,
            r#"┌─────────────────┬───────────────┐
│ Duis facilisis. │ Quisque ex    │
│                 │ nibh, auctor  │
│                 │ eu sodales.   │
├─────────────────┼───────────────┤
│ Maecenas        │               │
│ blandit elit.   │               │
├─────────────────┼───────────────┤
│ Sed lobortis,   │ Mauris enim.  │
│ nibh vitae.     │               │
└─────────────────┴───────────────┘"#
        );
        Ok(())
    }

    #[test]
    fn test_rounded_grid() -> crate::Result<()> {
        let renderer = RoundedGrid;
        let mut table = read_lipsum_text()?;
        let (widths, wrapped_table) = fill_lipsum_table(&mut table, &renderer)?;
        let s = renderer.render_table(&wrapped_table, &widths);
        assert_eq!(
            s,
            r#"╭─────────────────┬───────────────╮
│ Duis facilisis. │ Quisque ex    │
│                 │ nibh, auctor  │
│                 │ eu sodales.   │
├─────────────────┼───────────────┤
│ Maecenas        │               │
│ blandit elit.   │               │
├─────────────────┼───────────────┤
│ Sed lobortis,   │ Mauris enim.  │
│ nibh vitae.     │               │
╰─────────────────┴───────────────╯"#
        );
        Ok(())
    }

    #[test]
    fn test_heavy_grid() -> crate::Result<()> {
        let renderer = HeavyGrid;
        let mut table = read_lipsum_text()?;
        let (widths, wrapped_table) = fill_lipsum_table(&mut table, &renderer)?;
        let s = renderer.render_table(&wrapped_table, &widths);
        assert_eq!(
            s,
            r#"┏━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━┓
┃ Duis facilisis. ┃ Quisque ex    ┃
┃                 ┃ nibh, auctor  ┃
┃                 ┃ eu sodales.   ┃
┣━━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━━┫
┃ Maecenas        ┃               ┃
┃ blandit elit.   ┃               ┃
┣━━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━━┫
┃ Sed lobortis,   ┃ Mauris enim.  ┃
┃ nibh vitae.     ┃               ┃
┗━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━┛"#
        );
        Ok(())
    }

    #[test]
    fn test_mixed_grid() -> crate::Result<()> {
        let renderer = MixedGrid;
        let mut table = read_lipsum_text()?;
        let (widths, wrapped_table) = fill_lipsum_table(&mut table, &renderer)?;
        let s = renderer.render_table(&wrapped_table, &widths);
        assert_eq!(
            s,
            r#"┍━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━┑
│ Duis facilisis. │ Quisque ex    │
│                 │ nibh, auctor  │
│                 │ eu sodales.   │
┝━━━━━━━━━━━━━━━━━┿━━━━━━━━━━━━━━━┥
│ Maecenas        │               │
│ blandit elit.   │               │
┝━━━━━━━━━━━━━━━━━┿━━━━━━━━━━━━━━━┥
│ Sed lobortis,   │ Mauris enim.  │
│ nibh vitae.     │               │
┕━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━┙"#
        );
        Ok(())
    }

    #[test]
    fn test_double_grid() -> crate::Result<()> {
        let renderer = DoubleGrid;
        let mut table = read_lipsum_text()?;
        let (widths, wrapped_table) = fill_lipsum_table(&mut table, &renderer)?;
        let s = renderer.render_table(&wrapped_table, &widths);
        assert_eq!(
            s,
            r#"╔═════════════════╦═══════════════╗
║ Duis facilisis. ║ Quisque ex    ║
║                 ║ nibh, auctor  ║
║                 ║ eu sodales.   ║
╠═════════════════╬═══════════════╣
║ Maecenas        ║               ║
║ blandit elit.   ║               ║
╠═════════════════╬═══════════════╣
║ Sed lobortis,   ║ Mauris enim.  ║
║ nibh vitae.     ║               ║
╚═════════════════╩═══════════════╝"#
        );
        Ok(())
    }

    #[test]
    fn test_fancy_grid() -> crate::Result<()> {
        let renderer = FancyGrid;
        let mut table = read_lipsum_text()?;
        let (widths, wrapped_table) = fill_lipsum_table(&mut table, &renderer)?;
        let s = renderer.render_table(&wrapped_table, &widths);
        assert_eq!(
            s,
            r#"╒═════════════════╤═══════════════╕
│ Duis facilisis. │ Quisque ex    │
│                 │ nibh, auctor  │
│                 │ eu sodales.   │
╞═════════════════╪═══════════════╡
│ Maecenas        │               │
│ blandit elit.   │               │
├─────────────────┼───────────────┤
│ Sed lobortis,   │ Mauris enim.  │
│ nibh vitae.     │               │
╘═════════════════╧═══════════════╛"#
        );
        Ok(())
    }
}
