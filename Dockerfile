FROM lukemathwalker/cargo-chef:latest-rust-1.83 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --release --bin actix-testing

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update -y && \
    apt-get install -y --no-install-recommends openssl ca-certificates && \
    apt-get autoremove -y && \
    apt-get clean -y && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/actix-testing actix-testing

RUN groupadd -g 1001 appuser && \
    useradd -u 1001 -g 1001 appuser && \
    chown -R appuser:appuser actix-testing

USER appuser

# Expose port 8080
EXPOSE 8080

ENTRYPOINT ["./actix-testing"]