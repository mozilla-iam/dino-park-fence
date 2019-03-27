FROM rust:latest

RUN rustup target add x86_64-unknown-linux-musl
 RUN apt-get update && apt-get install -y \
    musl-tools \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest  

WORKDIR /root/
COPY --from=0 /usr/src/app/target/x86_64-unknown-linux-musl/release/dino-park-fence .
CMD ["./dino-park-fence"]