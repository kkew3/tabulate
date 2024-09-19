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

    /// Render a wrapped table into string given the widths of each column.
    fn render_table(
        &self,
        wrapped_table: &Table<Vec<String>>,
        widths: &[usize],
    ) -> String;
}

/// A wrapper over [`textwrap::Options`] that can change its `width` as needed.
pub struct WrapOptionsVarWidths<'a> {
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
    pub fn as_width(&mut self, width: usize) -> &textwrap::Options<'a> {
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
}
