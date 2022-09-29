FROM rust:1.62.1 as builder
WORKDIR /usr/src/bb-proxy
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/bb-proxy /usr/local/bin/bb-proxy
CMD ["bb-proxy"]