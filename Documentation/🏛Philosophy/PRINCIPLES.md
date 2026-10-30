# Principles — DATATYPE.wireless.CSV

The rules the CSV importer and its worker keep, each with the code that keeps it. Ecosystem rules are cited, not copied: [Refusal over fabrication](../../../../../../../Documentation/%F0%9F%8F%9BPhilosophy/Doctrine/readme.Refusal%20over%20fabrication.md), [Dockers golden rule](../../../../../../../Documentation/%F0%9F%8F%9BPhilosophy/Dockers/goldenrule.md).

← [Wiki home](../README.md)

| Principle | Kept by | Where it breaks |
|---|---|---|
| **One parser for every caller.** The DataMigration import, the worker and the runner all run `classify_csv_str`; the worker pulls `csv_importer.rs` in by `#[path]`, never a copy. | `classify_csv_str` (`csv_importer.rs` line 74); `SRC/src/lib.rs` | The Python edge (`IMPORT:file:CSV/`) calls a *different* Rust parser, `oaRustCore.csv_parser.convert_csv_unknown`, with its own header aliases |
| **Parsing has no side effect; publishing is a separate step.** | `classify_csv_str` vs `parse_and_classify_csv_str` (lines 60–74) | — |
| **Do not invent a value.** `band` is empty where the file has no band column, and an empty zone/group/device/name cell stays empty. | `StandardRfRecord` doc (line 27); the zone/group and custom arms of `classify_csv_str` | — (placeholders `N/A`, `Generic RF`, `CH_<n>` removed 2026-10-30) |
| **A tool is a job on the bus, not an endpoint.** | `run_job_agent` (`SRC/src/lib.rs` line 119); `Docker/docker-compose.yml` has no `ports:` | — |
| **A job is always answered.** Bad JSON, bad base64, an oversized body → `ok: false` with an `error`. | `handle_job` (`SRC/src/lib.rs` line 55), `max_job_bytes` (line 43) | — |
| **Topics come from the contract.** | `csv_importer_mqtt_topic()` (line 45) → `apkaudio_contracts::topics::data_migration` | — |
| **No silent fallback.** Without the Rust parser the Python edge raises; it never parses in Python. | `IMPORT:file:CSV/Entry.py`, `FileReaders/from_csv_unknown.py`, `Methods/csv_parser.py`; `Tests/test_negative_paths.py` | — |
| **Keep going on a bad row.** Unreadable records and out-of-range cells are skipped. | `rdr.records().flatten()` (lines 100, 118, 144); the 100–3000 MHz guard (127, 148) | A skipped row is not counted or reported anywhere |
| **Least privilege for the worker.** | `read_only: true`, `cap_drop: [ALL]`, uid 10001 (`Docker/docker-compose.yml`, `Docker/Dockerfile`) | — |
