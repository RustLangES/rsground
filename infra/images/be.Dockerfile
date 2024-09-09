## Idea came from: https://dev.to/rogertorres/first-steps-with-docker-rust-30oi
FROM rust:slim-bookworm AS build

RUN USER=root cargo new --bin backend
WORKDIR /tmp/app

## TODO: [QUESTION] It is needed in build time?
ARG DATABASE_URL="sqlite://data.db"
ENV DATABASE_URL=$DATABASE_URL

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
FROM rust:alpine3.20 AS runtime
RUN addgroup -S rust && adduser -S rust -G rust

WORKDIR /app

COPY --from=build /tmp/app/target/release/backend /app/backend
RUN chown -R rust:rust /app
USER rust
ENTRYPOINT ["/app/backend"]
