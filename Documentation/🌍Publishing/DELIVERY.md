# Delivery — DATATYPE.wireless.CSV

← [Wiki home](../README.md)

This repo ships in three places, none with a page of its own (no live URL):

| What | Built by | Lands |
|---|---|---|
| the crate `apkaudio-wireless-csv` (`SRC/importer/`) in the **orchestrator** | `MANAGEMENT.DataMigration` depends on it by git rev and re-exports it as `csv_importer`; `oaRustCore` (NODE.BareMetal `SRC/Core`) re-exports that crate and its CSV parser (`PARSER/csv_parser.rs`) calls it. `NODE.BareMetal/Docker/Dockerfile` copies this repository so Core's `[patch]` resolves it locally. | the **Node-BareMetal** container (`node-baremetal:local`); rebuild with `up-stack.sh NODE.BareMetal` |
| the **worker** `apk-worker-wireless-csv` | `Docker/Dockerfile` ; runtime `debian:bookworm-slim`, user `worker` | container **Worker-WIRELESS_CSV**, project `workers`, via `docker compose -f PODS/POD.workers/Docker/docker-compose.yml up -d --build`; healthcheck `apk-plugin-health` |
| the files themselves | `PODS/POD.THIN/Softapps` copied into **WEB.Frontend-Assets**; the monorepo's FTPS job (`.github/workflows/ci.yml`) uploads the checkout to apk.audio | nothing there opens it — `display.json`'s `entry` is `SRC/importer/src/lib.rs` (`tech: rust`), and `requires` names the `wireless_csv` worker |

Remote: `https://github.com/LikeDotAudio/APK.softapp.DATATYPE.wireless.CSV.git` (no workflow of its own).

**Tested:** by hand, one command — `bash SRC/check.sh`: `cargo test --workspace --locked` in `SRC/` (worker 8, parser 2 + fixtures 2, kernel 8 — 2026-10-30), `pytest IMPORT:file:CSV/Tests SRC/python/tests` (12 passed, 3 skipped without the `oaRustCore` wheel). MANAGEMENT.DataMigration's `tests/frame_headers.rs` holds the records to the Data Migration frames. The worker's self-test runs a five-row CSV through `handle_job` at start-up. `.apk.scripts/smoke.py` checks only that HEAD is pushed ([SMOKE](../%F0%9F%A7%A9Manual%20and%20Artifacts/SMOKE.md)).

**Not automated:** no CI runs any of these tests (the root CI verifies nothing on purpose; `SRC/check.sh` is the pre-push check); the `oaRustCore` wheel the Python edge needs is built by hand (`maturin build` in `NODE.BareMetal/SRC/Core`). The fixture tests publish nothing, so they are safe on a rig in use.

## Delivered from the 2026-10-30 audit

- **Job loop from the runner** — `SRC/src/lib.rs` `run_job_agent` is now `apk_plugin_runner::jobs::JobWorker` (session, ConnAck resubscribe, lane thread, busy answer); the hand-built rumqttc session, credentials, packet size and publish are gone, and so is the same loop in the IAS, SOUNDbase and SHUREwirelessWorkbench workers. Verified by `cargo test --workspace` in `SRC/` (8 worker tests, self test included).
- **SDK by tag** — `SRC/Cargo.toml` names `apk-plugin-runner` and `apkaudio-contracts` from APK.plugin.SDK `tag = "sdk-v0.5"` (no path into DISCOVERY.Engine / NODE.BareMetal); git-excluded `SRC/.cargo/config.toml` patches them to the checkout, the Dockerfile passes the same patches as `--config`; Cargo.lock refreshed (contracts 0.2.0, W01 handoff). Standalone builds need the next SDK tag (carries `jobs`) — publishing tags is the owner's.
- **The parser is a crate with one owner** — `SRC/csv_importer.rs` → `SRC/importer/` (`apkaudio-wireless-csv`); the worker links it by path and MANAGEMENT.DataMigration by git rev, so no other repository `#[path]`-compiles it. The `test_csv_pattern_importer` runner that DataMigration built from this repo's file is `SRC/importer/tests/fixtures.rs`, resolved from this crate's own manifest. Verified by `cargo test --workspace`.
- **Shared wireless kernel** — `SRC/kernel/` (`apkaudio-wireless-kernel`): `is_usable_mhz` (100–3000 MHz, exclusive), `import_status`, and with `worker` the file-job envelope (`read_file_job`, `FileJob::answer`, `failure`, `max_job_bytes`). All four wireless importers/workers use it (IAS / SOUNDbase / SHUREwirelessWorkbench by git rev). 8 kernel tests.
- **Versioned wire answer + pinned shape** — every answer `handle_job` sends carries `schema: 1` (`kernel::job::SCHEMA`); `the_result_shape_is_pinned` holds the `result` keys and a record's keys, so a parser rename fails a test before it is a wire change. The result stays the parser's struct (no second DTO type) — the test is the contract.
- **One local check** — `SRC/check.sh` runs `cargo test --workspace --locked` and both pytest suites (the root CI verifies nothing on purpose).
- **Python edge isolated** — `Entry.py` / `FileReaders/from_csv_unknown.py` take logging and the lifecycle from `apk_wireless_import` (`SRC/python/`, package `apk-wireless-import`, 8 tests) instead of NODE.BareMetal's `importer_logging`; `Tests/conftest.py` no longer walks into NODE.BareMetal. `Entry.py` declares `STATUS`, `report_status()` stops claiming "Running with Rust Engine", `run_tests()` runs a real file. Kept by design: `oaRustCore` stays the edge's engine (no Python parser, by the "no silent fallback" rule). Verified by `pytest` (12 passed, 3 skipped).
- **Home of the shared Python edge** — `apk_wireless_import` (MARKER_HEADERS + `Lifecycle` + `log`) serves the PDF, IAS and Shure edges too (from the SoundBase-PDF audit).
- **Health probe from the runner, not its source file** — `SRC/src/bin/apk-plugin-health.rs` is one line over `apk_plugin_runner::health_check()` (W02 foundation); the `[[bin]]` that compiled DISCOVERY.Engine's `RUNNER/src/bin/apk-plugin-health.rs` by path is gone. Verified by `bash SRC/check.sh` and `docker build`.
- **The image builds from this repository alone** — `Docker/Dockerfile` copies only `SRC/` (compose `context: ../SRC`); the COPYs of `contracts/` and DISCOVERY.Engine's runner and every `--config patch` are gone. `SRC/Cargo.toml` takes runner and contracts by SDK tag (`sdk-v0.7` after the coordinator's repin; jobs arrived in `sdk-v0.6`); Cargo.lock now records those GitHub sources (no local patch). Verified by a compose build of `Docker/Dockerfile` with context `SRC/` and `cargo test --workspace` against GitHub.
