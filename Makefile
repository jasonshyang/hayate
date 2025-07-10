all: build

build:
	cargo build

release:
	cargo build --release

run:
	cargo run

test:
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

doc:
	cargo doc --open

clean:
	cargo clean