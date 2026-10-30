# Delivery — DATATYPE.wireless.CSV

← [Wiki home](../README.md)

This repo ships in three places, none with a page of its own (no live URL):

| What | Built by | Lands |
|---|---|---|
| `csv_importer.rs` in the **orchestrator** | `MANAGEMENT.DataMigration` compiles it by `#[path]`; `oaRustCore` (NODE.BareMetal `SRC/Core`) re-exports that crate; `apkaudio-orchestrator` links it. `NODE.BareMetal/Docker/Dockerfile` copies `PODS/POD.THIN/Softapps` so the path resolves. | the **Node-BareMetal** container (`node-baremetal:local`); rebuild with `up-stack.sh NODE.BareMetal` |
| the **worker** `apk-worker-wireless-csv` | `Docker/Dockerfile` (repo-root context; links `apkaudio-contracts` and `apk-plugin-runner`); runtime `debian:bookworm-slim`, user `worker` | container **Worker-WIRELESS_CSV**, project `workers`, via `docker compose -f PODS/POD.workers/Docker/docker-compose.yml up -d --build`; healthcheck `apk-plugin-health` |
| the files themselves | `PODS/POD.THIN/Softapps` copied into **WEB.Frontend-Assets**; the monorepo's FTPS job (`.github/workflows/ci.yml`) uploads the checkout to apk.audio | nothing there opens it — `display.json`'s `entry` (`main.py`) is not a file here |

Remote: `https://github.com/LikeDotAudio/APK.softapp.DATATYPE.wireless.CSV.git` (no workflow of its own).

**Tested:** by hand — `cargo test` in `SRC/` (7 passed 2026-10-30), `cargo test --lib` in MANAGEMENT.DataMigration, `pytest` in `IMPORT:file:CSV/` (5 passed with the `oaRustCore` wheel, 2 + 3 skipped without). The worker's self-test runs a five-row CSV through `handle_job` at start-up. `.apk.scripts/smoke.py` checks only that HEAD is pushed ([SMOKE](../%F0%9F%A7%A9Manual%20and%20Artifacts/SMOKE.md)).

**Not automated:** no CI runs any of these tests; the `oaRustCore` wheel the Python edge needs is built by hand (`maturin build` in `NODE.BareMetal/SRC/Core`); `test_csv_pattern_importer` publishes retained to the live broker, so it is not safe to run on a rig in use.
