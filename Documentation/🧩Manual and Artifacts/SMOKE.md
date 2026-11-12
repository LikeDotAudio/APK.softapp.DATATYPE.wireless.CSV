# Smoke test — DATATYPE.wireless.CSV

Is `DATATYPE.wireless.CSV` deployed? Written by `.apk.scripts/smoke.py` from the module's own shape; what each check means is in that script's header.

← [Wiki home](../README.md)

**❌ FAIL** — 2/3 checks, since 2026-11-12 00:16 EST

| Check | Target | Result | Detail |
|---|---|---|---|
| pushed | [APK.softapp.DATATYPE.wireless.CSV](https://github.com/LikeDotAudio/APK.softapp.DATATYPE.wireless.CSV.git) | ❌ | 1 commit(s) not pushed |
| container | `Worker-WIRELESS_CSV` | ✅ | running, healthy |
| web | [SRC/importer/src/lib.rs](https://apk.audio/APK%3ASoftapps/DATATYPE/DATATYPE.wireless.CSV/SRC/importer/src/lib.rs) | ✅ | HTTP 200 |

Re-run: `python3 .apk.scripts/smoke.py PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.CSV`

<!-- smoke:sig ee08b7ee7884 since 2026-11-12 00:16 EST -->
