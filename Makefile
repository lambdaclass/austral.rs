.PHONY: usage build test

usage:
	@echo "Usage:"
	@echo "    build"
	@echo "    test"

build:
	cargo build

test: build
	cargo test
	
