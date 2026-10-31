# Architecture — DATATYPE.wireless.CSV

← [Wiki home](../README.md)

## Using it

- **From the Data Migration tab** — choose a CSV in the *CSV* frame (`WEB.Frontend-Assets/SRC/FrontEnd/Gui_Frames/3_Data_Migration/4_CSV/`). The browser reads the file and the frame posts its text as a job (below, `file_dialog.publish_as: "job"`); its table is bound to `…/wireless_csv/incoming/result`. The orchestrator's `DataMigration/CSV/FilePath` → retained `…/CSV/ImportedData` inbox is retired (LOOSE-datamigration-frames-to-workers).
- **As a job** — publish `{"job_id", "filename", "text" | "content_b64"}` to `APK.audio/System/Protocols/wireless_csv/outgoing/job`; the answer `{"schema": 1, "job_id", "tool", "ok", "result", "error", "elapsed_ms"}` arrives on `…/wireless_csv/incoming/result`. Neither is retained.

## How it works

```text
CSV text ──▶ classify_csv_str()  (SRC/importer, crate apkaudio-wireless-csv, no side effect)
               header row, upper-cased → SoundbaseReport | StandardZoneGroup | UnknownCustom
               each row → StandardRfRecord { zone, group, device, channel_name, frequency_mhz, band }
               status "success" | "empty" (no records); skipped_rows = data rows with no record
          └─▶ SRC handle_job() → JSON answer on …/wireless_csv/incoming/result   (Worker-WIRELESS_CSV)

job payload ─▶ kernel::job::read_file_job (job_id, text|content_b64, size)  ─▶ parser ─▶ FileJob::answer (schema 1)
               fed by apk_plugin_runner::jobs::JobWorker (session, ConnAck resubscribe, lane thread, busy answer)
```

Who depends on whom (all one-way):

```text
DATATYPE.wireless.{IAS,SOUNDbase,SHUREwirelessWorkbench}/SRC ──git rev──▶ SRC/kernel (apkaudio-wireless-kernel)
MANAGEMENT.DataMigration ──git rev──▶ SRC/importer (apkaudio-wireless-csv)
the IAS / PDF / Shure Python edges ──package──▶ SRC/python (apk_wireless_import)
SRC (worker) ──path──▶ SRC/importer, SRC/kernel ; ──SDK tag sdk-v0.6──▶ apk-plugin-runner, apkaudio-contracts
```

| Pattern | Recognised by | Read |
|---|---|---|
| Soundbase export | headers contain `SPACING` and `FREQUENCY` | Group, Model, Band, Name, Preset, Spacing, Frequency (by position) |
| Standard zone/group | headers contain `ZONE` and `FREQ` | ZONE, GROUP, DEVICE, NAME, FREQ; 100–3000 MHz only |
| Arbitrary | anything else | the first cell per row that is 100–3000 MHz; other fields empty |

## Files

| Path | What |
|---|---|
| `SRC/importer/` | the parser crate `apkaudio-wireless-csv` (it publishes nothing); `tests/fixtures.rs` holds six fixtures to their record counts (`report_converted.csv` 0, a negative fixture). MANAGEMENT.DataMigration depends on it by git rev and re-exports it as `csv_importer` |
| `SRC/kernel/` | `apkaudio-wireless-kernel`: the band rule, the status word, and (feature `worker`) the versioned file-job envelope — shared by all four wireless workers |
| `SRC/python/` | `apk_wireless_import`: `MARKER_HEADERS`, `Lifecycle`, `log` — shared by the Python edges here and in IAS, PDF, Shure |
| `test_data/` | the fixtures (`report.csv`, `venue*.csv`, `SB PDF.csv`, …) |
| `SRC/` | the worker crate `apk-worker-wireless-csv`: `lib.rs` (`handle_job`, `run_job_agent` on `JobWorker`), `self_test.rs`, `bin/apk-worker-wireless-csv.rs` (`apk_plugin_runner::main!`), `config.ini` (`max_job_bytes`), `check.sh` |
| `Docker/` | the worker image and compose, `include:`d by `PODS/POD.workers/Docker/docker-compose.yml` |
| `IMPORT.file.CSV/` | the Python edge from the desktop app: `Entry.py` / `FileReaders/from_csv_unknown.py` call `oaRustCore.csv_parser.convert_csv_unknown`, which runs `parse_and_classify_csv` (this parser) and raises on a file with no record; `Tests/` (pytest; `conftest.py` puts the package root and `../SRC/python` on the path) |

## Testing

```bash
bash PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.CSV/SRC/check.sh
#   cargo test --workspace --locked in SRC/ (worker 8, parser 2 + fixtures 2, kernel 8)
#   pytest IMPORT.file.CSV/Tests SRC/python/tests (12 passed, 3 skipped without the oaRustCore wheel)
```
