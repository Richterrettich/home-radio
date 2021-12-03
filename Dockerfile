FROM docker.io/rust:1.57-slim as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:11
RUN apt update && apt install -y vlc && apt clean
COPY --from=builder /usr/src/app/target/release/home-radio /usr/local/bin/home-radio
ENTRYPOINT ["/usr/local/bin/home-radio"]
CMD ["serve","--dir","/var/lib/home-radio"]
