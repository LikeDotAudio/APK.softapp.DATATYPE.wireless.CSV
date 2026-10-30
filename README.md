# DATATYPE.wireless.CSV

> 📑 **Wireless CSV importer** — reads a wireless-coordination CSV, recognises which of three
> layouts it is, turns it into channel records and publishes them on the bus.

---

## 🧩 Architectural Role
* **Family**: DATATYPE — an importer for somebody else's file format
* **Category**: Wireless / RF coordination reports — source `4_CSV` of the four DataMigration importers (IAS, Soundbase, Wireless Workbench, CSV)
* **Pipeline Stage**: Data Migration — the record lands on `APK.audio/System/DataMigration/CSV/ImportedData`, the topic the `Gui_Frames/3_Data_Migration/4_CSV` frame shows

---

## ⚙️ Repository Overview

| Path | What it is |
|---|---|
| `csv_importer.rs` | The importer. Compiled as module `csv_importer` of the `apkaudio-data-migration` crate. |
| `test_csv_pattern_importer.rs` | A `[[bin]]` runner of that crate: parses six fixtures, prints a summary, exits 1 if any fails. |
| `test_data/` | Fixtures: `report.csv`, `report_converted.csv`, `venue.csv`, `venue_data.csv`, `venue_converted.csv`, `SB PDF.csv`, and a Lectrosonics programming guide PDF. |
| `IMPORT:file:CSV/` | The Python edge (`oaFileImportCSV`) from the desktop app this was lifted out of — see below. |

### `csv_importer.rs`

`parse_and_classify_csv(path)` / `parse_and_classify_csv_str(text, source_name)` read the header
row, upper-case it and pick a pattern:

| Pattern | Recognised by | Columns read |
|---|---|---|
| `SoundbaseReport` — "Soundbase CSV Export" | headers contain `SPACING` and `FREQUENCY` | Group, Model, Band, Name, Preset, Spacing, Frequency (Group fills both zone and group) |
| `StandardZoneGroup` — "Standard Zone/Group CSV" | headers contain `ZONE` and `FREQ` | ZONE, GROUP, DEVICE, NAME, FREQ; rows outside 100–3000 MHz are dropped |
| `UnknownCustom` — "Arbitrary Custom CSV" | anything else | the first cell per row that parses as 100–3000 MHz |

Each row becomes a `StandardRfRecord { zone, group, device, channel_name, frequency_mhz, band }`.
`band` is filled only by the Soundbase layout; the others leave it empty. Empty zone/group/device/
name cells in the zone/group layout are filled with `N/A`, `General`, `Generic RF` and `CH_<n>`;
the custom layout fills all four with fixed labels.

**Every parse publishes.** The whole `CsvRecognizedResult` (status, source file, pattern, topic,
count, records) goes out as pretty JSON, QoS 1, **retained**, to
`APK.audio/System/DataMigration/CSV/ImportedData` (built by
`apkaudio_contracts::topics::data_migration("CSV", "ImportedData")`) on the loopback broker
`MQTT_LOOPBACK_HOST:MQTT_DEFAULT_PORT`, with credentials from
`apkaudio_contracts::broker_auth` when set. It waits up to 3 s for the PUBACK; no broker is not
an error.

Who calls it: `PODS/POD.THICK/NODE.BareMetal/SRC/Core/ORCHESTRATOR:hunter:VISA/src/services/data_migration.rs`
subscribes to `APK.audio/System/DataMigration/+/FilePath` and hands a CSV payload to
`oaRustCore::data_migration::csv_importer`.

### Build and test

The crate is `Softapps/MANAGEMENT_AND_UTILITIES/MANAGEMENT.DataMigration` — `mod.rs` pulls this
file in by `#[path]`, and `Cargo.toml` declares the runner as a `[[bin]]`:

```bash
cd PODS/POD.THIN/Softapps/MANAGEMENT_AND_UTILITIES/MANAGEMENT.DataMigration
cargo build --bin test_csv_pattern_importer
cargo run   --bin test_csv_pattern_importer      # publishes to the local broker, retained
```

Checked 2026-10-30: it builds, and the runner **fails all six files** — it looks for its fixtures
at `<MANAGEMENT.DataMigration>/DataMigration/4_CSV/test_data/`, which no longer exists; they are
here in `test_data/`. Pointed at `test_data/` directly, the importer reads `report.csv` 168
records, `venue*.csv` 253 each, `SB PDF.csv` 20 (Soundbase), `report_converted.csv` 0.

### `IMPORT:file:CSV/` (Python)

`Entry.py` and `FileReaders/from_csv_unknown.py` expose `convert_unknown_csv_report()`, which
calls `oaRustCore.csv_parser.convert_csv_unknown` and raises if it is not importable — there is
no Python CSV parser, on purpose. `Methods/csv_parser.py` imports it unguarded. Logging comes from
`importer_logging` (in `PODS/POD.THICK/NODE.BareMetal/SRC/Core/`), which falls back to a no-op.
`Methods/oaCSVParser_rs/` holds only a maturin `pyproject.toml` and Python shims, no Rust source;
the parser it names is `NODE.BareMetal/SRC/Core/CORE:engine:RUST/PARSER/csv_parser.rs`.

```bash
cd "IMPORT:file:CSV"
PYTHONPATH=<repo>/PODS/POD.THICK/NODE.BareMetal/SRC/Core python3 -m pytest -q Tests --ignore=Tests/test_rust_csv_parser.py
```

Checked 2026-10-30: 3 passed. `Tests/test_rust_csv_parser.py` does not collect here — it imports
`oaFileImportCSV…`, a package name only the original desktop layout provides — and
`Methods/oaCSVParser_rs/python/tests` skips without `oaRustCore`.

---

## 🚀 Usage & Integration
* **Web Portal Entry**: none of its own. The records are drawn by the Data Migration frame
  `PODS/POD.THIN/WEB.Frontend-Assets/SRC/FrontEnd/Gui_Frames/3_Data_Migration/4_CSV/CSV_Importer.json`.
  `display.json` declares the folder to the launcher; its `entry`, `main.py`, is not a file here,
  so the launcher opens the folder itself.
* **Remote Repository**: `https://github.com/LikeDotAudio/APK.softapp.DATATYPE.wireless.CSV.git`

---

## 🏛 Philosophy

- **The bus is the API.** The importer's output is a retained message on a topic built from the
  contract crate, not a return value another app links to (`csv_importer.rs`,
  `csv_importer_mqtt_topic()` and `publish_to_mqtt()`).
- **Topics come from the contract, never typed out** (`apkaudio_contracts::topics::data_migration`,
  comment above `csv_importer_mqtt_topic()`).
- **Do not invent a band.** Only the Soundbase layout has a band column; for the others `band` is
  left empty "rather than invented" (`StandardRfRecord`). The same file does fill empty
  zone/group/device/name cells with placeholder labels.
- **One parser, no silent fallback.** The Python edge refuses to run without the Rust parser
  rather than quietly producing different records (`IMPORT:file:CSV/Entry.py`,
  `Methods/csv_parser.py` headers).
- **A failing fixture fails the run.** The runner counts failures and exits 1
  (`test_csv_pattern_importer.rs`); it used to print and exit 0.
- **Keep going on a bad row.** Unreadable records and cells that are not a frequency in range
  are skipped, not fatal (`rdr.records().flatten()`, the 100–3000 MHz guard).
