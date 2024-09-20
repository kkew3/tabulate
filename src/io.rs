use crate::table::Table;
use std::io::BufRead;

/// Options for reading table from input.
pub struct ReadOptions {
    /// The column separator
    pub sep: String,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self { sep: "\t".into() }
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
