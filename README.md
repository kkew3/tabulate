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
usage: tabulate [-h] [-W WIDTH_LIST] [-T TABLE_WIDTH] [-L {grid,hline}] [-S]
                [-B] [-d DELIMITER]
                [FILE]

Make fixed-width plaintext table with multi-line cell supports. What plaintext
table content is expected: <TAB> will be regarded as field delimiter, <LF> (or
<CRLF> if on Windows) as row delimiter, and all the others as cell content.

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
                        the program, in objective of minimizing the number of
                        rowsof the table. Each WIDTH defines the maximum
                        number of characters in the cell per row. Note,
                        however, that the sum of WIDTHs does not necessarily
                        equal to the width of table, since the table layout is
                        not taken into account with WIDTHs. One CJK character
                        takes up two units of width. If `-W WIDTH_LIST' is not
                        specified, the entire WIDTH_LIST will be decided by
                        the program, in the same objective as above mentioned
  -T TABLE_WIDTH, --table-width TABLE_WIDTH
                        the total table width. If specified, unless WIDTH_LIST
                        contains `-', and TABLE_WIDTH is sufficiently large,
                        TABLE_WIDTH may not imply the actual table width
                        rendered. Default to terminal width
  -L {grid,hline}, --layout {grid,hline}
                        table layout; default to grid
  -S, --strict          to enable strict mode, where wrapped lines exceeding
                        the WIDTHs that will ruin the table layout are
                        forbidden
  -B, --no-break-long-words
                        to not break long words even if necessary, default to
                        break long words
  -d DELIMITER, --delimiter DELIMITER
                        the field delimiter in input data, default to <TAB>
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

Requirement
-----------

- `more_bisect`: used to perform binary search of the optimal table when WIDTH_LIST is not fully specified.

Optional Requirement
--------------------

- `cjkwrap`: for CJK language support. Notice that CJK characters align correctly only under font where the width of one CJK character equals two ASCII characters.

Note
----

Breaking long words (the default behavior when not specifying `-B` option) seems necessary for `cjkwrap` to effectively wrap CJK characters.
When forming table without CJK characters, remember to specify `-B` if you don't want to break long words.

Example
-------

Type `tabulate -W14,56 example.txt` to convert [example.txt](example.txt) to [example-formatted.txt](example-formatted.txt).

> example.txt

```
Usage	tabulate [-h] [-W WIDTH_LIST] [FILE]
Description	Make fixed-width plaintext table with multi-line cell supports. Currently only support grid table, but it's trivial to adapt it to other layout once the table has been built. What plaintext table content is expected: <TAB> will be regarded as field delimiter, <LF> (or <CRLF> if on Windows) as row delimiter, and all the others as cell content.
FILE	table content from which to read; if FILE is not specified, the table content will be expected from stdin
-h, --help	show this help message and exit
```

> example-formatted.txt

```
+----------------+----------------------------------------------------------+
| Usage          | tabulate [-h] [-W WIDTH_LIST] [FILE]                     |
+----------------+----------------------------------------------------------+
| Description    | Make fixed-width plaintext table with multi-line cell    |
|                | supports. Currently only support grid table, but it's    |
|                | trivial to adapt it to other layout once the table has   |
|                | been built. What plaintext table content is expected:    |
|                | <TAB> will be regarded as field delimiter, <LF> (or      |
|                | <CRLF> if on Windows) as row delimiter, and all the      |
|                | others as cell content.                                  |
+----------------+----------------------------------------------------------+
| FILE           | table content from which to read; if FILE is not         |
|                | specified, the table content will be expected from stdin |
+----------------+----------------------------------------------------------+
| -h, --help     | show this help message and exit                          |
+----------------+----------------------------------------------------------+
```

Type `tabulate -W14,- -T62 -Lhline example_cjk.txt` to convert [example_cjk.txt](example_cjk.txt) to [example_cjk-formatted.txt](example_cjk-formatted.txt).

> example_cjk.txt

```
用法	tabulate [-h] [-W WIDTH_LIST] [FILE]
描述	制作一个带有多行文字单元格支持的纯文本表格。目前只支持 grid 样式的表格，但是一旦表格制作完成，改变它的样式将会很容易。纯文本表格的输入应为：<TAB> 为列分隔符，<LF>（在 Windows 上则为 <CRLF>）为行分隔符，其它所有字符为单元格内容。
FILE	待读入的表格内容；如果 FILE 没有指定，那么表格内容将从标准输入流读取
-h, --help	显示帮助信息然后退出
```

> example_cjk-formatted.txt

```
==============  ==============================================
用法            tabulate [-h] [-W WIDTH_LIST] [FILE]
--------------  ----------------------------------------------
描述            制作一个带有多行文字单元格支持的纯文本表格。目
                前只支持 grid 样式的表格，但是一旦表格制作完成
                ，改变它的样式将会很容易。纯文本表格的输入应为
                ：<TAB> 为列分隔符，<LF>（在 Windows 上则为
                <CRLF>）为行分隔符，其它所有字符为单元格内容。
--------------  ----------------------------------------------
FILE            待读入的表格内容；如果 FILE
                没有指定，那么表格内容将从标准输入流读取
--------------  ----------------------------------------------
-h, --help      显示帮助信息然后退出
==============  ==============================================
```

## Difference with `PrettyTable` and `python-tabulate`

[PrettyTable](https://pypi.org/project/PrettyTable/) and [python-tabulate](https://github.com/astanin/python-tabulate.git) are awesome packages to draw plaintext table.
However, this utility solves a different problem.
The focus of this utility lies in fixed-width table, facilitating users to specify the width of each column themselves.
This way, multiline cell has builtin support.
This utility is not the right tool to display single line data in good alignment.


## Similar projects

- [table-layout](https://github.com/75lb/table-layout.git)
