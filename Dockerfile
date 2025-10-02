FROM rust:1.76-slim

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml ./
COPY src/ ./src/

RUN cargo build --release
RUN mkdir -p /data


ENV DATABASE_PATH=/data
EXPOSE 8080

CMD ["./target/release/lsm-btree-db"]
