FROM rust:1.81.0-alpine3.20

# should we lock versions?
RUN apk add --no-cache --update \
docker bash curl python3 nodejs-current \
musl-dev build-base gcc npm libssl3 openssl-libs-static \
&& cargo install cargo-watch

WORKDIR /app

