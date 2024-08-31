use std::io::BufRead;
use std::iter;

/// Options for reading table from input.
pub struct ReadOptions {
    /// The column separator
    pub sep: &'static str,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self { sep: "\t" }
    }
}

/// A table of `T`.
#[derive(Debug)]
pub struct Table<T> {
    cells: Vec<T>,
    nrows: usize,
}

impl Table<String> {
    /// Construct a string table from reader.
    pub fn from_bufread<R: BufRead>(
        reader: R,
        opts: &ReadOptions,
    ) -> crate::Result<Self> {
        // These rows may contain different number of fields. We will need to
        // pad empty strings accordingly to normalize the table.
        let mut rows_before_norm: Vec<Vec<String>> = vec![];
        for line in reader.lines() {
            let line: String = line?;
            let row: Vec<String> = if line.is_empty() {
                vec![]
            } else {
                line.split(opts.sep).map(ToOwned::to_owned).collect()
            };
            rows_before_norm.push(row);
        }
        // Max number of fields in the rows read.
        let max_nfields: usize = rows_before_norm
            .iter()
            .map(|row| row.len())
            .max()
            .ok_or(crate::Error::EmptyTable)?;
        if max_nfields == 0 {
            return Err(crate::Error::EmptyTable);
        }
        let mut cells =
            Vec::with_capacity(rows_before_norm.len() * max_nfields);
        let mut nrows: usize = 0;
        for row in rows_before_norm.iter_mut() {
            let nfield = row.len();
            cells.append(row);
            cells.extend((0..max_nfields - nfield).map(|_| "".to_string()));
            nrows += 1;
        }
        Ok(Self { nrows, cells })
    }
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
        iter::from_fn(move || {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

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
    fn test_table_from_bufread() {
        let input = "foo\tbar\nfoo2\tbar2\tbaz\nfoo3\n\n".as_bytes().to_vec();
        let reader = BufReader::new(Cursor::new(input));
        let opts = ReadOptions::default();
        let table = Table::from_bufread(reader, &opts).unwrap();
        assert_eq!(table.nrows(), 4);
        assert_eq!(
            table.cells,
            vec![
                "foo".to_string(),
                "bar".to_string(),
                "".to_string(),
                "foo2".to_string(),
                "bar2".to_string(),
                "baz".to_string(),
                "foo3".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ]
        );
    }

    #[test]
    fn test_empty_table_from_bufread() {
        let input = "".as_bytes().to_vec();
        let reader = BufReader::new(Cursor::new(input));
        let opts = ReadOptions::default();
        let table = Table::from_bufread(reader, &opts);
        assert!(matches!(table, Err(crate::Error::EmptyTable)));

        let input = "\n\n\n".as_bytes().to_vec();
        let reader = BufReader::new(Cursor::new(input));
        let opts = ReadOptions::default();
        let table = Table::from_bufread(reader, &opts);
        assert!(matches!(table, Err(crate::Error::EmptyTable)));
    }
}
