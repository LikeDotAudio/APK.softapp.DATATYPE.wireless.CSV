// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! Smart CSV Pattern Classifier & Importer written in Rust.
//!
//! It parses; it does not publish. The worker (`SRC/`) answers a job with the
//! result (the orchestrator's DataMigration inbox that also published it is
//! retired: LOOSE-datamigration-frames-to-workers). Nothing in this file opens a broker.

use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use apkaudio_contracts::topics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecognizedCsvPattern {
    SoundbaseReport,    // Headers: Group, Model, Band, Name, Preset, Spacing, Frequency
    StandardZoneGroup,  // Headers: ZONE, GROUP, DEVICE, NAME, FREQ
    UnknownCustom,      // Arbitrary / Custom CSV
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardRfRecord {
    pub zone: String,
    pub group: String,
    pub device: String,
    pub channel_name: String,
    pub frequency_mhz: f64,
    /// RF band as the vendor spells it — `G57 - Narrowband` and the like.
    /// Only the Soundbase export carries one; the zone/group patterns have no
    /// band column, so this is empty for them rather than invented.
    pub band: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CsvRecognizedResult {
    pub status: String,
    pub source_file: String,
    pub pattern: RecognizedCsvPattern,
    pub pattern_name: String,
    pub mqtt_topic: String,
    pub total_records: usize,
    /// Data rows that yielded no record: unreadable, too short, or no
    /// frequency in range. A blank line is not counted.
    pub skipped_rows: usize,
    pub records: Vec<StandardRfRecord>,
}

/// `APK.audio/System/DataMigration/CSV/ImportedData`, built from the contract
/// rather than typed out — see `contracts/rust/src/topics.rs`, whose own header
/// names the DataMigration importers as the literals it exists to replace.
pub fn csv_importer_mqtt_topic() -> String {
    topics::data_migration("CSV", "ImportedData")
        .expect("CSV/ImportedData are literal segments the contract accepts")
}

pub fn parse_and_classify_csv(file_path: impl AsRef<Path>) -> Result<CsvRecognizedResult, String> {
    let path = file_path.as_ref();
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read CSV file {:?}: {}", path, e))?;
    classify_csv_str(&content, &path.to_string_lossy())
}

/// Classify the layout and extract standard RF records. NO SIDE EFFECT.
///
/// The one parser: oaRustCore's CSV parser, the worker (`SRC/`,
/// PLAN-3423.01) and the test runner all run this function. Publishing is the
/// caller's business (LOOSE-datatype-wireless-csv-held-by-session: it used to
/// publish retained to the loopback broker itself, so a test run overwrote
/// the live `ImportedData`).
pub fn classify_csv_str(content: &str, source_name: &str) -> Result<CsvRecognizedResult, String> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(content.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| format!("Failed to parse headers: {}", e))?
        .clone();

    let header_vec: Vec<String> = headers.iter().map(|s| s.trim().to_uppercase()).collect();

    // Pattern Recognition Logic
    let (pattern, pattern_name) = if header_vec.contains(&"SPACING".into()) && header_vec.contains(&"FREQUENCY".into()) {
        (RecognizedCsvPattern::SoundbaseReport, "Soundbase CSV Export")
    } else if header_vec.contains(&"ZONE".into()) && header_vec.contains(&"FREQ".into()) {
        (RecognizedCsvPattern::StandardZoneGroup, "Standard Zone/Group CSV")
    } else {
        (RecognizedCsvPattern::UnknownCustom, "Arbitrary Custom CSV")
    };

    let mut records = Vec::new();
    let mut skipped_rows = 0usize;
    // Every data row ends up a record or a skipped row; a blank line is neither.
    let rows = rdr.records().filter_map(|r| match r {
        Ok(rec) if rec.iter().all(|f| f.trim().is_empty()) => None,
        other => Some(other.ok()),
    });

    match pattern {
        RecognizedCsvPattern::SoundbaseReport => {
            for rec in rows {
                let Some(rec) = rec else { skipped_rows += 1; continue };
                let freq = if rec.len() >= 7 { rec[6].trim().parse::<f64>().ok() } else { None };
                let Some(freq) = freq else { skipped_rows += 1; continue };
                // Header: Group, Model, Band, Name, Preset, Spacing, Frequency.
                // There is no zone column, so Group answers for both.
                records.push(StandardRfRecord {
                    zone: rec[0].trim().to_string(),
                    group: rec[0].trim().to_string(),
                    device: rec[1].trim().to_string(),
                    channel_name: rec[3].trim().to_string(),
                    frequency_mhz: freq,
                    band: rec[2].trim().to_string(),
                });
            }
        }
        RecognizedCsvPattern::StandardZoneGroup => {
            for rec in rows {
                let Some(rec) = rec else { skipped_rows += 1; continue };
                let before = records.len();
                if rec.len() >= 5 {
                    let zone = rec[0].trim().to_string();
                    let group = rec[1].trim().to_string();
                    let device = rec[2].trim().to_string();
                    let name = rec[3].trim().to_string();
                    let freq_str = rec[4].trim();

                    if let Ok(freq) = freq_str.parse::<f64>() {
                        if freq > 100.0 && freq < 3000.0 {
                            // An empty cell stays empty: "N/A", "General", "Generic RF" and
                            // "CH_<n>" read downstream as things the file said.
                            records.push(StandardRfRecord {
                                zone,
                                group,
                                device,
                                channel_name: name,
                                frequency_mhz: freq,
                                band: String::new(),
                            });
                        }
                    }
                }
                if records.len() == before {
                    skipped_rows += 1;
                }
            }
        }
        RecognizedCsvPattern::UnknownCustom => {
            for rec in rows {
                let Some(rec) = rec else { skipped_rows += 1; continue };
                let before = records.len();
                // Heuristic scan for frequency floats in cells
                for cell in rec.iter() {
                    if let Ok(freq) = cell.trim().parse::<f64>() {
                        if freq > 100.0 && freq < 3000.0 {
                            // Only the frequency was found; the rest is unknown, and left
                            // empty rather than labelled ("Unknown Zone", "CH_<n>").
                            // `pattern_name` already says this was an arbitrary CSV.
                            records.push(StandardRfRecord {
                                zone: String::new(),
                                group: String::new(),
                                device: String::new(),
                                channel_name: String::new(),
                                frequency_mhz: freq,
                                band: String::new(),
                            });
                            break;
                        }
                    }
                }
                if records.len() == before {
                    skipped_rows += 1;
                }
            }
        }
    }

    Ok(CsvRecognizedResult {
        status: status_for(records.len()),
        source_file: source_name.to_string(),
        pattern,
        pattern_name: pattern_name.to_string(),
        mqtt_topic: csv_importer_mqtt_topic(),
        total_records: records.len(),
        skipped_rows,
        records,
    })
}

/// "success" only when the file yielded records; a parse that found none is
/// "empty", never a success the panel would draw as an imported, blank table
/// (the rule `soundbase_importer.rs` already keeps).
fn status_for(records: usize) -> String {
    if records > 0 { "success" } else { "empty" }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_with_no_records_is_empty_and_counts_its_dropped_rows() {
        let r = classify_csv_str("ZONE,GROUP,DEVICE,NAME,FREQ\nA,G,D,N,50\n\nA,G,D,N,x\n", "t.csv").unwrap();
        assert_eq!((r.status.as_str(), r.total_records, r.skipped_rows), ("empty", 0, 2));
    }

    #[test]
    fn a_good_row_is_a_record_and_a_bad_one_is_skipped() {
        let r = classify_csv_str("ZONE,GROUP,DEVICE,NAME,FREQ\nA,G,D,N,566.6\nshort\n", "t.csv").unwrap();
        assert_eq!((r.status.as_str(), r.total_records, r.skipped_rows), ("success", 1, 1));
    }
}
