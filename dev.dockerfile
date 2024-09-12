FROM rust:1.81.0-alpine3.20

RUN apk add --no-cache --update \
docker=26.1.5-r0 \
nodejs-current=21.7.3-r0 \
musl-dev=1.2.5-r0 \
build-base=0.5-r3 \
npm=10.8.0-r0 \
libssl3=3.3.2-r0 \
openssl-libs-static=3.3.2-r0 \
&& rm -rf /var/cache/apk/*

COPY ./.build-context/cargo-watch /usr/local/cargo/bin/cargo-watch

RUN addgroup -S rsground \
	&& adduser -S rsground -G rsground \
	&& addgroup rsground docker

USER rsground

WORKDIR /app
