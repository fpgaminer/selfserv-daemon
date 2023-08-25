FROM rust:1.72.0 as builder

WORKDIR /usr/src/selfserv-daemon
COPY src ./src
COPY Cargo.* ./


RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/selfserv-daemon /usr/local/bin/selfserv-daemon

LABEL org.opencontainers.image.source=https://github.com/fpgaminer/selfserv-daemon

ENTRYPOINT ["selfserv-daemon"]