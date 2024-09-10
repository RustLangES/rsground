FROM rust:slim-bookworm AS build

RUN USER=root cargo new --bin backend
WORKDIR /tmp/app

## TODO: [OPTIMIZE] Should check which one is necessary and which one is not
RUN apt-get update && apt-get install --no-install-recommends -y \
    build-essential \
    libssl-dev \
    ca-certificates \
    openssl \
    tzdata \
    pkg-config \
    gcc

COPY . .
## This can be hardcoded.
ENV DATABASE_URL="sqlite://data.db"
RUN cargo build --release --bin backend

## TODO: [RESEARCH] Should we use alpine?: https://andygrove.io/2020/05/why-musl-extremely-slow/
FROM rust:slim-bookworm AS runtime

WORKDIR /app
COPY --from=build /tmp/app/target/release/backend /app/backend

ENTRYPOINT ["/app/backend"]
