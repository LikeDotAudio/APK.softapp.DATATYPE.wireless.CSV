# 🔍 Architecture audit — DATATYPE.wireless.CSV (2026-10-30)

Audited against the APK.audio multi-repo architecture standard: coupling, boundaries, duplication, contracts.
← [Audits](INDEX.md) · Improvements: [🛣Roadmap/IMPROVEMENTS.md](../%F0%9F%9B%A3Roadmap/IMPROVEMENTS.md)

## Bounded context
Wireless-coordination CSV import: recognise one of three layouts (Soundbase, Zone/Group, custom) and emit `StandardRfRecord`s; runs as the bus worker `Worker-WIRELESS_CSV` and as a module compiled into MANAGEMENT.DataMigration. Rust (~600 lines) plus a 250-line Python edge `IMPORT:file:CSV/`, 7 fixtures. Sampled: all Rust, `Docker/*`, `SRC/Cargo.toml`, `config.ini`, Python entry/tests, README.

## Context map
| Talks to | How (topic / endpoint / file / import) | Relationship |
|---|---|---|
| Browser Data Migration frame (WEB.Frontend-Assets `3_Data_Migration/4_CSV`) | `APK.audio/System/Protocols/wireless_csv/outgoing/job` → `…/incoming/result` (`schema: 1`); `…/incoming/api` retained | Published Language (bus) |
| APK.plugin.SDK (`apk-plugin-runner`, `apkaudio-contracts`) | Cargo `git … tag = "sdk-v0.6"`, pinned in Cargo.lock; job loop `apk_plugin_runner::jobs::JobWorker` | Conformist, by version |
| DATATYPE.wireless.IAS / .SOUNDbase / .SHUREwirelessWorkbench | depend on `SRC/kernel` (`apkaudio-wireless-kernel`) by git rev | Supplier (shared kernel, one-way) |
| MANAGEMENT.DataMigration | depends on `SRC/importer` (`apkaudio-wireless-csv`) by git rev | Supplier (crate) |
| IAS / PDF / Shure Python edges | `apk_wireless_import` (`SRC/python`, package `apk-wireless-import`) | Supplier (package) |
| NODE.BareMetal `oaRustCore` | the Python edge imports `oaRustCore.csv_parser` (optional native wheel) | Customer (Python legacy) |
| POD.workers aggregator | `include:` of `Docker/docker-compose.yml`; network `apk-audio_default` | Pod aggregator |

## Scorecard
| Category | Score (0–3) | Finding |
|---|---|---|
| Coupling & Cohesion | 3 | Builds from its own repository: SDK by tag and pinned git revs in Cargo.lock, image context `SRC/`; no other repository's file compiled or copied |
| Boundary Integrity | 3 | Parser is pure; the wire answer is versioned (`schema`) and its shape is pinned by a test |
| Duplication Control | 3 | The job loop is the runner's; the band rule, status word and envelope are the kernel's, used by all four wireless workers |
| Testing & Contracts | 3 | `SRC/check.sh`: worker, parser fixtures, kernel and both pytest suites; result shape test; DataMigration's frame-header contract test |
| Change management | 2 | Worker 0.2.0, result `schema: 1`; crates not yet tagged (the owner publishes tags) |
Score key: 3 meets the standard · 2 minor gaps · 1 significant gaps · 0 absent / violated.

## Findings
1. **Good: one parser, no side effects.** `SRC/importer/src/lib.rs` `classify_csv_str` is pure; the Python edge refuses to run without the Rust parser.
2. **Good: worker hardening.** Compose runs `read_only`, `cap_drop: [ALL]`, `no-new-privileges`, no port; image runs as uid 10001; oversize jobs are refused (`config.ini` `max_job_bytes`).
3. **Good: topics pinned by test.** `job_and_result_topics_are_this_modules` asserts the exact job/result topic strings from the contracts helpers.
