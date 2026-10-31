# Principles — DATATYPE.wireless.CSV

The rules the CSV importer and its worker keep, each with the code that keeps it. Ecosystem rules are cited, not copied: [Refusal over fabrication](../../../../../../../Documentation/%F0%9F%8F%9BPhilosophy/Doctrine/readme.Refusal%20over%20fabrication.md), [Dockers golden rule](../../../../../../../Documentation/%F0%9F%8F%9BPhilosophy/Dockers/goldenrule.md).

← [Wiki home](../README.md)

| Principle | Kept by | Where it breaks |
|---|---|---|
| **One parser for every caller, one owner.** The DataMigration import, the worker and the Python edge (`oaRustCore.csv_parser.convert_csv_unknown`) all run `classify_csv_str` from the crate `apkaudio-wireless-csv`: the worker by path, DataMigration by git rev. No repository compiles the file by `#[path]`. | `SRC/importer/src/lib.rs`; `SRC/Cargo.toml` | — (until 2026-10-30 the Python edge's `convert_csv_unknown` was a second Rust parser, and DataMigration `#[path]`-included this file) |
| **What the four importers share is written once.** The 100–3000 MHz rule, the status word and the job envelope live in the kernel; the marker header row and the importer lifecycle in `apk_wireless_import`. | `SRC/kernel/src/lib.rs`, `SRC/kernel/src/job.rs`; `SRC/python/apk_wireless_import/__init__.py` | — (until 2026-10-30 each was re-typed in three or four importers) |
| **The wire answer is versioned and its shape is pinned.** `schema` rises with any key a reader relies on. | `kernel::job::SCHEMA`; `the_result_shape_is_pinned` (`SRC/src/lib.rs`) | the result is still the parser's struct serialised, held by the shape test rather than a separate DTO type |
| **Builds from its own repository.** Runner and contracts by `tag = "sdk-v0.7"`; Cargo.lock pins those git sources; the image builds from `SRC/` alone. | `SRC/Cargo.toml`, `SRC/Cargo.lock`; `Docker/Dockerfile` (context `../SRC`) | building against a local runner checkout needs a hand-added, git-excluded `[patch]` |
| **Parsing has no side effect; publishing is a separate step.** | `csv_importer.rs` has no MQTT client; the daemon (`data_migration.rs`) and the worker publish | — (until 2026-10-30 `parse_and_classify_csv_str` published retained to the loopback broker) |
| **Do not invent a value.** `band` is empty where the file has no band column, and an empty zone/group/device/name cell stays empty. | `StandardRfRecord` doc (line 29); the zone/group and custom arms of `classify_csv_str` | — (placeholders `N/A`, `Generic RF`, `CH_<n>` removed 2026-10-30) |
| **A tool is a job on the bus, not an endpoint.** | `run_job_agent` (`SRC/src/lib.rs`, the runner's `JobWorker`); `Docker/docker-compose.yml` has no `ports:` | — |
| **A job is always answered.** Bad JSON, bad base64, an oversized body → `ok: false` with an `error`. | `handle_job` (`SRC/src/lib.rs`) over `kernel::job::read_file_job`; `max_job_bytes` | — |
| **No silent fallback.** Without the Rust parser the Python edge raises; it never parses in Python. | `IMPORT.file.CSV/Entry.py`, `FileReaders/from_csv_unknown.py`, `Methods/csv_parser.py`; `Tests/test_negative_paths.py` | — |
| **Kept by design: the Python edge's engine is `oaRustCore`.** The edge stays (it is the desktop-app interface) but owns no parsing; it no longer imports NODE.BareMetal's `importer_logging`, only `apk_wireless_import`, and `oaRustCore` is an optional native wheel whose absence reads `STATUS = "unavailable"`. | `IMPORT.file.CSV/Entry.py`; `Tests/test_entry_status.py` | — |
| **Keep going on a bad row.** Unreadable records and out-of-range cells are skipped. | `rows` in `classify_csv_str`; the 100–3000 MHz guard | — (each one is counted in `skipped_rows`; a file with no records is `status: "empty"`) |
| **Least privilege for the worker.** | `read_only: true`, `cap_drop: [ALL]`, uid 10001 (`Docker/docker-compose.yml`, `Docker/Dockerfile`) | — |
