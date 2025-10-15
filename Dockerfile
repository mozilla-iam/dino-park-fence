FROM rust:1.69-bullseye

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /root/
COPY --from=0 /usr/src/app/target/release/dino-park-fence .
CMD ["./dino-park-fence"]
