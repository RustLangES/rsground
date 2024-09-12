FROM rust:1.81.0-slim

RUN apt-get update \
&& apt-get install -y --no-install-recommends \
nodejs=18.19.0+dfsg-6~deb12u2 \
musl-dev=1.2.3-1 \
build-essential=12.9 \
npm=9.2.0~ds1-1 \
bash=5.2.15-2+b7 \
libssl3=3.0.14-1~deb12u2 \
openssl=3.0.14-1~deb12u2 \
pkg-config=1.8.1-1 \
&& apt-get clean \
&& rm -rf /var/lib/apt/lists/*

RUN apt-get update \
&& apt-get install -y --no-install-recommends \
ca-certificates=20230311 \
curl=7.88.1-10+deb12u7 \
&& install -m 0755 -d /etc/apt/keyrings \
&& curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc \
&& chmod a+r /etc/apt/keyrings/docker.asc \
&& apt-get clean \
&& rm -rf /var/lib/apt/lists/*

RUN echo \
"deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian \
$(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
tee /etc/apt/sources.list.d/docker.list > /dev/null

RUN apt-get update \
&& apt-get install -y --no-install-recommends \
docker-ce=5:27.1.1-1~debian.12~bookworm \
docker-ce-cli=5:27.1.1-1~debian.12~bookworm \
containerd.io \
docker-buildx-plugin \
docker-compose-plugin \
&& apt-get clean \
&& rm -rf /var/lib/apt/lists/*

RUN groupadd rsground \
&& useradd -m -g rsground rsground \
&& usermod -aG docker rsground

## TODO: start docker daemon, PD: this does not work.

## RUN dockerd --host=unix///var/run/docker.sock &

USER rsground

RUN cargo install cargo-watch

WORKDIR /app
