# SYNOPSIS

Format plain text into fixed-width table with multi-line cell by wrapping
text in each field.

## COLUMN WIDTHS

To specify the column widths, use the `-W` option. For
example, `-W23,54,18` gives the column widths of a three-column table.
However, it soon becomes cumbersome to manually designate the widths. We
may instead opt to let the program decide the width of each column, with
the objective to minimize the number of lines taken by the table. To fully
automate the decision, simply skip the `-W` option. Otherwise, e.g.,
`-W'18,*,*'` tell the program to optimize the 2nd and 3rd columns, but
leave the first column width fixed as 18 ASCII characters wide.

## TABLE WIDTH

Option `-T <TABLE_WIDTH>` takes effect if and only if the column widths are
not fully specified. If `-T` is omitted, it will be set to the terminal
width.

## TABLE LAYOUT

Currently supported table layouts are:

- grid_no_header
- grid
- plain
- simple
- github
- simple_grid
- rounded_grid
- heavy_grid
- mixed_grid
- double_grid
- fancy_grid

## STRICT MODE

If strict mode is enabled, the program errors if any column is not wide
enough such that some text protrudes out some columns and ruin the table.
