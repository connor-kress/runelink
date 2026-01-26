.PHONY: check check-server build build-server test help

# Ensure commands work even when rustup has no default toolchain set.
# Users can override, e.g. `make check-server TOOLCHAIN=nightly`.
TOOLCHAIN ?= stable
CARGO ?= cargo

help:
	@echo "Available targets:"
	@echo "  check        - Run cargo check on all crates"
	@echo "  check-server - Run cargo check on runelink-server only"
	@echo "  build        - Build all crates"
	@echo "  build-server - Build runelink-server only"
	@echo "  test         - Run tests"

check:
	$(CARGO) +$(TOOLCHAIN) check

check-server:
	$(CARGO) +$(TOOLCHAIN) check -p runelink-server

build:
	$(CARGO) +$(TOOLCHAIN) build

build-server:
	$(CARGO) +$(TOOLCHAIN) build -p runelink-server

test:
	$(CARGO) +$(TOOLCHAIN) test
