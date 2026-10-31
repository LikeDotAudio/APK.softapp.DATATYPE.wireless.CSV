# 🛣 Improvements — DATATYPE.wireless.CSV (from the 2026-10-30 architecture audit)

What this repo should change to meet the standard, most valuable first. Source: [🔍Audits/ARCHITECTURE-AUDIT.md](../%F0%9F%94%8DAudits/ARCHITECTURE-AUDIT.md).
← [Roadmap](INDEX.md)

| # | Improvement | Category | Effort (S/M/L) | Evidence |
|---|---|---|---|---|
| 1 | Build the image from this repository's own context — Needs owner: publish an SDK tag that carries `apk_plugin_runner::jobs`, then drop the four COPYs and two `--config` patches | Coupling & Cohesion | S | `Docker/Dockerfile`; compose `context: ../../../../../..` |

Delivered items: see [🌍Publishing/DELIVERY.md](../%F0%9F%8C%8DPublishing/DELIVERY.md).
