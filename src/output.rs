//! Output routines.

use std::{
    collections::HashMap,
    io::stdout,
    path::{Path, PathBuf},
};

use csv::Writer as CsvWriter;
use serde::Serialize;

use crate::{cli::OutputFormat, warlocs::Warlocs};

/// Prints to stdout multiple stats formatted based on the given `output_format`.
pub fn output_multiple_file_stats(output_format: &OutputFormat, stats: HashMap<PathBuf, Warlocs>) {
    match output_format {
        OutputFormat::Tabular => {
            println!("Total file count: {}", stats.len());
            for (path, warloc) in stats.iter() {
                println!("\nFile path : {}", path.to_str().unwrap());
                single_stat_tabular(warloc);
            }
        }
        OutputFormat::Json => {
            let multistats = SerializableMultiFileStats::from_file_stats(stats);
            println!(
                "{}",
                serde_json::to_string(&multistats).expect("JSON serialization should work")
            )
        }
        OutputFormat::Yaml => {
            let multistats = SerializableMultiFileStats::from_file_stats(stats);
            println!(
                "{}",
                serde_yaml::to_string(&multistats).expect("YAML serialization should work")
            )
        }
        OutputFormat::Csv => {
            let multistats = SerializableMultiFileStats::from_file_stats(stats);
            csv_output::output_csv(&multistats.totals, &multistats.files, stdout());
        }
    }
}

/// Prints to stdout the provided total [Warloc] stats formatted to the given `output_format`.
pub fn output_total_stats(
    root_dir: impl AsRef<Path>,
    stats: &Warlocs,
    output_format: &OutputFormat,
) {
    match output_format {
        OutputFormat::Tabular => single_stat_tabular(stats),
        OutputFormat::Json => {
            print!(
                "{}",
                serde_json::to_string(&stats).expect("JSON serialization should work")
            )
        }
        OutputFormat::Yaml => {
            print!(
                "{}",
                serde_yaml::to_string(&stats).expect("YAML serialization should work")
            )
        }
        OutputFormat::Csv => {
            let mut writer = CsvWriter::from_writer(stdout());
            csv_output::write_csv_header_row(&mut writer);
            csv_output::write_single_csv_value(root_dir, stats, &mut writer);
        }
    }
}

/// Prints to stdout a tabular representation for a single [Warlocs] values.
fn single_stat_tabular(stats: &Warlocs) {
    if stats.file_count >= 1 {
        println!("File count: {}", stats.file_count);
    }
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

/// Simple wrapper struct representing the structure of serialized multi-file output.
#[derive(Serialize)]
struct SerializableMultiFileStats {
    totals: Warlocs,
    files: HashMap<PathBuf, Warlocs>,
}

impl SerializableMultiFileStats {
    fn from_file_stats(files: HashMap<PathBuf, Warlocs>) -> Self {
        SerializableMultiFileStats {
            totals: files.values().fold(Warlocs::default(), |acc, v| acc + *v),
            files,
        }
    }
}

/// Module defining CSV output routines.
mod csv_output {
    use std::{
        collections::HashMap,
        io::Write,
        path::{Path, PathBuf},
    };

    use csv::Writer as CsvWriter;

    use crate::warlocs::Warlocs;

    /// Performs CSV serialization and outputs to the provided [Write].
    pub fn output_csv(totals: &Warlocs, files: &HashMap<PathBuf, Warlocs>, writer: impl Write) {
        let mut csvw = csv::Writer::from_writer(writer);

        write_csv_header_row(&mut csvw);

        write_single_csv_value(PathBuf::from("Totals"), totals, &mut csvw);

        for (path, stats) in files.iter() {
            write_single_csv_value(path, stats, &mut csvw);
        }
    }

    /// Writes out the olumn row for the CSV output of [Warlocs].
    pub fn write_csv_header_row<T: Write>(writer: &mut CsvWriter<T>) {
        let columns = [
            "File Path",
            "Main Code",
            "Main Docs",
            "Main Comments",
            "Main Spaces",
            "Test Code",
            "Test Docs",
            "Test Comments",
            "Test Spaces",
            "Example Code",
            "Example Docs",
            "Example Comments",
            "Example Spaces",
        ];
        writer.write_record(columns).expect("Write CSV columns");
    }

    pub fn write_single_csv_value<T: Write>(
        path: impl AsRef<Path>,
        stats: &Warlocs,
        writer: &mut CsvWriter<T>,
    ) {
        let s = stats;
        let mut row: Vec<String> = vec![path.as_ref().to_str().unwrap().to_string()];
        row.extend(
            [
                s.main.code,
                s.main.docs,
                s.main.comments,
                s.main.whitespaces,
                s.tests.code,
                s.tests.docs,
                s.tests.comments,
                s.tests.whitespaces,
                s.examples.code,
                s.examples.docs,
                s.examples.comments,
                s.examples.whitespaces,
            ]
            .map(|v| v.to_string()),
        );

        writer.write_record(row).expect("Write CSV row");
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        cli::OutputFormat,
        output::{output_multiple_file_stats, output_total_stats},
        warlocs::Warlocs,
    };

    macro_rules! all_output_formats {
        () => {
            [
                OutputFormat::Csv,
                OutputFormat::Json,
                OutputFormat::Tabular,
                OutputFormat::Yaml,
            ]
        };
    }

    #[test]
    fn test_output_single_no_panics() {
        let val = Warlocs::default();

        for output_format in all_output_formats!() {
            output_total_stats(PathBuf::from("."), &val, &output_format);
        }
    }

    #[test]
    fn test_output_multiple_no_panics() {
        for output_format in all_output_formats!() {
            let files = [
                (PathBuf::from("abc"), Warlocs::default()),
                (PathBuf::from("123"), Warlocs::default()),
            ]
            .into_iter()
            .collect();
            output_multiple_file_stats(&output_format, files);
        }
    }
}
