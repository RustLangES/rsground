## Idea came from: https://dev.to/jorgecastro/hot-reload-in-rust-with-cargo-watch-and-docker-5d25
FROM rust:slim-bookworm

ARG DATABASE_URL="sqlite://data.db"
ENV DATABASE_URL=$DATABASE_URL

RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    ca-certificates \
    openssl \
    tzdata \
    pkg-config \
    gcc
RUN cargo install cargo-watch

ENV RUST_BACKTRACE=full
WORKDIR /backend
COPY . .
