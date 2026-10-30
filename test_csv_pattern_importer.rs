// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! Standalone Test Runner for CSV Pattern Recognition. It parses the fixtures and
//! publishes nothing: the importer has no broker of its own, so a run cannot
//! overwrite the live retained `DataMigration/CSV/ImportedData`.

use std::path::Path;

mod csv_importer;

fn main() {
    let test_files = vec![
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../DATATYPE/DATATYPE.wireless.CSV/test_data/report_converted.csv"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../DATATYPE/DATATYPE.wireless.CSV/test_data/report.csv"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../DATATYPE/DATATYPE.wireless.CSV/test_data/SB PDF.csv"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../DATATYPE/DATATYPE.wireless.CSV/test_data/venue_converted.csv"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../DATATYPE/DATATYPE.wireless.CSV/test_data/venue_data.csv"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../DATATYPE/DATATYPE.wireless.CSV/test_data/venue.csv"),
    ];

    println!("=== SMART CSV PATTERN RECOGNITION TEST RUNNER ===");

    // One file failing must fail the run. This loop used to print to stderr and
    // exit 0, so `check.sh` could have run it and never seen a broken importer.
    let mut failures = 0;

    for file_path in test_files {
        let path = Path::new(file_path);
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        println!("\nTesting File: '{}'", filename);

        match csv_importer::parse_and_classify_csv(path) {
            Ok(res) => {
                println!("  └─ Recognized Pattern: [{}]", res.pattern_name);
                println!("  └─ Total Records Extracted: {}", res.total_records);
                println!("  └─ Status: {} ({} row(s) skipped)", res.status, res.skipped_rows);

                if let Some(sample) = res.records.first() {
                    println!("  └─ Sample Record: Zone='{}' | Group='{}' | Device='{}' | Name='{}' | Freq={:.3} MHz | Band='{}'",
                        sample.zone, sample.group, sample.device, sample.channel_name,
                        sample.frequency_mhz, sample.band);
                }
            }
            Err(e) => {
                eprintln!("  └─ ERROR: {}", e);
                failures += 1;
            }
        }
    }
    println!("\n=================================================");
    if failures > 0 {
        eprintln!("{} file(s) failed to parse.", failures);
        std::process::exit(1);
    }
}
