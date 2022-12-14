FROM rustlang/rust:nightly as builder
WORKDIR /usr/src/bb-proxy
COPY . .
RUN cargo install --features vendored --path .


FROM debian:buster-slim
RUN apt-get update -y && apt-get upgrade -y && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/bb-proxy /usr/local/bin/bb-proxy
CMD ["bb-proxy"]
