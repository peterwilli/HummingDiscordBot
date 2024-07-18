FROM rust:1.75.0-slim-buster as rust_builder
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    pkg-config libssl-dev perl make git && \
    rm -rf /var/lib/apt/lists/*

COPY ./ /src
RUN cargo install --path /src

FROM debian:buster-slim
COPY --from=rust_builder /usr/local/cargo/bin/MDHBot /usr/local/bin/MDHBot
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    dumb-init chromium ca-certificates && \
    rm -rf /var/lib/apt/lists/*
ENV CHROMIUM_FLAGS="--disable-gpu --headless --no-sandbox  --remote-debugging-address=0.0.0.0  --remote-debugging-port=9222 --user-data-dir=/data"
WORKDIR /storage
ENTRYPOINT ["/usr/bin/dumb-init", "--", "/usr/local/bin/MDHBot"]
