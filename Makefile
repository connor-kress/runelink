.PHONY: check check-server build build-server test help

help:
	@echo "Available targets:"
	@echo "  check        - Run cargo check on all crates"
	@echo "  check-server - Run cargo check on runelink-server only"
	@echo "  build        - Build all crates"
	@echo "  build-server - Build runelink-server only"
	@echo "  test         - Run tests"

check:
	cargo check

check-server:
	cargo check -p runelink-server

build:
	cargo build

build-server:
	cargo build -p runelink-server

test:
	cargo test
