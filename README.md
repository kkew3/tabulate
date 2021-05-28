```
 _        _           _       _       
| |_ __ _| |__  _   _| | __ _| |_ ___ 
| __/ _` | '_ \| | | | |/ _` | __/ _ \
| || (_| | |_) | |_| | | (_| | ||  __/
 \__\__,_|_.__/ \__,_|_|\__,_|\__\___|
                                      
```

Make fixed-width plaintext grid table with multiline cell support.

Usage
-----

> copied from `tabulate --help`:

```
usage: tabulate [-h] [-W WIDTH_LIST] [-T TOTAL_WIDTH] [-B CHARS] [-y]
                 [-L {grid}] [-S] [-d DELIMITER]
                 [FILE]

Make fixed-width plaintext table with multi-line cell supports. Currently only
support grid table, but it's trivial to adapt it to other layout once the
table has been built. What plaintext table content is expected: <TAB> will be
regarded as field delimiter, <LF> (or <CRLF> if on Windows) as row delimiter,
and all the others as cell content.

positional arguments:
  FILE                  table content from which to read; if FILE is not
                        specified, the table content will be expected from
                        stdin

optional arguments:
  -h, --help            show this help message and exit
  -W WIDTH_LIST, --widths WIDTH_LIST
                        a comma-separated list of WIDTH (int) or `-'
                        specifying the width of each column; `-' implies that
                        the width of the underlying column can be decided by
                        the program in objective of minimizing the total
                        number of rows. Each WIDTH defines the maximum number
                        of characters in the cell per row, except that when
                        `-B' is specified, (WIDTH - 2) will be the maximum
                        number. Note, however, that the sum of WIDTHs does not
                        necessarily equal to the width of table, since the
                        table layout is not taken into account with WIDTHs.
  -T TOTAL_WIDTH, --table-width TOTAL_WIDTH
                        the total table width; if specified, unless WIDTH_LIST
                        contains at least one `-', TABLE_WIDTH may not imply
                        the actual table width rendered; default to terminal
                        width
  -B CHARS, --bullets CHARS
                        a set of characters used as leading bullets with
                        additional indentation; default none
  -y, --break-hyphen    to allow break on hyphen of long words
  -L {grid}, --layout {grid}
                        table layout; default to grid
  -S, --strict          to enable strict mode, where wrapped lines exceeding
                        the WIDTHs that will ruin the table layout are
                        forbidden
  -d DELIMITER, --delimiter DELIMITER
                        the column delimiter in input data, default to <TAB>
```

Installation
------------

Write a launcher script of `src/tabulate` and enjoy.

Launcher script example (bash)

```bash
#!/usr/bin/env bash
python3 src/tabulate "$@"
```

Launcher script example (dos-batch)

```batch
@py -3 src\tabulate %*
```

Optional Requirement
--------------------

- `cjkwrap`: for CJK language support

Example
-------

Type `tabulate -W14,56 example.txt` to convert [example.txt](example.txt) to [example-formatted.txt](example-formatted.txt).

> example.txt

```
Usage	tabulate [-h] [-W WIDTH_LIST] [FILE]
Description	Make fixed-width plaintext table with multi-line cell supports. Currently only support grid table, but it's trivial to adapt it to other layout once the table has been built. What plaintext table content is expected: <TAB> will be regarded as field delimiter, <LF> (or <CRLF> if on Windows) as row delimiter, and all the others as cell content.
FILE	table content from which to read; if FILE is not specified, the table content will be expected from stdin
-h, --help	show this help message and exit
-W WIDTH_LIST, --widths WIDTH_LIST	a comma-separated list of WIDTH (int) specifying the width of each column. Note, however, that the actual width of each field is (WIDTH-2) because of the left and right one-space paddings, and that the actual width of the entire table is (1+NF+sum(WIDTH)), NF being the max number of fields in each row, because of the column rulers.
```

> example-formatted.txt

```
+--------------+--------------------------------------------------------+
| Usage        | tabulate [-h] [-W WIDTH_LIST] [FILE]                   |
+--------------+--------------------------------------------------------+
| Description  | Make fixed-width plaintext table with multi-line cell  |
|              | supports. Currently only support grid table, but it's  |
|              | trivial to adapt it to other layout once the table has |
|              | been built. What plaintext table content is expected:  |
|              | <TAB> will be regarded as field delimiter, <LF> (or    |
|              | <CRLF> if on Windows) as row delimiter, and all the    |
|              | others as cell content.                                |
+--------------+--------------------------------------------------------+
| FILE         | table content from which to read; if FILE is not       |
|              | specified, the table content will be expected from     |
|              | stdin                                                  |
+--------------+--------------------------------------------------------+
| -h, --help   | show this help message and exit                        |
+--------------+--------------------------------------------------------+
| -W           | a comma-separated list of WIDTH (int) specifying the   |
| WIDTH_LIST,  | width of each column. Note, however, that the actual   |
| --widths     | width of each field is (WIDTH-2) because of the left   |
| WIDTH_LIST   | and right one-space paddings, and that the actual      |
|              | width of the entire table is (1+NF+sum(WIDTH)), NF     |
|              | being the max number of fields in each row, because of |
|              | the column rulers.                                     |
+--------------+--------------------------------------------------------+
```

## Difference with `PrettyTable`

[PrettyTable](https://pypi.org/project/PrettyTable/) is an awesome package to draw plaintext table.
However, this utility solves a different problem than `PrettyTable`.
The focus of this utility lies in fixed-width table, facilitating users to specify the width of each column themselves.
This way, multiline cell has builtin support.
This utility is not the right tool to display single line data in good alignment.


## Similar projects

- [table-layout](https://github.com/75lb/table-layout.git)
