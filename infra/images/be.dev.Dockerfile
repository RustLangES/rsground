FROM rust:slim-bookworm

ENV DATABASE_URL="sqlite://data.db"

## RUN already updates packages https://app.deepsource.com/directory/analyzers/docker/issues/DOK-DL3009
RUN apt-get install --no-install-recommends -y \
    build-essential \
    libssl-dev \
    ca-certificates \
    openssl \
    tzdata \
    pkg-config \
    gcc &&\
	cargo install cargo-watch

WORKDIR /backend
COPY . .
