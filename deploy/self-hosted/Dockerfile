FROM rust:1.90-bookworm AS builder
WORKDIR /source
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps/server ./apps/server
RUN cargo build --release --locked -p seven-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home --uid 10001 seven
COPY --from=builder /source/target/release/seven-server /usr/local/bin/seven-server
USER seven
ENV SEVEN_BIND=127.0.0.1:43821
ENTRYPOINT ["/usr/local/bin/seven-server"]
