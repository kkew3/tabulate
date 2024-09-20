use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process::ExitCode;
use tabulate::column_planner::complete_user_widths;
use tabulate::table::{
    ensure_row_within_widths, fill_table, wrap_table, Table, TableRenderer,
    WrapOptionsVarWidths,
};
use tabulate::table_renderers::TableRenderers;
use tabulate::ui::Cli;

macro_rules! eprintln_and_exit {
    ( $err:ident, $code:literal ) => {{
        eprintln!("E: {}", $err);
        ExitCode::from($code)
    }};
}

fn main() -> ExitCode {
    let cli = match Cli::parse_and_validate() {
        Err(exit_code) => return exit_code,
        Ok(cli) => cli,
    };
    let reader: Box<dyn BufRead> = match cli.filename {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => {
            let file: tabulate::Result<File> =
                File::open(filename).map_err(Into::into);
            match file {
                Err(err) => return eprintln_and_exit!(err, 1),
                Ok(file) => Box::new(BufReader::new(file)),
            }
        }
    };
    let mut table = match Table::from_bufread(reader, &cli.read_opts) {
        Err(err) => return eprintln_and_exit!(err, 1),
        Ok(table) => table,
    };
    let nrows = table.nrows();
    let ncols = table.ncols();
    table.transpose();
    let mut wrap_opts = WrapOptionsVarWidths::default();
    let renderer = match TableRenderers::new(&cli.table_layout) {
        Err(err) => return eprintln_and_exit!(err, 1),
        Ok(renderer) => renderer,
    };
    let widths = match complete_user_widths(
        cli.user_widths.into_vec(ncols),
        cli.user_total_width,
        &table,
        &renderer,
        &mut wrap_opts,
    ) {
        Err(err) => return eprintln_and_exit!(err, 1),
        Ok(widths) => widths,
    };
    table.transpose();
    let mut table = wrap_table(&table, &widths, &mut wrap_opts);
    for row_idx in 0..nrows {
        let wrapped_row = table.row(row_idx).unwrap();
        if let Err(err) =
            ensure_row_within_widths(row_idx, wrapped_row, &widths)
        {
            if cli.strict {
                return eprintln_and_exit!(err, 1);
            }
            eprintln!("W: {}", err);
        }
    }
    fill_table(&mut table, &widths);
    println!("{}", renderer.render_table(&table, &widths));
    ExitCode::SUCCESS
}
