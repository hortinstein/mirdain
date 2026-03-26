FROM rust:1.77-slim as builder
WORKDIR /app
COPY Cargo.toml ./
COPY backend/Cargo.toml ./backend/
# Cache deps
RUN mkdir -p backend/src && echo "fn main(){}" > backend/src/main.rs
RUN cargo build --release --manifest-path backend/Cargo.toml 2>/dev/null || true
# Real build
COPY backend/src ./backend/src
RUN touch backend/src/main.rs && cargo build --release --manifest-path backend/Cargo.toml

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/backend/target/release/mirdain-backend /usr/local/bin/mirdain
ENV RUST_LOG=mirdain=info
ENV BIND_ADDR=0.0.0.0:3001
ENV DATA_PATH=/data/pipelines.json
EXPOSE 3001
CMD ["mirdain"]
