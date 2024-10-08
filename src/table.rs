use std::borrow::Cow;

/// A table of `T`.
#[derive(Debug)]
pub struct Table<T> {
    cells: Vec<T>,
    nrows: usize,
}

/// Indicate that a vec can not be interpreted as a `Table`, since its len
/// modulo `nrows` is not zero.
#[derive(Debug)]
pub struct NotTableError;

impl<T> Table<T> {
    /// Construct a `Table` from a Vec.
    pub fn from_vec(
        cells: Vec<T>,
        nrows: usize,
    ) -> Result<Self, NotTableError> {
        if cells.len() % nrows == 0 {
            Ok(Self { cells, nrows })
        } else {
            Err(NotTableError)
        }
    }

    /// Number of rows.
    #[inline]
    pub const fn nrows(&self) -> usize {
        self.nrows
    }

    /// Number of cells.
    #[inline]
    pub fn ncells(&self) -> usize {
        self.cells.len()
    }

    /// Number of columns.
    #[inline]
    pub fn ncols(&self) -> usize {
        self.cells.len() / self.nrows()
    }

    /// Get all cells. Used in tests only.
    #[cfg(test)]
    pub(crate) fn cells(&self) -> &[T] {
        &self.cells
    }

    /// Get the cell at `(row_idx, col_idx)`.
    #[inline]
    pub fn get(&self, row_idx: usize, col_idx: usize) -> Option<&T> {
        let idx = row_idx * self.ncols() + col_idx;
        self.cells.get(idx)
    }

    /// Get the `row_idx`-th row from the table.
    #[inline]
    pub fn row(&self, row_idx: usize) -> Option<&[T]> {
        if row_idx < self.nrows() {
            let ncols = self.ncols();
            Some(&self.cells[row_idx * ncols..(row_idx + 1) * ncols])
        } else {
            None
        }
    }

    /// Get the `row_idx`-th row as mutable from the table.
    pub fn row_mut(&mut self, row_idx: usize) -> Option<&mut [T]> {
        if row_idx < self.nrows() {
            let ncols = self.ncols();
            Some(&mut self.cells[row_idx * ncols..(row_idx + 1) * ncols])
        } else {
            None
        }
    }

    /// Transpose the table.
    pub fn transpose(&mut self) {
        let coordinates = self.get_transpose_target_coordinates();
        let mut transposed_cells: Vec<_> = self
            .cells
            .drain(..)
            .zip(coordinates)
            .map(|(s, co)| CoordinateCell::new(s, co))
            .collect();
        transposed_cells.sort_by(|a, b| a.coordinate.cmp(&b.coordinate));
        let mut transposed_cells: Vec<_> =
            transposed_cells.drain(..).map(|x| x.into_cell()).collect();
        self.cells.append(&mut transposed_cells);
        self.nrows = self.ncols();
    }

    fn get_transpose_target_coordinates(
        &self,
    ) -> impl Iterator<Item = (usize, usize)> {
        let nrows = self.nrows();
        let ncols = self.ncols();
        let mut i: usize = 0; // cycle through 0..ncols
        let mut j: usize = 0; // cycle through 0..nrows
        std::iter::from_fn(move || {
            if j == nrows {
                None
            } else {
                let c = (i, j);
                i += 1;
                if i == ncols {
                    i = 0;
                    j += 1;
                }
                Some(c)
            }
        })
    }

    #[cfg(test)]
    fn get_transpose_target_coordinates_wrapper(
        nrows: usize,
        ncols: usize,
    ) -> impl Iterator<Item = (usize, usize)> {
        let len = nrows * ncols;
        let table = Table {
            nrows,
            cells: vec!["".to_string(); len],
        };
        table.get_transpose_target_coordinates()
    }
}

/// A cell with target coordinate, used to transpose a table.
struct CoordinateCell<T> {
    cell: T,
    coordinate: (usize, usize),
}

impl<T> CoordinateCell<T> {
    fn new(cell: T, coordinate: (usize, usize)) -> Self {
        Self { cell, coordinate }
    }

    fn into_cell(self) -> T {
        self.cell
    }
}

pub trait TableRenderer {
    /// Return the part of table width attributed to the table layout rather
    /// than the table content, given the `ncols` of a table.
    fn layout_width(&self, table_ncols: usize) -> usize;

    /// Render a filled table into string given the widths of each column.
    fn render_table(
        &self,
        filled_table: &Table<Vec<Cow<'_, str>>>,
        widths: &[usize],
    ) -> String;
}

impl TableRenderer for Box<dyn TableRenderer> {
    fn layout_width(&self, table_ncols: usize) -> usize {
        self.as_ref().layout_width(table_ncols)
    }

    fn render_table(
        &self,
        filled_table: &Table<Vec<Cow<'_, str>>>,
        widths: &[usize],
    ) -> String {
        self.as_ref().render_table(filled_table, widths)
    }
}

/// A wrapper over [`textwrap::Options`] that can change its `width` as needed.
pub struct WrapOptionsVarWidths<'a> {
    inner: textwrap::Options<'a>,
}

impl<'a> From<textwrap::Options<'a>> for WrapOptionsVarWidths<'a> {
    fn from(value: textwrap::Options<'a>) -> Self {
        Self { inner: value }
    }
}

impl<'a> Default for WrapOptionsVarWidths<'a> {
    fn default() -> Self {
        WrapOptionsVarWidths::from(
            // `79` is arbitrary, since it will be erased anyway.
            textwrap::Options::new(79)
                .break_words(false)
                .word_separator(textwrap::WordSeparator::UnicodeBreakProperties)
                .word_splitter(textwrap::WordSplitter::HyphenSplitter),
        )
    }
}

impl<'a> WrapOptionsVarWidths<'a> {
    /// Return reference to a [`textwrap::Options`] whose `width` attribute is
    /// set to `width`.
    pub fn as_width(&mut self, width: usize) -> &textwrap::Options<'a> {
        self.inner.width = width;
        &self.inner
    }
}

/// Wrap a row of strings. Return the wrapped lines of each cell along the row.
fn wrap_row<'s>(
    row: &'s [String],
    widths: &[usize],
    opts: &mut WrapOptionsVarWidths<'_>,
) -> Vec<Vec<Cow<'s, str>>> {
    row.iter()
        .zip(widths.iter())
        .map(|(s, w)| textwrap::wrap(s, opts.as_width(*w)))
        .collect()
}

/// Wrap every line of the table. Return the wrapped table.
pub fn wrap_table<'s>(
    table: &'s Table<String>,
    widths: &[usize],
    opts: &mut WrapOptionsVarWidths<'_>,
) -> Table<Vec<Cow<'s, str>>> {
    let nrows = table.nrows();
    let mut wrapped_cells = Vec::with_capacity(table.ncells());
    for i in 0..nrows {
        wrapped_cells.extend(wrap_row(table.row(i).unwrap(), widths, opts));
    }
    Table::from_vec(wrapped_cells, nrows).unwrap()
}

/// Fill a wrapped cell. `max_nlines` is the max number of lines of cells of
/// the row where current cell lies in.
fn fill_cell(
    wrapped_cell: &mut Vec<Cow<'_, str>>,
    width: usize,
    max_nlines: usize,
) {
    for line in wrapped_cell.iter_mut() {
        let padded = " "
            .repeat(width.saturating_sub(textwrap::core::display_width(line)));
        line.to_mut().push_str(&padded);
    }
    let nlines = wrapped_cell.len();
    for _ in 0..max_nlines.saturating_sub(nlines) {
        let padded = " ".repeat(width);
        wrapped_cell.push(Cow::from(padded));
    }
}

/// Fill the wrapped table, assuming the table is non-empty.
pub fn fill_table(table: &mut Table<Vec<Cow<'_, str>>>, widths: &[usize]) {
    let nrows = table.nrows();
    for i in 0..nrows {
        let wrapped_row = table.row_mut(i).unwrap();
        let max_nlines = wrapped_row.iter().map(|r| r.len()).max().unwrap();
        for (wrapped_cell, w) in wrapped_row.iter_mut().zip(widths.iter()) {
            fill_cell(wrapped_cell, *w, max_nlines);
        }
    }
}

/// Ensure all lines in a wrapped row is within corresponding width in
/// `widths`. The `row_idx` is needed to prepare the error message.
pub fn ensure_row_within_widths(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_get_transpose_target_coordinates() {
        let coordinates: Vec<_> =
            Table::<i32>::get_transpose_target_coordinates_wrapper(3, 4)
                .collect();
        assert_eq!(
            coordinates,
            vec![
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (3, 1),
                (0, 2),
                (1, 2),
                (2, 2),
                (3, 2),
            ]
        );
    }

    #[test]
    fn test_table_transpose() {
        let mut table = Table {
            nrows: 2,
            cells: ["a", "b", "c", "d", "e", "f"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        };
        table.transpose();

        assert_eq!(
            table.cells,
            vec![
                "a".to_string(),
                "d".to_string(),
                "b".to_string(),
                "e".to_string(),
                "c".to_string(),
                "f".to_string(),
            ]
        );
        assert_eq!(table.nrows, 3);
    }

    #[test]
    fn test_fill_cell() {
        let mut cell = vec![Cow::from("abcde")];
        fill_cell(&mut cell, 10, 2);
        assert_eq!(
            cell,
            vec![Cow::from("abcde     "), Cow::from("          ")]
        );

        let mut cell = vec![Cow::from("12345678")];
        fill_cell(&mut cell, 5, 1);
        assert_eq!(cell, vec![Cow::from("12345678")]);

        let mut cell = vec![Cow::from("12345678")];
        fill_cell(&mut cell, 5, 2);
        assert_eq!(cell, vec![Cow::from("12345678"), Cow::from("     ")]);
    }
}
