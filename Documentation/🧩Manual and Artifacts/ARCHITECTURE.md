# Architecture — DATATYPE.wireless.CSV

← [Wiki home](../README.md)

## Using it

- **From the Data Migration tab** — choose a CSV in the *CSV* frame (`WEB.Frontend-Assets/SRC/FrontEnd/Gui_Frames/3_Data_Migration/4_CSV/`). The browser reads the file and the frame posts its text as a job (below, `file_dialog.publish_as: "job"`); its table is bound to `…/wireless_csv/incoming/result`. The orchestrator's `DataMigration/CSV/FilePath` → retained `…/CSV/ImportedData` path still exists for any other publisher, but the frame no longer uses it.
- **As a job** — publish `{"job_id", "filename", "text" | "content_b64"}` to `APK.audio/System/Protocols/wireless_csv/outgoing/job`; the answer `{"job_id", "tool", "ok", "result", "error", "elapsed_ms"}` arrives on `…/wireless_csv/incoming/result`. Neither is retained.

## How it works

```text
CSV text ──▶ classify_csv_str()  (csv_importer.rs, no side effect)
               header row, upper-cased → SoundbaseReport | StandardZoneGroup | UnknownCustom
               each row → StandardRfRecord { zone, group, device, channel_name, frequency_mhz, band }
               status "success" | "empty" (no records); skipped_rows = data rows with no record
          ├─▶ NODE.BareMetal's data-migration daemon (oaRustCore::data_migration)
          │      → publishes retained to …/DataMigration/CSV/ImportedData on its own session
          └─▶ SRC handle_job() → JSON answer on …/wireless_csv/incoming/result   (Worker-WIRELESS_CSV)
```

| Pattern | Recognised by | Read |
|---|---|---|
| Soundbase export | headers contain `SPACING` and `FREQUENCY` | Group, Model, Band, Name, Preset, Spacing, Frequency (by position) |
| Standard zone/group | headers contain `ZONE` and `FREQ` | ZONE, GROUP, DEVICE, NAME, FREQ; 100–3000 MHz only |
| Arbitrary | anything else | the first cell per row that is 100–3000 MHz; other fields empty |

## Files

| Path | What |
|---|---|
| `csv_importer.rs` | the parser (it publishes nothing); module `csv_importer` of `MANAGEMENT.DataMigration` (`mod.rs`, `#[path]`) |
| `test_csv_pattern_importer.rs` | `[[bin]]` runner over six fixtures; exits 1 on any failure; publishes nothing |
| `test_data/` | the fixtures (`report.csv`, `venue*.csv`, `SB PDF.csv`, …) |
| `SRC/` | the worker crate `apk-worker-wireless-csv`: `lib.rs` (`handle_job`, `run_job_agent`), `self_test.rs`, `bin/apk-worker-wireless-csv.rs` (`apk_plugin_runner::main!`), `config.ini` (`max_job_bytes`) |
| `Docker/` | the worker image and compose, `include:`d by `PODS/POD.workers/Docker/docker-compose.yml` |
| `IMPORT:file:CSV/` | the Python edge from the desktop app: `Entry.py` / `FileReaders/from_csv_unknown.py` call `oaRustCore.csv_parser.convert_csv_unknown`; `Tests/` (pytest, `conftest.py` sets the path) |

## Testing

```bash
cd PODS/POD.THIN/Softapps/MANAGEMENT_AND_UTILITIES/MANAGEMENT.DataMigration && cargo test --lib
cd PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.CSV/SRC && cargo test        # worker: 7 passed
cd "PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.CSV/IMPORT:file:CSV" && python3 -m pytest -q Tests
```
