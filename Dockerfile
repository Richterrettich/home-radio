FROM docker.io/rust:1.56-slim as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM fedora:35
COPY --from=builder /usr/src/app/target/release/home-radio /usr/local/bin/home-radio
ENTRYPOINT ["/usr/local/bin/home-radio"]
CMD ["/var/lib/home-radio"]
