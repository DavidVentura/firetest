.PHONY: build
build:
	cd init && cargo build --release && zstd -f target/x86_64-unknown-linux-musl/release/init
	cd firetest && cargo build --release
	mkdir -p dist
	cp firetest/target/release/firetest dist/
