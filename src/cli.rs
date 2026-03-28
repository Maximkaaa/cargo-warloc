use std::fmt::Display;

use clap::{Parser, ValueEnum};

/// Wise analysis of Rust lines of code
///
/// Calculates lines of code of a rust project by finding all not-ignored .rs files, providing
/// counts for lines of code itself, comments and doc comments in test (both unit and integration
/// tests) and examples.
#[derive(Debug, Parser)]
pub struct Cli {
    /// Prints out contents of the analyzed files line by line with the category the line was
    /// assigned
    #[arg(long, hide = true)]
    pub debug: bool,
    /// If set, will print out stats for each file separately
    #[arg(long)]
    pub by_file: bool,

    /// Output format to print to standard output
    #[arg(long, default_value_t = OutputFormat::Tabular)]
    pub output_format: OutputFormat,

    /// Optional directory to use as the root of the file search.
    ///
    /// Defaults to "", in which case the git repository root is
    /// looked up and used as the root of the search, falling back
    /// to "." in case of not being within a git repository.
    ///
    /// Use "." to force stat collection from the current directory
    /// while still maintaining the Git LoC stats collection functionality.
    #[arg(default_value_t = String::from(""))]
    pub target_dir: String,
}

#[derive(Clone, Debug, Default, ValueEnum)]
#[value(rename_all = "lower")]
pub enum OutputFormat {
    #[default]
    Tabular,
    Csv,
    Json,
    Yaml,
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Tabular => "tabular",
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Csv => "csv",
        };
        f.write_str(s)
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(name = "cargo", bin_name = "cargo")]
pub enum CargoCli {
    #[command(name = "warloc")]
    Command(Cli),
}
