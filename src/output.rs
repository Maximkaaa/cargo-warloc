//! Output routines.

use std::{collections::BTreeMap, path::PathBuf};

use csv::Writer as CsvWriter;
use serde::Serialize;

use crate::{cli::OutputFormat, warlocs::Warlocs};

/// Prints to stdout multiple stats formatted based on the given `output_format`.
pub fn output_multiple_file_stats(output_format: &OutputFormat, stats: BTreeMap<PathBuf, Warlocs>) {
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
            print!(
                "{}",
                serde_yaml_bw::to_string(&multistats).expect("YAML serialization should work")
            )
        }
        OutputFormat::Csv => {
            let multistats = SerializableMultiFileStats::from_file_stats(stats);
            let mut buff: Vec<u8> = Vec::new();
            csv_output::output_csv(&multistats.totals, &multistats.files, &mut buff);
            println!(
                "{}",
                str::from_utf8(&buff).expect("CSV output should be UTF-8")
            )
        }
    }
}

/// Prints to stdout the provided total [Warloc] stats formatted to the given `output_format`.
pub fn output_total_stats(file_count: u64, stats: &Warlocs, output_format: &OutputFormat) {
    match output_format {
        OutputFormat::Tabular => {
            println!("Total file count: {file_count}",);
            single_stat_tabular(stats)
        }
        OutputFormat::Json => {
            let stats = SerializableTotalStats { file_count, stats };
            println!(
                "{}",
                serde_json::to_string(&stats).expect("JSON serialization should work")
            )
        }
        OutputFormat::Yaml => {
            let stats = SerializableTotalStats { file_count, stats };
            print!(
                "{}",
                serde_yaml_bw::to_string(&stats).expect("YAML serialization should work")
            )
        }
        OutputFormat::Csv => {
            let mut buff: Vec<u8> = Vec::new();
            let mut writer = CsvWriter::from_writer(&mut buff);
            writer
                .write_field("File Count")
                .expect("Write CSV first row");
            csv_output::write_csv_header_row(&mut writer);
            writer
                .write_field(file_count.to_string())
                .expect("Write CSV row field");
            csv_output::write_single_csv_value(stats, &mut writer);
            drop(writer);
            println!(
                "{}",
                str::from_utf8(&buff).expect("CSV output should be UTF-8")
            )
        }
    }
}

/// Prints to stdout a tabular representation for a single [Warlocs] values.
fn single_stat_tabular(stats: &Warlocs) {
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

#[derive(Serialize)]
struct SerializableTotalStats<'a> {
    file_count: u64,
    #[serde(flatten)]
    stats: &'a Warlocs,
}

/// Simple wrapper struct representing the structure of serialized multi-file output.
#[derive(Serialize)]
struct SerializableMultiFileStats {
    file_count: u64,
    totals: Warlocs,
    files: BTreeMap<PathBuf, Warlocs>,
}

impl SerializableMultiFileStats {
    fn from_file_stats(files: BTreeMap<PathBuf, Warlocs>) -> Self {
        SerializableMultiFileStats {
            file_count: files.len() as u64,
            totals: files.values().fold(Warlocs::default(), |acc, v| acc + *v),
            files,
        }
    }
}

/// Module defining CSV output routines.
mod csv_output {
    use std::{collections::BTreeMap, io::Write, path::PathBuf};

    use csv::Writer as CsvWriter;

    use crate::warlocs::Warlocs;

    /// Performs CSV serialization and outputs to the provided [Write].
    pub fn output_csv(totals: &Warlocs, files: &BTreeMap<PathBuf, Warlocs>, writer: impl Write) {
        let mut csvw = csv::Writer::from_writer(writer);

        csvw.write_field("File").expect("Write CSV columns row");
        write_csv_header_row(&mut csvw);

        for (path, stats) in files.iter() {
            csvw.write_field(path.to_str().expect("Filepaths should be UTF-8"))
                .expect("Write CSV stats value");
            write_single_csv_value(stats, &mut csvw);
        }

        let file_count = files.len();
        csvw.write_field(format!("{file_count} files total"))
            .expect("Write CSV totals row");
        write_single_csv_value(totals, &mut csvw);
    }

    /// Writes out the column titles row for the CSV output of [Warlocs].
    pub fn write_csv_header_row<T: Write>(writer: &mut CsvWriter<T>) {
        let columns = [
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

    pub fn write_single_csv_value<T: Write>(stats: &Warlocs, writer: &mut CsvWriter<T>) {
        let s = stats;

        let row: Vec<String> = [
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
        .iter()
        .map(|v| v.to_string())
        .collect();

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
            output_total_stats(1, &val, &output_format);
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
