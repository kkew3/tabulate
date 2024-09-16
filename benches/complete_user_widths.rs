use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand_chacha::rand_core::SeedableRng;
use tabulate::io::Table;
use tabulate::table_renderers::NullTableRenderer;
use textwrap::{WordSeparator, WordSplitter};

const AVERAGE_COLUMN_WIDTH: usize = 35;
const NUM_WRODS_IN_CELL: usize = 25;
const NUM_ROWS: usize = 5;

fn generate_transposed_table(ncols: usize, mut seed: u64) -> Table<String> {
    let cells: Vec<_> = (0..NUM_ROWS * ncols)
        .map(|_| {
            let rng = rand_chacha::ChaChaRng::seed_from_u64(seed);
            seed = seed.wrapping_add(1);
            lipsum::lipsum_words_with_rng(rng, NUM_WRODS_IN_CELL)
        })
        .collect();
    let mut table = Table::from_vec(cells, NUM_ROWS).unwrap();
    table.transpose();
    table
}

fn worker(ncols: usize, transposed_table: &Table<String>) {
    tabulate::core::complete_user_widths(
        vec![None; ncols],
        Some(AVERAGE_COLUMN_WIDTH * ncols),
        transposed_table,
        &NullTableRenderer,
        textwrap::Options::new(80)
            .word_splitter(WordSplitter::HyphenSplitter)
            .word_separator(WordSeparator::AsciiSpace)
            .break_words(false),
    )
    .unwrap();
}

#[cfg(any(
    all(feature = "bench-brute", feature = "bench-bisect"),
    not(any(feature = "bench-brute", feature = "bench-bisect")),
))]
compile_error!(
    "`bench-brute` XOR `bench-bisect` must be activated to run this benchmark."
);
#[cfg(all(feature = "bench-brute", not(feature = "bench-bisect")))]
const BENCH_NAME: &str = "brute";
#[cfg(all(feature = "bench-bisect", not(feature = "bench-brute")))]
const BENCH_NAME: &str = "bisect";

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group(BENCH_NAME);
    let ncols_choices = [1usize, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20];
    let seeds = 0..ncols_choices.len() as u64;
    for (&ncols, seed) in ncols_choices.iter().zip(seeds) {
        let transposed_table = generate_transposed_table(ncols, seed);
        group.bench_with_input(
            BenchmarkId::from_parameter(ncols),
            &(ncols, transposed_table),
            |b, (ncols, transposed_table)| {
                b.iter(|| worker(*ncols, transposed_table))
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
