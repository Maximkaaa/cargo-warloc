mod cli;
mod visitor;
mod warlocs;

use std::{
    io,
    path::{Path, PathBuf},
};

use clap::Parser;
use cli::{CargoCli, Cli};
use csv::WriterBuilder;
use ignore::Walk;
use visitor::Visitor;
use warlocs::Warlocs;

use crate::cli::OutputFormat;

fn main() {
    let CargoCli::Command(args) = CargoCli::parse();

    let mut stats = Warlocs::default();

    for rust_file in enumerate_rust_files(".") {
        let file_stats = calculate_file_stats(&rust_file, &args);
        stats += file_stats;
    }

    output_stats(&stats, &args.output_format);
}

fn calculate_file_stats(file_path: impl AsRef<Path>, args: &Cli) -> Warlocs {
    let stats = Visitor::new(&file_path, args.debug).visit_file();

    if args.by_file {
        println!("File name: {}", file_path.as_ref().to_str().unwrap());
        pretty_print_stats(&stats);
        println!();
    }

    stats
}

fn output_stats(stats: &Warlocs, output_format: &OutputFormat) {
    match output_format {
        OutputFormat::Tabular => pretty_print_stats(stats),
        OutputFormat::Csv => {
            let mut writer = WriterBuilder::new().from_writer(io::stdout());
            for loc in [stats.main, stats.tests, stats.examples] {
                writer
                    .serialize(loc)
                    .expect("CSV serialization should work")
            }
        }
        OutputFormat::Json => {
            print!(
                "{}",
                serde_json::to_string(&stats.serializable_totals())
                    .expect("JSON serialization should work")
            )
        }
        OutputFormat::Yaml => {
            print!(
                "{}",
                serde_yaml::to_string(&stats.serializable_totals())
                    .expect("YAML serialization should work")
            )
        }
    }
}

fn pretty_print_stats(stats: &Warlocs) {
    println!("File count: {}", stats.file_count);
    println!(
        "{0: <12} | {1: <12} | {2: <12} | {3: <12} | {4: <12} | {5: <12}",
        "Type", "Code", "Blank", "Doc comments", "Comments", "Total",
    );
    println!(
        "{0:-<12}-|-{1:-<12}-|-{2:-<12}-|-{3:-<12}-|-{4:-<12}-|-{5:-<12}",
        "", "", "", "", "", "",
    );

    println!(
        "{0: <12} | {1: <12} | {2: <12} | {3: <12} | {4: <12} | {5: <12}",
        "Main",
        stats.main.code,
        stats.main.whitespaces,
        stats.main.docs,
        stats.main.comments,
        stats.main.sum(),
    );
    println!(
        "{0: <12} | {1: <12} | {2: <12} | {3: <12} | {4: <12} | {5: <12}",
        "Tests",
        stats.tests.code,
        stats.tests.whitespaces,
        stats.tests.docs,
        stats.tests.comments,
        stats.tests.sum(),
    );
    println!(
        "{0: <12} | {1: <12} | {2: <12} | {3: <12} | {4: <12} | {5: <12}",
        "Examples",
        stats.examples.code,
        stats.examples.whitespaces,
        stats.examples.docs,
        stats.examples.comments,
        stats.examples.sum(),
    );
    println!(
        "{0:-<12}-|-{1:-<12}-|-{2:-<12}-|-{3:-<12}-|-{4:-<12}-|-{5:-<12}",
        "", "", "", "", "", "",
    );
    println!(
        "{0: <12} | {1: <12} | {2: <12} | {3: <12} | {4: <12} | {5: <12}",
        "",
        stats.code(),
        stats.whitespaces(),
        stats.docs(),
        stats.comments(),
        stats.sum(),
    );
}

fn enumerate_rust_files(root: impl AsRef<Path>) -> impl Iterator<Item = PathBuf> {
    Walk::new(root)
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_some_and(|e| e.is_file())
                && entry
                    .file_name()
                    .to_str()
                    .map(|name| name.ends_with(".rs"))
                    .unwrap_or(false)
        })
        .map(|entry| entry.into_path())
}
