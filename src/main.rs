mod cli;
mod output;
mod visitor;
mod warlocs;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use clap::Parser;
use cli::{CargoCli, Cli};
use ignore::Walk;
use visitor::Visitor;
use warlocs::Warlocs;

use crate::output::{output_multiple_file_stats, output_total_stats};

fn main() {
    let CargoCli::Command(args) = CargoCli::parse();

    let root_dir = PathBuf::from(".");

    let files_stats: HashMap<PathBuf, Warlocs> = enumerate_rust_files(&root_dir)
        .map(|p| (p.clone(), calculate_file_stats(&p, &args)))
        .collect();

    if args.by_file {
        output_multiple_file_stats(&args.output_format, files_stats);
    } else {
        let total_stats = files_stats.into_values().sum();
        output_total_stats(&root_dir, &total_stats, &args.output_format);
    }
}

fn calculate_file_stats(file_path: impl AsRef<Path>, args: &Cli) -> Warlocs {
    Visitor::new(&file_path, args.debug).visit_file()
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
