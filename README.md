# DATATYPE.wireless.CSV

> 📑 **Wireless CSV importer** — reads a wireless-coordination CSV, recognises which of three
> layouts it is and turns it into channel records, which the node publishes on the bus. It also runs as
> the worker `Worker-WIRELESS_CSV` in `PODS/POD.workers`: a browser posts a CSV as a job on the
> bus and gets the records back.
>
> 📚 **Wiki:** [Documentation/README.md](Documentation/README.md) — principles, architecture, delivery.

## Why it exists

A coordination report (Soundbase, a zone/group sheet, or any CSV with frequencies in it) has to
become channel records the bench can show and plan with. This repository is the one place that
conversion lives. It runs two ways: compiled into the DataMigration crate, where a file path on
the bench is imported and the records are published retained; and as a worker, where any
client sends the CSV itself as a job and receives the records as the answer. Both run the same
function, `classify_csv_str` in `csv_importer.rs`.

---

## 🧩 Architectural Role
* **Family**: DATATYPE — an importer for somebody else's file format
* **Category**: Wireless / RF coordination reports — source `4_CSV` of the four DataMigration importers (IAS, Soundbase, Wireless Workbench, CSV)
* **Pipeline Stage**: Data Migration — the `Gui_Frames/3_Data_Migration/4_CSV` frame posts the file's text as a job to `wireless_csv` and shows the answer from `…/wireless_csv/incoming/result`; the orchestrator's `FilePath` daemon still lands records on `APK.audio/System/DataMigration/CSV/ImportedData` for other publishers

---

## ⚙️ Repository Overview

| Path | What it is |
|---|---|
| `csv_importer.rs` | The importer. Compiled as module `csv_importer` of the `apkaudio-data-migration` crate. |
| `test_csv_pattern_importer.rs` | A `[[bin]]` runner of that crate: parses six fixtures, prints a summary, exits 1 if any fails. |
| `test_data/` | Fixtures: `report.csv`, `report_converted.csv`, `venue.csv`, `venue_data.csv`, `venue_converted.csv`, `SB PDF.csv`, and a Lectrosonics programming guide PDF. |
| `IMPORT:file:CSV/` | The Python edge (`oaFileImportCSV`) from the desktop app this was lifted out of — see below. |
| `SRC/` | The worker crate `apk-worker-wireless-csv` (PLAN-3423.01): `#[path]`s `csv_importer.rs` in and answers jobs on the bus. |
| `Docker/` | The worker's `Dockerfile` (repo-root context) and `docker-compose.yml` (project `workers`, container `Worker-WIRELESS_CSV`), `include:`d by `PODS/POD.workers/Docker/docker-compose.yml`. |

### `csv_importer.rs`

`classify_csv_str(text, source_name)` reads the header row, upper-cases it and picks a pattern.
It has no side effect; `parse_and_classify_csv(path)` reads a file and calls it. Nothing in
`csv_importer.rs` publishes — the caller does (below):

| Pattern | Recognised by | Columns read |
|---|---|---|
| `SoundbaseReport` — "Soundbase CSV Export" | headers contain `SPACING` and `FREQUENCY` | Group, Model, Band, Name, Preset, Spacing, Frequency (Group fills both zone and group) |
| `StandardZoneGroup` — "Standard Zone/Group CSV" | headers contain `ZONE` and `FREQ` | ZONE, GROUP, DEVICE, NAME, FREQ; rows outside 100–3000 MHz are dropped |
| `UnknownCustom` — "Arbitrary Custom CSV" | anything else | the first cell per row that parses as 100–3000 MHz |

Each row becomes a `StandardRfRecord { zone, group, device, channel_name, frequency_mhz, band }`.
`band` is filled only by the Soundbase layout; the others leave it empty. An empty
zone/group/device/name cell stays empty, and the custom layout, which finds only a frequency,
leaves all four empty — `pattern_name` already says what kind of file it was.

`status` is `"success"` only when the file yielded records, `"empty"` otherwise (the rule
`soundbase_importer.rs` keeps). `skipped_rows` counts the data rows that yielded no record —
unreadable, too short, or no frequency in 100–3000 MHz; a blank line is not counted.

**The importer opens no broker.** `PODS/POD.THICK/NODE.BareMetal/SRC/Core/ORCHESTRATOR:hunter:VISA/src/services/data_migration.rs`
subscribes to `APK.audio/System/DataMigration/+/FilePath`, runs `classify_csv_str` on the CSV
payload and publishes the whole `CsvRecognizedResult` (status, source file, pattern, topic,
counts, records) as pretty JSON, QoS 1, **retained**, on its own session to
`APK.audio/System/DataMigration/CSV/ImportedData` (built by
`apkaudio_contracts::topics::data_migration("CSV", "ImportedData")`). Until 2026-10-30 the
importer also carried a publisher of its own (`parse_and_classify_csv_str`, `publish_to_mqtt`)
to the loopback broker, which only the test runner still reached — so a test run overwrote the
live retained document.

### The worker (`SRC/`, `Docker/`)

```text
client ── APK.audio/System/Protocols/wireless_csv/outgoing/job ──▶ Worker-WIRELESS_CSV
          {"job_id": "…", "filename": "venue.csv", "text": "<csv>"}   ("content_b64" for raw bytes)
client ◀─ APK.audio/System/Protocols/wireless_csv/incoming/result ── Worker-WIRELESS_CSV
          {"job_id": "…", "tool": "DATATYPE.wireless.CSV", "ok": true, "result": {…}, "error": null, "elapsed_ms": 0}
```

- `SRC/src/lib.rs` `handle_job` is the tool: it decodes the job, refuses one over `max_job_bytes`
  (`SRC/config.ini`, 8 MiB), runs `csv_importer::classify_csv_str`, and always returns a result
  document. A malformed job gets `ok: false` and an `error`, with its `job_id` when it had one.
- `SRC/src/lib.rs` `run_job_agent` is the loop: its own MQTT session, subscribed to
  `outgoing/job`, publishing each answer to `incoming/result`. Neither leaf is retained
  (`contracts/rust/src/baremetal_io.rs`, `module_job` / `module_result`). It calls
  `apk_plugin_runner::tick()` on every broker event so the health file stays fresh while idle.
- `SRC/src/bin/apk-worker-wireless-csv.rs` hands both to `apk_plugin_runner::main!`. The runner
  (`PODS/POD.THICK/DISCOVERY.Engine/SRC/RUNNER`) publishes the config, the last will, the
  self-test door and the retained self-description at `wireless_csv/incoming/api`.
- `SRC/src/self_test.rs` runs a five-row CSV through `handle_job` and expects 4 records.
- The image (`Docker/Dockerfile`) builds from the repo root, because the crate links
  `apkaudio-contracts` and `apk-plugin-runner`. The runtime is `debian:bookworm-slim`, user
  `worker` (uid 10001). Compose runs it `read_only`, `cap_drop: [ALL]`, no published port, on
  the `workers` network plus `apk-audio_default` to reach the broker as `broker:1883`.
  The healthcheck is the runner's `apk-plugin-health`, compiled from the runner's source file.

```bash
cd SRC && cargo test                                                          # handle_job on the fixtures
docker compose -f PODS/POD.workers/Docker/docker-compose.yml up -d --build    # from the repo root
mosquitto_pub -h 127.0.0.1 -t APK.audio/System/Protocols/wireless_csv/outgoing/job \
  -m '{"job_id":"t1","filename":"x.csv","text":"ZONE,GROUP,DEVICE,NAME,FREQ\nA,G,D,N,566.6\n"}'
```

### Build and test

The crate is `Softapps/MANAGEMENT_AND_UTILITIES/MANAGEMENT.DataMigration` — `mod.rs` pulls this
file in by `#[path]`, and `Cargo.toml` declares the runner as a `[[bin]]`:

```bash
cd PODS/POD.THIN/Softapps/MANAGEMENT_AND_UTILITIES/MANAGEMENT.DataMigration
cargo build --bin test_csv_pattern_importer
cargo run   --bin test_csv_pattern_importer      # parses the fixtures; publishes nothing
```

The runner reads its six fixtures from `../../DATATYPE/DATATYPE.wireless.CSV/test_data/` (until
2026-10-30 it looked under `<MANAGEMENT.DataMigration>/DataMigration/4_CSV/`, which no longer
exists, and failed all six). Each fixture carries the record count it must import, and a
different count fails the run: `report.csv` 168, `venue*.csv` 253 each, `SB PDF.csv` 20
(Soundbase), `report_converted.csv` 0 (`status: "empty"`, 179 rows skipped).

`report_converted.csv` is a negative fixture, not a broken one. It is the output of a retired
PDF-report converter (not the current `convert_csv_unknown`, which never writes "No Match") that
found no frequency on any line and wrote `No Match` into every FREQ cell; its source text is not
kept. The importer is right to drop every row: `No Match` is not a 100–3000 MHz frequency. The
same coordination report, converted correctly, is `report.csv`.

### `IMPORT:file:CSV/` (Python)

`Entry.py` and `FileReaders/from_csv_unknown.py` expose `convert_unknown_csv_report()`, which
calls `oaRustCore.csv_parser.convert_csv_unknown` and raises `RuntimeError` if it is not
importable — there is no Python CSV parser, on purpose. `Methods/csv_parser.py` imports it
unguarded. Logging comes from `importer_logging` (in `PODS/POD.THICK/NODE.BareMetal/SRC/Core/`),
which falls back to a no-op. The Rust parser is `NODE.BareMetal/SRC/Core/CORE:engine:RUST/PARSER/csv_parser.rs`,
built into the `oaRustCore` wheel; it exports that one function.

```bash
cd "IMPORT:file:CSV" && python3 -m pytest -q Tests        # Tests/conftest.py finds importer_logging
```

The folder name is not a Python identifier, so `Tests/conftest.py` puts the package root on
`sys.path` and the tests import `FileReaders.from_csv_unknown` directly. Checked 2026-10-30:
without `oaRustCore`, 2 passed and 3 skipped (each skip names `maturin build`); with the wheel
built from `NODE.BareMetal/SRC/Core` (`maturin build --release`) and installed, 5 passed — the
venue fixture, a missing file, a malformed row, and a refusal when the parser is absent.

---

## 🚀 Usage & Integration
* **Web Portal Entry**: none of its own. The records are drawn (from the worker's `incoming/result`) by the Data Migration frame
  `PODS/POD.THIN/WEB.Frontend-Assets/SRC/FrontEnd/Gui_Frames/3_Data_Migration/4_CSV/CSV_Importer.json`.
  `display.json` declares the folder to the launcher with `entry: csv_importer.rs`, `tech: rust`
  (as DATATYPE.wireless.IAS does), and `requires: {plugins: [wireless_csv], services: [broker]}`,
  so the Launch Pad marks it *needs wireless_csv (not running)* when Worker-WIRELESS_CSV is down.
* **Remote Repository**: `https://github.com/LikeDotAudio/APK.softapp.DATATYPE.wireless.CSV.git`

---

## 🏛 Philosophy

- **A tool is a job on the bus, not an endpoint.** The worker opens no port; it answers
  `outgoing/job` on `incoming/result` and describes itself at `incoming/api`
  (`SRC/src/lib.rs` `run_job_agent`, `Docker/docker-compose.yml` has no `ports:`).
- **One parser for every caller.** The worker and the DataMigration import both run
  `classify_csv_str`; the worker pulls `csv_importer.rs` in by `#[path]` rather than copying it
  (`SRC/src/lib.rs`).
- **A job is always answered.** Bad JSON, a missing body, bad base64 or an oversized CSV comes
  back as `ok: false` with an `error`, never as silence (`handle_job`, its tests).
- **The bus is the API.** The importer's output reaches other apps as a retained message on a
  topic built from the contract crate (`csv_importer_mqtt_topic()`), published by the node's
  data-migration daemon, never by the parser itself.
- **Topics come from the contract, never typed out** (`apkaudio_contracts::topics::data_migration`,
  comment above `csv_importer_mqtt_topic()`).
- **Do not invent a value.** Only the Soundbase layout has a band column; for the others `band` is
  left empty "rather than invented" (`StandardRfRecord`), and so is any zone/group/device/name
  cell the file left empty (`classify_csv_str`; it once filled `N/A`, `Generic RF`, `CH_<n>`).
- **One parser, no silent fallback.** The Python edge refuses to run without the Rust parser
  rather than quietly producing different records (`IMPORT:file:CSV/Entry.py`,
  `FileReaders/from_csv_unknown.py`, `Methods/csv_parser.py`; tested by
  `Tests/test_negative_paths.py`).
- **A failing fixture fails the run.** The runner counts failures and exits 1
  (`test_csv_pattern_importer.rs`); it used to print and exit 0.
- **Keep going on a bad row.** Unreadable records and cells that are not a frequency in range
  are skipped, not fatal (`rdr.records().flatten()`, the 100–3000 MHz guard).

<!-- wiki:smoke -->
**Smoke test** — ✅ PASS 2/2 since 2026-10-30 13:37 EDT — [results](Documentation/%F0%9F%A7%A9Manual%20and%20Artifacts/SMOKE.md)
<!-- /wiki:smoke -->
