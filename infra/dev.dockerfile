FROM rust:1.81.0-slim

RUN apt-get update \
&& apt-get install -y --no-install-recommends \
nodejs=18.19.0+dfsg-6~deb12u2 \
musl-dev=1.2.3-1 \
build-essential=12.9 \
npm=9.2.0~ds1-1 \
libssl3=3.0.14-1~deb12u2 \
openssl=3.0.14-1~deb12u2 \
pkg-config=1.8.1-1 \
git=1:2.39.5-0+deb12u1 \
&& apt-get clean \
&& rm -rf /var/lib/apt/lists/*

RUN apt-get update \
&& apt-get install -y --no-install-recommends \
ca-certificates=20230311 \
curl=7.88.1-10+deb12u7 \
&& apt-get clean \
&& rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://get.docker.com -o get-docker.sh \
&& sh get-docker.sh

RUN groupadd rsground \
&& useradd -m -g rsground rsground \
&& usermod -aG docker rsground

ARG USERNAME
RUN groupadd -r $USERNAME && useradd -r -g $USERNAME $USERNAME
RUN usermod -aG docker "$USERNAME"

USER $USERNAME

RUN cargo install cargo-watch

WORKDIR /app
