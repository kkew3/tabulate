use crate::table::Table;
use std::io::BufRead;

/// Options for reading table from input.
pub struct ReadOptions {
    /// The column separator
    pub sep: String,
    pub enable_backslash_escape: bool,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            sep: "\t".into(),
            enable_backslash_escape: false,
        }
    }
}

// Adapted from
// https://github.com/uutils/coreutils/blob/main/src/uu/echo/src/echo.rs.
mod escape {
    use std::io::{self, Write};
    use std::iter::Peekable;
    use std::str::Chars;

    #[repr(u8)]
    #[derive(Clone, Copy)]
    enum Base {
        Oct = 8,
        Hex = 16,
    }

    impl Base {
        const fn max_digits(&self) -> u8 {
            match self {
                Self::Oct => 3,
                Self::Hex => 2,
            }
        }
    }

    /// Parse the numeric part of the `\xHHH` and `\0NNN` escape sequences
    fn parse_code(input: &mut Peekable<Chars>, base: Base) -> Option<u8> {
        // All arithmetic on `ret` needs to be wrapping, because octal input can
        // take 3 digits, which is 9 bits, and therefore more than what fits in a
        // `u8`. GNU just seems to wrap these values.
        // Note that if we instead make `ret` a `u32` and use `char::from_u32` will
        // yield incorrect results because it will interpret values larger than
        // `u8::MAX` as unicode.
        let mut ret = input.peek().and_then(|c| c.to_digit(base as u32))? as u8;

        // We can safely ignore the None case because we just peeked it.
        let _ = input.next();

        for _ in 1..base.max_digits() {
            match input.peek().and_then(|c| c.to_digit(base as u32)) {
                Some(n) => {
                    ret = ret.wrapping_mul(base as u8).wrapping_add(n as u8)
                }
                None => break,
            }
            // We can safely ignore the None case because we just peeked it.
            let _ = input.next();
        }

        Some(ret)
    }

    enum CharByte {
        Char(char),
        Byte(u8),
    }

    impl From<char> for CharByte {
        fn from(value: char) -> Self {
            Self::Char(value)
        }
    }

    impl From<u8> for CharByte {
        fn from(value: u8) -> Self {
            Self::Byte(value)
        }
    }

    pub fn print_escaped(input: &str, output: &mut Vec<u8>) -> io::Result<()> {
        let mut iter = input.chars().peekable();
        while let Some(c) = iter.next() {
            if c != '\\' {
                write!(output, "{}", c)?;
                continue;
            }

            // This is for the \NNN syntax for octal sequences.
            // Note that '0' is intentionally omitted because that
            // would be the \0NNN syntax.
            if let Some('1'..='8') = iter.peek() {
                if let Some(parsed) = parse_code(&mut iter, Base::Oct) {
                    write!(output, "{}", parsed)?;
                    continue;
                }
            }

            if let Some(next) = iter.next() {
                let unescaped: CharByte = match next {
                    '\\' => '\\'.into(),
                    'a' => '\x07'.into(),
                    'b' => '\x08'.into(),
                    'c' => return Ok(()),
                    'e' => '\x1b'.into(),
                    'f' => '\x0c'.into(),
                    'n' => '\n'.into(),
                    'r' => '\r'.into(),
                    't' => '\t'.into(),
                    'v' => '\x0b'.into(),
                    'x' => {
                        if let Some(c) = parse_code(&mut iter, Base::Hex) {
                            c.into()
                        } else {
                            write!(output, "\\")?;
                            'x'.into()
                        }
                    }
                    '0' => {
                        parse_code(&mut iter, Base::Oct).unwrap_or(b'\0').into()
                    }
                    c => {
                        write!(output, "\\")?;
                        c.into()
                    }
                };
                match unescaped {
                    CharByte::Char(c) => write!(output, "{}", c)?,
                    CharByte::Byte(b) => output.push(b),
                }
            } else {
                write!(output, "\\")?;
            }
        }

        Ok(())
    }
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
            } else if opts.enable_backslash_escape {
                let mut row = vec![];
                for s in line.split(&opts.sep) {
                    let mut buf = Vec::with_capacity(s.len());
                    // Why unwrap: write to Vec<u8> never fails.
                    escape::print_escaped(s, &mut buf).unwrap();
                    let buf = String::from_utf8(buf)?;
                    row.push(buf);
                }
                row
            } else {
                line.split(&opts.sep).map(ToOwned::to_owned).collect()
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
        Ok(Self::from_vec(cells, nrows).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn test_table_from_bufread() {
        let input = "foo\tbar\nfoo2\tbar2\tbaz\nfoo3\n\n".as_bytes().to_vec();
        let reader = BufReader::new(Cursor::new(input));
        let opts = ReadOptions::default();
        let table = Table::from_bufread(reader, &opts).unwrap();
        assert_eq!(table.nrows(), 4);
        assert_eq!(
            table.cells(),
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
    fn test_table_from_bufread_escape_seq() {
        let input = "foo\\xf0\\x9f\\x98\\x82\tbar\\nbaz\n";
        let reader = BufReader::new(Cursor::new(input));
        let opts = ReadOptions {
            sep: "\t".into(),
            enable_backslash_escape: true,
        };
        let table = Table::from_bufread(reader, &opts).unwrap();
        assert_eq!(table.nrows(), 1);
        assert_eq!(
            table.cells(),
            vec!["foo😂".to_string(), "bar\nbaz".to_string()]
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
