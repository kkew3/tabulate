use std::string::FromUtf8Error;

pub mod column_planner;
pub mod io;
pub mod table;
pub mod table_renderers;
pub(crate) mod try_wrap;
pub mod ui;

#[derive(Debug)]
pub enum Error {
    /// If the input table is empty.
    EmptyTable,
    /// If input is not valid utf-8.
    Utf8(FromUtf8Error),
    /// If IO error occurs while reading the input table.
    Io(std::io::Error),
    /// If wrapped line in a cell `(row_idx, col_idx)` is too long to fit
    /// within given width. If it's not possible to know the cell coordinate,
    /// a `None` may be used instead.
    ColumnNotWideEnough(Option<(usize, usize)>),
    /// If total width is not wide enough to support user widths and the
    /// underlying table layout.
    TotalWidthNotLargeEnough(usize),
    /// Unrecognized table layout. The wrapped string is the layout name.
    InvalidTableLayout(String),
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
