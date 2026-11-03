FROM rust:1.90.0-bookworm

ENV CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1

RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential ca-certificates git pkg-config libssl-dev python3 python3-venv \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add rustfmt clippy \
    && rustup target add wasm32-unknown-unknown

WORKDIR /workspace
COPY . .
RUN cargo fetch --locked

CMD ["make", "check"]
