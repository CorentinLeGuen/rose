# builder
FROM rust:1.91 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# run
FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/target/release/rose /usr/local/bin/rose

EXPOSE 8003

CMD ["rose"]