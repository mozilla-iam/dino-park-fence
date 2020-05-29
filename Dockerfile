FROM rust:latest

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-gnu

FROM debian:10-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /root/
COPY --from=0 /usr/src/app/target/x86_64-unknown-linux-gnu/release/dino-park-fence .
CMD ["./dino-park-fence"]