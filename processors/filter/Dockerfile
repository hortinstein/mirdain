FROM rust:1.77-slim as builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/mirdain-filter /usr/local/bin/processor
ENV PORT=3000
EXPOSE 3000
CMD ["processor"]
