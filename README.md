```
 _        _           _       _       
| |_ __ _| |__  _   _| | __ _| |_ ___ 
| __/ _` | '_ \| | | | |/ _` | __/ _ \
| || (_| | |_) | |_| | | (_| | ||  __/
 \__\__,_|_.__/ \__,_|_|\__,_|\__\___|
                                      
```

Format plaintext into fixed-width table with multiline cells.

# Usage

> copied from `tabulate --help`:

```
Format plain text into fixed-width table with multi-line cell by wrapping text
in each field

Usage: tabulate [OPTIONS] [FILENAME]

Arguments:
  [FILENAME]  The input stream, default to stdin

Options:
  -W, --widths <WIDTHS>            The column widths
  -T, --table-width <TABLE_WIDTH>  The table total width, default to terminal
                                   width
  -L, --layout <TABLE_LAYOUT>      The table layout, default to "grid_no_header"
  -S, --strict                     Specify to enable strict mode
  -d, --delimiter <DELIMITER>      The field delimiter in the input data,
                                   default to <TAB>
  -h, --help                       Print help
```

See [synopsis.md](docs/synopsis.md) for details.

# Installation

Download the pre-built binary from the release page, or clone and build using [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```bash
git clone https://github.com/kkew3/tabulate.git
cd tabulate
cargo install --path .
```

# Example

Type `tabulate -T35 docs/lipsum.txt` to format [lipsum.txt](examples/lipsum.txt):

> lipsum.txt

```
Duis facilisis.	Quisque ex nibh, auctor eu sodales.
Maecenas blandit elit.
Sed lobortis, nibh vitae.	Mauris enim.
```

Output:

```
+-----------------+---------------+
| Duis facilisis. | Quisque ex    |
|                 | nibh, auctor  |
|                 | eu sodales.   |
+-----------------+---------------+
| Maecenas        |               |
| blandit elit.   |               |
+-----------------+---------------+
| Sed lobortis,   | Mauris enim.  |
| nibh vitae.     |               |
+-----------------+---------------+
```

# Difference with `PrettyTable` and `python-tabulate`

[`PrettyTable`](https://pypi.org/project/PrettyTable/) and [`python-tabulate`](https://github.com/astanin/python-tabulate.git) are awesome packages to draw plaintext table.
However, while sharing some table layout with `python-tabulate`, this utility solves a different problem.
The focus of this utility lies in fixed-width table, facilitating users to specify the width of each column themselves (or let the program decide the column widths).
This way, multiline cell has builtin support.
This utility is not the right tool to display single line data in good alignment.

# Similar projects

- [table-layout](https://github.com/75lb/table-layout.git)

# Dev

## Tests

To test locally, run

```bash
cargo test
```

Note that randomized property tests are included, so it may take some time to finish the tests.

## Benchmark

Either benchmark the bisect algorithm with:

```bash
cargo bench --bench complete_user_widths -F bench-bisect
```

or the brute-force algorithm with:

```bash
cargo bench --bench complete_user_widths -F bench-brute
```
