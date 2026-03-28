mod cli;
mod git;
mod pathutil;
mod visitor;
mod warlocs;

use std::{
    fs::canonicalize,
    io,
    path::{Path, PathBuf},
    process::exit,
};

use clap::Parser;
use cli::{CargoCli, Cli};
use csv::WriterBuilder;
use ignore::Walk;
use visitor::Visitor;
use warlocs::Warlocs;

use crate::{cli::OutputFormat, git::GitContext};

fn main() {
    let CargoCli::Command(args) = CargoCli::parse();

    let target_dir = match args.target_dir.as_str() {
        "" => ".",
        td => td,
    };

    let mut target_dir = match canonicalize(target_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "Provided target directory {:?} does not exist: {e:?}",
                args.target_dir
            );
            exit(1)
        }
    };

    let mut git_context = match GitContext::from_path_in_repo(&target_dir) {
        Ok(gc) => {
            // If no target dir was provided and repo initalization has succeeded,
            // use the git repo root as the root of the file search.
            if args.target_dir.is_empty() {
                let git_dir = gc
                    .repo
                    .path()
                    .parent()
                    .expect("Git repo path is canonicalized and should always have a parent")
                    .to_path_buf();
                if git_dir != target_dir {
                    eprintln!("INFO: using Git repo root path {git_dir:?} for file search. Pass '.' as an argument to force CWD.");
                    target_dir = git_dir;
                }
            }
            Some(gc)
        }
        Err(e) => {
            eprintln!("WARN: Skipping LoC timestamp stat aggregation, failed to open Git repository: {e:?}");
            None
        }
    };

    let mut stats = Warlocs::default();
    for rust_file in enumerate_rust_files(&target_dir) {
        let file_stats = calculate_file_stats(&mut git_context, &rust_file, &args);
        stats += file_stats;
    }

    output_stats(&stats, &args.output_format);
}

fn calculate_file_stats(
    git_context: &mut Option<GitContext>,
    file_path: impl AsRef<Path>,
    args: &Cli,
) -> Warlocs {
    let visitor = match git_context {
        Some(git_context) => Visitor::new_with_git(git_context, &file_path, args.debug),
        None => Visitor::new(&file_path, args.debug),
    };

    let stats = visitor.visit_file();

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
    eprintln!("INFO: Processing all Rust files under: {:?}", root.as_ref());
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
