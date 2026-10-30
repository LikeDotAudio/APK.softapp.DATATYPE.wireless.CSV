// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! Smart CSV Pattern Classifier & Importer written in Rust with MQTT Publishing.

use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use apkaudio_contracts::topics::{self, MQTT_DEFAULT_PORT, MQTT_LOOPBACK_HOST};

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
    parse_and_classify_csv_str(&content, &path.to_string_lossy())
}

/// Classify and parse, then publish the result retained on
/// `DataMigration/CSV/ImportedData` — what the orchestrator's file-path import
/// and the test runner call.
pub fn parse_and_classify_csv_str(content: &str, source_name: &str) -> Result<CsvRecognizedResult, String> {
    let result = classify_csv_str(content, source_name)?;
    if let Ok(json_payload) = serde_json::to_string_pretty(&result) {
        publish_to_mqtt(&csv_importer_mqtt_topic(), &json_payload);
    }
    Ok(result)
}

/// Classify the layout and extract standard RF records. NO SIDE EFFECT.
///
/// Split out of `parse_and_classify_csv_str` for the worker (`SRC/`,
/// PLAN-3423.01), which answers a job on `incoming/result` and must not also
/// overwrite the retained `ImportedData` document every time a browser asks
/// for a parse. One parser either way: both entry points run this function.
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

    match pattern {
        RecognizedCsvPattern::SoundbaseReport => {
            for rec in rdr.records().flatten() {
                if rec.len() >= 7 {
                    if let Ok(freq) = rec[6].trim().parse::<f64>() {
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
            }
        }
        RecognizedCsvPattern::StandardZoneGroup => {
            for rec in rdr.records().flatten() {
                if rec.len() >= 5 {
                    let zone = rec[0].trim().to_string();
                    let group = rec[1].trim().to_string();
                    let device = rec[2].trim().to_string();
                    let name = rec[3].trim().to_string();
                    let freq_str = rec[4].trim();

                    if let Ok(freq) = freq_str.parse::<f64>() {
                        if freq > 100.0 && freq < 3000.0 {
                            records.push(StandardRfRecord {
                                zone: if zone.is_empty() { "N/A".into() } else { zone },
                                group: if group.is_empty() { "General".into() } else { group },
                                device: if device.is_empty() { "Generic RF".into() } else { device },
                                channel_name: if name.is_empty() { format!("CH_{}", records.len() + 1) } else { name },
                                frequency_mhz: freq,
                                band: String::new(),
                            });
                        }
                    }
                }
            }
        }
        RecognizedCsvPattern::UnknownCustom => {
            for rec in rdr.records().flatten() {
                // Heuristic scan for frequency floats in cells
                for cell in rec.iter() {
                    if let Ok(freq) = cell.trim().parse::<f64>() {
                        if freq > 100.0 && freq < 3000.0 {
                            records.push(StandardRfRecord {
                                zone: "Unknown Zone".into(),
                                group: "Custom CSV".into(),
                                device: "Generic".into(),
                                channel_name: format!("CH_{}", records.len() + 1),
                                frequency_mhz: freq,
                                band: String::new(),
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(CsvRecognizedResult {
        status: "success".to_string(),
        source_file: source_name.to_string(),
        pattern,
        pattern_name: pattern_name.to_string(),
        mqtt_topic: csv_importer_mqtt_topic(),
        total_records: records.len(),
        records,
    })
}

pub fn publish_to_mqtt(topic: &str, payload: &str) {
    use rumqttc::{Client, MqttOptions, Event, Packet, QoS};
    use std::time::Duration;

    let mut mqttoptions = MqttOptions::new("apkaudio-csv-importer", MQTT_LOOPBACK_HOST, MQTT_DEFAULT_PORT);
    mqttoptions.set_keep_alive(Duration::from_secs(10));
    if let Some((username, password)) = apkaudio_contracts::broker_auth::credentials() {
        mqttoptions.set_credentials(username, password);
    }

    let (client, mut connection) = Client::new(mqttoptions, 10);
    let _ = client.publish(topic, QoS::AtLeastOnce, true, payload.as_bytes());

    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    let mut acked = false;
    for notification in connection.iter() {
        if let Ok(Event::Incoming(Packet::PubAck(_))) = notification {
            println!("[SMART CSV MQTT PUBLISHER] Published & Acked by Broker ({MQTT_LOOPBACK_HOST}:{MQTT_DEFAULT_PORT}) Topic: '{}'", topic);
            acked = true;
            break;
        }
        if std::time::Instant::now() > deadline {
            break;
        }
    }
    if acked {
        std::thread::sleep(Duration::from_millis(200));
    }
}
