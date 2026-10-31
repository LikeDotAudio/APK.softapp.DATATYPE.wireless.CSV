// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! Every fixture in this repository's `test_data/` with the record count it
//! must import. A count that drifts is a failure, not a line to read past.
//!
//! This was the `test_csv_pattern_importer` `[[bin]]` that MANAGEMENT.DataMigration
//! built from this repository's file, with fixture paths spelled from
//! DataMigration's manifest. It is a `cargo test` here now, resolved from this
//! crate's own manifest (2026-10-30 audit).
//!
//! `report_converted.csv` is a negative fixture: the output of a retired
//! PDF-report converter that found no frequency on any line and wrote "No Match"
//! into every FREQ cell. The importer must classify it as Standard Zone/Group and
//! import nothing, all 179 rows skipped. `report.csv` is the same coordination
//! report converted correctly: 168 records.

use std::path::PathBuf;

use apkaudio_wireless_csv::parse_and_classify_csv;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test_data").join(name)
}

#[test]
fn every_fixture_imports_its_known_count() {
    let expected: [(&str, usize, &str); 6] = [
        ("report_converted.csv", 0, "empty"),
        ("report.csv", 168, "success"),
        ("SB PDF.csv", 20, "success"),
        ("venue_converted.csv", 253, "success"),
        ("venue_data.csv", 253, "success"),
        ("venue.csv", 253, "success"),
    ];
    for (name, count, status) in expected {
        let res = parse_and_classify_csv(fixture(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(res.total_records, count, "{name}: record count");
        assert_eq!(res.status, status, "{name}: status");
    }
}

#[test]
fn the_negative_fixture_skips_every_row() {
    let res = parse_and_classify_csv(fixture("report_converted.csv")).unwrap();
    assert_eq!(res.pattern_name, "Standard Zone/Group CSV");
    assert_eq!(res.skipped_rows, 179);
}
