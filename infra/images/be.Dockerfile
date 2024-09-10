## Idea came from: https://dev.to/rogertorres/first-steps-with-docker-rust-30oi
FROM rust:slim-bookworm AS build

RUN USER=root cargo new --bin backend
WORKDIR /tmp/app

## TODO: [QUESTION] It is needed in build time?
ARG DB
ENV DATABASE_URL=$DB

## TODO: [OPTIMIZE] Should check which one is necessary and which one is not
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    ca-certificates \
    openssl \
    tzdata \
    pkg-config \
    gcc

COPY . .
RUN cargo build --release --bin backend

## TODO: [RESEARCH] Should we use alpine?: https://andygrove.io/2020/05/why-musl-extremely-slow/
FROM rust:slim-bookworm AS runtime

WORKDIR /app
COPY --from=build /tmp/app/target/release/backend /app/backend

ENV RUST_BACKTRACE=1
ENV DATABASE_URL="sqlite://data.db"

ENTRYPOINT ["/app/backend"]
