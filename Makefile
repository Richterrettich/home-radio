build-arm:
	cargo build --target armv7-unknown-linux-gnueabihf

image-pi:
	buildah bud --arch arm64 .

image:
	buildah bud -t home-radio .