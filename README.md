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
conversion lives. It runs two ways: as the crate `apkaudio-wireless-csv` that
MANAGEMENT.DataMigration (and through it oaRustCore) depends on by git rev; and as a worker,
where any client sends the CSV itself as a job and receives the records as the answer. Both run
the same function, `classify_csv_str` in `SRC/importer/src/lib.rs`.

It is also the home of what the four wireless importers share: the **wireless kernel**
(`SRC/kernel/`, crate `apkaudio-wireless-kernel` — the 100–3000 MHz rule, the status word, the
versioned file-job envelope) and the Python package **`apk_wireless_import`** (`SRC/python/` —
`MARKER_HEADERS`, the stateless importer lifecycle, the logging call).

---

## 🧩 Architectural Role
* **Family**: DATATYPE — an importer for somebody else's file format
* **Category**: Wireless / RF coordination reports — source `4_CSV` of the four DataMigration importers (IAS, Soundbase, Wireless Workbench, CSV)
* **Pipeline Stage**: Data Migration — the `Gui_Frames/3_Data_Migration/4_CSV` frame posts the file's text as a job to `wireless_csv` and shows the answer from `…/wireless_csv/incoming/result`; the orchestrator's `FilePath` inbox is retired (LOOSE-datamigration-frames-to-workers)

---

## ⚙️ Repository Overview

| Path | What it is |
|---|---|
| `SRC/importer/` | The parser crate `apkaudio-wireless-csv` (`src/lib.rs`), and `tests/fixtures.rs`: six fixtures, each with the record count it must import. |
| `SRC/kernel/` | `apkaudio-wireless-kernel`: `is_usable_mhz` / `usable_mhz_text` (100–3000 MHz, exclusive), `import_status`, and with feature `worker` the `job` module (`read_file_job`, `FileJob::answer`, `failure`, `max_job_bytes`, `SCHEMA`). Taken by IAS, SOUNDbase and SHUREwirelessWorkbench by git rev. |
| `SRC/python/` | The package `apk-wireless-import` (`apk_wireless_import`): `MARKER_HEADERS`, `Lifecycle`, `log`; its tests in `SRC/python/tests/`. Taken by the IAS, PDF and Shure Python edges. |
| `SRC/check.sh` | Every check here in one command: `cargo test --workspace --locked` and both pytest suites. |
| `test_data/` | Fixtures: `report.csv`, `report_converted.csv`, `venue.csv`, `venue_data.csv`, `venue_converted.csv`, `SB PDF.csv`, and a Lectrosonics programming guide PDF. |
| `IMPORT.file.CSV/` | The Python edge (`oaFileImportCSV`) from the desktop app this was lifted out of — see below. |
| `SRC/` | The worker crate `apk-worker-wireless-csv` (PLAN-3423.01): links `importer/` and `kernel/` by path (workspace members) and answers jobs on the bus. |
| `Docker/` | The worker's `Dockerfile` (context `SRC/`) and `docker-compose.yml` (project `workers`, container `Worker-WIRELESS_CSV`), `include:`d by `PODS/POD.workers/Docker/docker-compose.yml`. |

### `SRC/importer/` (`apkaudio-wireless-csv`)

`classify_csv_str(text, source_name)` reads the header row, upper-cases it and picks a pattern.
It has no side effect; `parse_and_classify_csv(path)` reads a file and calls it. Nothing in
the parser publishes — the caller does (below):

| Pattern | Recognised by | Columns read |
|---|---|---|
| `SoundbaseReport` — "Soundbase CSV Export" | headers contain `SPACING` and `FREQUENCY` | Group, Model, Band, Name, Preset, Spacing, Frequency (Group fills both zone and group) |
| `StandardZoneGroup` — "Standard Zone/Group CSV" | headers contain `ZONE` and `FREQ` | ZONE, GROUP, DEVICE, NAME, FREQ; rows outside 100–3000 MHz are dropped |
| `UnknownCustom` — "Arbitrary Custom CSV" | anything else | the first cell per row that parses as 100–3000 MHz |

Each row becomes a `StandardRfRecord { zone, group, device, channel_name, frequency_mhz, band }`.
`band` is filled only by the Soundbase layout; the others leave it empty. An empty
zone/group/device/name cell stays empty, and the custom layout, which finds only a frequency,
leaves all four empty — `pattern_name` already says what kind of file it was.

`status` is `"success"` only when the file yielded records, `"empty"` otherwise
(`apkaudio_wireless_kernel::import_status`, the one rule all four importers use). The band guard
is `apkaudio_wireless_kernel::is_usable_mhz`. `skipped_rows` counts the data rows that yielded no record —
unreadable, too short, or no frequency in 100–3000 MHz; a blank line is not counted.

**The importer opens no broker.** It returns the `CsvRecognizedResult` (status, source file,
pattern, topic, counts, records); the `wireless_csv` worker answers a job with it. The
orchestrator's DataMigration inbox, which published it retained on
`APK.audio/System/DataMigration/CSV/ImportedData`, is retired (LOOSE-datamigration-frames-to-workers).
Until 2026-10-30 the
importer also carried a publisher of its own (`parse_and_classify_csv_str`, `publish_to_mqtt`)
to the loopback broker, which only the test runner still reached — so a test run overwrote the
live retained document.

### The worker (`SRC/`, `Docker/`)

```text
client ── APK.audio/System/Protocols/wireless_csv/outgoing/job ──▶ Worker-WIRELESS_CSV
          {"job_id": "…", "filename": "venue.csv", "text": "<csv>"}   ("content_b64" for raw bytes)
client ◀─ APK.audio/System/Protocols/wireless_csv/incoming/result ── Worker-WIRELESS_CSV
          {"schema": 1, "job_id": "…", "tool": "DATATYPE.wireless.CSV", "ok": true, "result": {…}, "error": null, "elapsed_ms": 0}
```

`schema` is `apkaudio_wireless_kernel::job::SCHEMA`; `the_result_shape_is_pinned` holds the
`result` keys and a record's keys, so a renamed parser field fails a test before it is a wire
change. (The `busy` answer the runner sends when four jobs already wait is the runner's shared
shape, without `schema` or `result`.)

- `SRC/src/lib.rs` `handle_job` is the tool: the kernel's `read_file_job` decodes the job and
  refuses one over `max_job_bytes` (`SRC/config.ini`, 8 MiB), `classify_csv_str` parses, and
  `FileJob::answer` wraps it. A malformed job gets `ok: false` and an `error`, with its `job_id`
  when it had one.
- `SRC/src/lib.rs` `run_job_agent` is the runner's job loop, `apk_plugin_runner::jobs::JobWorker`:
  its own session, subscribed to `outgoing/job` on every ConnAck, the job on a lane thread off
  the session thread, each answer to `incoming/result`. Neither leaf is retained
  (`baremetal_io::module_job` / `module_result`).
- `SRC/src/bin/apk-worker-wireless-csv.rs` hands both to `apk_plugin_runner::main!`. The runner
  (`PODS/POD.THICK/DISCOVERY.Engine/SRC/RUNNER`) publishes the config, the last will, the
  self-test door and the retained self-description at `wireless_csv/incoming/api`.
- `SRC/src/self_test.rs` runs a five-row CSV through `handle_job` and expects 4 records.
- The crate names `apk-plugin-runner` and `apkaudio-contracts` from APK.plugin.SDK by tag
  (`sdk-v0.7`), like the THICK plugins; Cargo.lock pins the git sources, and the image
  (`Docker/Dockerfile`) builds from this repository's `SRC/` alone. The runtime is `debian:bookworm-slim`, user
  `worker` (uid 10001). Compose runs it `read_only`, `cap_drop: [ALL]`, no published port, on
  the `workers` network plus `apk-audio_default` to reach the broker as `broker:1883`.
  The healthcheck is `apk-plugin-health`, one line over `apk_plugin_runner::health_check()`.

```bash
bash SRC/check.sh                                                             # every check, cargo + pytest
docker compose -f PODS/POD.workers/Docker/docker-compose.yml up -d --build    # from the monorepo root (the pod aggregator)
mosquitto_pub -h 127.0.0.1 -t APK.audio/System/Protocols/wireless_csv/outgoing/job \
  -m '{"job_id":"t1","filename":"x.csv","text":"ZONE,GROUP,DEVICE,NAME,FREQ\nA,G,D,N,566.6\n"}'
```

### Build and test

```bash
cd SRC && cargo test --workspace      # worker, parser crate (fixtures), kernel
bash SRC/check.sh                     # the same, --locked, plus both pytest suites
```

`SRC/importer/tests/fixtures.rs` reads its six fixtures from this repository's `test_data/`
(through its own `CARGO_MANIFEST_DIR`). It was the `test_csv_pattern_importer` `[[bin]]` that
MANAGEMENT.DataMigration built from this repository's file until the 2026-10-30 audit. Each
fixture carries the record count it must import: `report.csv` 168, `venue*.csv` 253 each,
`SB PDF.csv` 20 (Soundbase), `report_converted.csv` 0 (`status: "empty"`, 179 rows skipped).

`report_converted.csv` is a negative fixture, not a broken one. It is the output of a retired
PDF-report converter (not the current `convert_csv_unknown`, which never writes "No Match") that
found no frequency on any line and wrote `No Match` into every FREQ cell; its source text is not
kept. The importer is right to drop every row: `No Match` is not a 100–3000 MHz frequency. The
same coordination report, converted correctly, is `report.csv`.

### `IMPORT.file.CSV/` (Python)

`Entry.py` and `FileReaders/from_csv_unknown.py` expose `convert_unknown_csv_report()`, which
calls `oaRustCore.csv_parser.convert_csv_unknown` and raises `RuntimeError` if it is not
importable — there is no Python CSV parser, on purpose. `Methods/csv_parser.py` imports it
unguarded. Logging and the lifecycle come from `apk_wireless_import` (`SRC/python/`), whose
`log` falls back to a no-op without `oaLogging`; `Entry.py` declares `STATUS` ("live" /
"unavailable") and its `run_tests()` runs a two-row file through the parser — never True for
having run nothing. `convert_csv_unknown` (`NODE.BareMetal/SRC/Core/CORE.engine.RUST/PARSER/csv_parser.rs`,
built into the `oaRustCore` wheel) does not parse: it runs `parse_and_classify_csv` from
`apkaudio-wireless-csv` — the same parser as every other caller — and returns `(headers, rows)`
with the columns `ZONE, GROUP, DEVICE, NAME, FREQ_MHZ, BAND`. A file with no record is an
`OSError`, never an empty list.

```bash
cd "IMPORT.file.CSV" && python3 -m pytest -q Tests        # conftest puts ../SRC/python on sys.path unless installed
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
  `display.json` declares the folder to the launcher with `entry: SRC/importer/src/lib.rs`, `tech: rust`
  (as DATATYPE.wireless.IAS does), and `requires: {plugins: [wireless_csv], services: [broker]}`,
  so the Launch Pad marks it *needs wireless_csv (not running)* when Worker-WIRELESS_CSV is down.
* **Remote Repository**: `https://github.com/LikeDotAudio/APK.softapp.DATATYPE.wireless.CSV.git`

---

## 🏛 Philosophy

- **A tool is a job on the bus, not an endpoint.** The worker opens no port; it answers
  `outgoing/job` on `incoming/result` and describes itself at `incoming/api`
  (`SRC/src/lib.rs` `run_job_agent`, `Docker/docker-compose.yml` has no `ports:`).
- **One parser for every caller.** The worker and the DataMigration import both run
  `classify_csv_str`; the worker links the parser crate by path and DataMigration by git rev —
  one owner, no `#[path]` include anywhere (`SRC/importer/`).
- **What the importers share is written once.** The band rule, the status word and the job
  envelope are the kernel's (`SRC/kernel/`); the marker header row and the lifecycle are
  `apk_wireless_import`'s (`SRC/python/`).
- **A job is always answered.** Bad JSON, a missing body, bad base64 or an oversized CSV comes
  back as `ok: false` with an `error`, never as silence (`handle_job`, its tests).
- **The bus is the API.** The importer's output reaches other apps as the worker's answer on
  `APK.audio/System/Protocols/wireless_csv/incoming/result` (a topic built by the contract's
  `baremetal_io::module_result`), never published by the parser itself.
- **Do not invent a value.** Only the Soundbase layout has a band column; for the others `band` is
  left empty "rather than invented" (`StandardRfRecord`), and so is any zone/group/device/name
  cell the file left empty (`classify_csv_str`; it once filled `N/A`, `Generic RF`, `CH_<n>`).
- **One parser, no silent fallback.** The Python edge refuses to run without the Rust parser
  rather than quietly producing different records (`IMPORT.file.CSV/Entry.py`,
  `FileReaders/from_csv_unknown.py`, `Methods/csv_parser.py`; tested by
  `Tests/test_negative_paths.py`).
- **A failing fixture fails the run.** `SRC/importer/tests/fixtures.rs` asserts each count
  (the old runner printed and exited 0 until it was fixed, then moved here as a test).
- **Keep going on a bad row.** Unreadable records and cells that are not a frequency in range
  are skipped, not fatal (`rdr.records().flatten()`, the 100–3000 MHz guard).

<!-- wiki:smoke -->
**Smoke test** — ✅ PASS 2/2 since 2026-10-30 18:07 EDT — [results](Documentation/%F0%9F%A7%A9Manual%20and%20Artifacts/SMOKE.md)
<!-- /wiki:smoke -->
