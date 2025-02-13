use clap::Parser;

/// Wise analysis of Rust lines of code
///
/// Calculates lines of code of a rust project by finding all not-ignored .rs files, providing
/// counts for lines of code itself, comments and doc comments in test (both unit and integration
/// tests) and examples.
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// Prints out contents of the analyzed files line by line with the category the line was
    /// assigned
    #[arg(long, hide = true)]
    pub debug: bool,
    /// If set, will print out stats for each file separately
    #[arg(long)]
    pub by_file: bool,
}
