# Worker-WIRELESS_CSV — the wireless CSV importer as a worker on the bus (PLAN-3423.01).
#   docker compose -f 'PODS/POD.workers/Docker/docker-compose.yml' up -d --build
#   docker build -f Docker/Dockerfile -t worker-wireless-csv:local SRC        # from this repository's root
#
# THE BUILD CONTEXT IS THIS REPOSITORY'S `SRC/` (`context: ../SRC` in the
#    compose file beside this one). Every dependency outside it — runner and
#    contracts from APK.plugin.SDK by tag — is a public git source pinned in
#    Cargo.lock, so `cargo build --locked` fetches exactly those and the image
#    needs nothing else from the monorepo (2026-10-30 audit).

# ---------- build ----------
FROM rust:1.94-bookworm AS build
WORKDIR /src
ENV CARGO_TARGET_DIR=/src/target
COPY . ./SRC

RUN cargo build --release --locked --manifest-path SRC/Cargo.toml --bins \
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
