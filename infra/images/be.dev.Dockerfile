## Idea came from: https://dev.to/jorgecastro/hot-reload-in-rust-with-cargo-watch-and-docker-5d25
FROM rust:alpine3.20
WORKDIR /backend

ARG DATABASE_URL="sqlite://data.db"
ENV DATABASE_URL=$DATABASE_URL

RUN apk add --no-cache musl-dev openssl ca-certificates
RUN cargo install cargo-watch
COPY . .
