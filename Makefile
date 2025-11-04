b: build

bt: build test

build:
	@cargo build

test:
	@cargo nextest run --all-features
	@cargo test --doc

.PHONY: build test
