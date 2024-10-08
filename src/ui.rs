use crate::io::ReadOptions;
use clap::Parser;
use std::fmt::{Display, Formatter};
use std::process::ExitCode;

#[derive(Debug, Clone)]
pub struct UserWidths(Vec<Option<usize>>);

impl TryFrom<Option<String>> for UserWidths {
    type Error = String;

    fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
        let mut user_widths = vec![];
        if let Some(value) = value {
            for s in value.split(',') {
                let uw = if s == "*" {
                    None
                } else {
                    Some(s.parse().map_err(|_| {
                        format!("width `{}` is not a nonnegative integer", s)
                    })?)
                };
                user_widths.push(uw);
            }
        }
        Ok(Self(user_widths))
    }
}

impl UserWidths {
    pub fn into_vec(self, ncols: usize) -> Vec<Option<usize>> {
        let mut user_widths = self.0;
        let len = user_widths.len();
        if len < ncols {
            let n_rest = ncols - len;
            // If `len` is zero, it's most likely that user skips the option
            // instead of misspelling it.
            if len > 0 {
                eprintln!("W: Padding USER_WIDTHS with `*`");
            }
            user_widths.extend(std::iter::repeat(None).take(n_rest));
        } else if len > ncols {
            eprintln!("W: Truncating USER_WIDTHS to ncols={}", ncols);
            user_widths.truncate(ncols);
        }
        user_widths
    }
}

/// Format plain text into fixed-width table with multi-line cell by wrapping
/// text in each field.
#[derive(Parser, Debug)]
pub struct Cli {
    /// The column widths.
    #[arg(short = 'W', long = "widths", name = "WIDTHS")]
    user_widths: Option<String>,
    /// The table total width, default to terminal width.
    #[arg(short = 'T', long = "table-width", name = "TABLE_WIDTH")]
    user_total_width: Option<usize>,
    /// The table layout, default to "grid_no_header".
    #[arg(short = 'L', long = "layout")]
    table_layout: Option<String>,
    /// Specify to enable strict mode.
    #[arg(short = 'S', long, default_value_t = false)]
    strict: bool,
    /// The field delimiter in the input data, default to <TAB>.
    #[arg(short = 'd', long = "delimiter", name = "DELIMITER")]
    field_delimiter: Option<String>,
    /// Specify to enable escape sequence as `echo -e` in input data.
    #[arg(short = 'e', long = "escape", default_value_t = false)]
    enable_escape_sequence: bool,
    /// The input stream, default to stdin.
    filename: Option<String>,
}

/// Post-processed [`Cli`] arguments.
pub struct PostCli {
    pub user_widths: UserWidths,
    pub user_total_width: Option<usize>,
    pub table_layout: String,
    pub strict: bool,
    pub read_opts: ReadOptions,
    pub filename: Option<String>,
}

impl Cli {
    pub fn parse_and_validate() -> Result<PostCli, ExitCode> {
        let cli = Self::parse();
        let user_widths =
            UserWidths::try_from(cli.user_widths).map_err(|msg| {
                eprintln!("E: {}", msg);
                ExitCode::from(1)
            })?;
        let table_layout = cli.table_layout.unwrap_or("grid_no_header".into());
        let mut read_opts = ReadOptions::default();
        if let Some(field_delimiter) = cli.field_delimiter {
            read_opts.sep = field_delimiter;
        }
        if cli.enable_escape_sequence {
            read_opts.enable_backslash_escape = true;
        }
        Ok(PostCli {
            user_widths,
            user_total_width: cli.user_total_width,
            table_layout,
            strict: cli.strict,
            read_opts,
            filename: cli.filename,
        })
    }
}

impl Display for crate::Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::Error::EmptyTable => write!(f, "The input table is empty."),
            crate::Error::Utf8(err) => {
                write!(f, "The input is not valid utf-8: {}", err)
            }
            crate::Error::Io(err) => write!(f, "IO error occurs: {}.", err),
            crate::Error::ColumnNotWideEnough(cell) => match cell.as_ref() {
                None => write!(f, "Some columns are not wide enough."),
                Some((row_idx, col_idx)) => {
                    write!(
                        f,
                        "Column is not wide enough at row={} column={}.",
                        row_idx + 1,
                        col_idx + 1
                    )
                }
            },
            crate::Error::TotalWidthNotLargeEnough(total_width) => {
                write!(f, "Table width {} is not large enough to falicitate the columns and/or table layout.", total_width)
            }
            crate::Error::InvalidTableLayout(name) => {
                write!(f, "Invalid layout `{}`", name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::UserWidths;

    #[test]
    fn test_user_widths() {
        assert_eq!(
            UserWidths::try_from(None).unwrap().into_vec(3),
            vec![None; 3]
        );
        assert_eq!(
            UserWidths::try_from(Some("4,*,8".into()))
                .unwrap()
                .into_vec(3),
            vec![Some(4), None, Some(8)]
        );
        assert_eq!(
            UserWidths::try_from(Some("4,*".into()))
                .unwrap()
                .into_vec(3),
            vec![Some(4), None, None]
        );
    }
}
