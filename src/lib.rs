pub mod core;
pub mod io;
pub(crate) mod try_wrap;
pub mod table_renderers;

#[derive(Debug)]
pub enum Error {
    /// If the input table is empty.
    EmptyTable,
    /// If IO error occurs while reading the input table.
    Io(std::io::Error),
    /// If wrapped line in a cell `(row_idx, col_idx)` is too long to fit
    /// within given width. If it's not possible to know the cell coordinate,
    /// a `None` may be used instead.
    ColumnNotWideEnough(Option<(usize, usize)>),
    /// If the command line argument is invalid. The string is the error
    /// message.
    InvalidArgument(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
