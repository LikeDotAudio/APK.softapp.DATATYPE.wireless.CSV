# Worker-WIRELESS_CSV — the wireless CSV importer as a worker on the bus (PLAN-3423.01).
#   docker compose -f 'PODS/POD.workers/Docker/docker-compose.yml' up -d --build
#   docker build -f 'PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.CSV/Docker/Dockerfile' -t worker-wireless-csv:local .
#
# ⚠️ THE BUILD CONTEXT IS THE REPO ROOT (`context: ../../../../../..` in the
#    compose file beside this one), because the worker links two crates that
#    live outside this repository: apkaudio-contracts and apk-plugin-runner.
#    Every COPY source below is a REAL repo-root path (PODS/README.md rule 1),
#    and <repo>/.dockerignore is what trims the context. That file drops every
#    Softapps *.py (PLAN-3422.01), which is one reason this worker is Rust: the
#    importer it runs is `csv_importer.rs`, and the Python edge in
#    IMPORT:file:CSV/ refuses to run without a Rust parser anyway.

# ---------- build ----------
FROM rust:1.94-bookworm AS build
WORKDIR /src
ENV CARGO_TARGET_DIR=/src/target
COPY rust-toolchain.toml ./

# THE DESTINATION MIRRORS THE CHECKOUT, so the crate's relative path
# dependencies resolve here as they do on disk. Contracts are reached through
# NODE.BareMetal/SRC/contracts, which in a checkout is a SYMLINK to
# <repo>/contracts; COPY would carry the link pointing out of the image, so the
# two real directories land where the link names (same move as
# DISCOVERY.Engine/Docker/Dockerfile).
COPY ["contracts/rust",   "./PODS/POD.THICK/NODE.BareMetal/SRC/contracts/rust"]
COPY ["contracts/tokens", "./PODS/POD.THICK/NODE.BareMetal/SRC/contracts/tokens"]
# The runner, and the workspace manifest it inherits `edition` from. Only the
# manifest: cargo reads the root for inheritance and does not load its members
# for a crate that is not one of them.
COPY ["PODS/POD.THICK/DISCOVERY.Engine/SRC/Cargo.toml", "./PODS/POD.THICK/DISCOVERY.Engine/SRC/Cargo.toml"]
COPY ["PODS/POD.THICK/DISCOVERY.Engine/SRC/RUNNER",     "./PODS/POD.THICK/DISCOVERY.Engine/SRC/RUNNER"]
# The tool: its parser at the repository root, and the worker crate in SRC/.
ARG TOOL=PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.CSV
COPY ["PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.CSV/csv_importer.rs", "./PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.CSV/csv_importer.rs"]
COPY ["PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.CSV/SRC",             "./PODS/POD.THIN/Softapps/DATATYPE/DATATYPE.wireless.CSV/SRC"]

RUN cargo build --release --locked --manifest-path "$TOOL/SRC/Cargo.toml" --bins \
 && mkdir -p /out \
 && cp target/release/apk-worker-wireless-csv target/release/apk-plugin-health /out/

# ---------- runtime ----------
FROM debian:bookworm-slim
# Not root: the worker opens no port and touches no device, so it needs no
# privilege at all; compose drops every capability and mounts / read-only.
RUN useradd --uid 10001 --no-create-home --shell /usr/sbin/nologin worker
COPY --from=build /out/ /usr/local/bin/
USER worker
ENTRYPOINT ["apk-worker-wireless-csv"]
