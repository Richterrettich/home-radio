build-arm:
	cargo build --target armv7-unknown-linux-gnueabihf

image-pi:
	podman run -it --rm --arch arm64 -w /usr/src/app -v ${PWD}:/usr/src/app:z -v ${PWD}/cargo-cache:/usr/local/cargo/registry:z docker.io/rust:1.56.1-slim cargo build --release --target-dir=/usr/src/app/target/arm-build
 
image:
	buildah bud -t home-radio .