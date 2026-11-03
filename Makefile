SHELL := /bin/sh
CARGO ?= cargo
PYTHON ?= python3
CONTRACT ?= pirc-economic-settlement

.PHONY: bootstrap fmt fixture check test wasm clean help

help:
	@printf '%s\n' \
	  'bootstrap  Install/check local toolchain prerequisites' \
	  'fmt        Check Rust formatting' \
	  'fixture    Validate shared economic golden vectors' \
	  'check      Run formatting, fixture validation, and locked workspace check' \
	  'test       Run the complete locked Rust workspace test suite' \
	  'wasm       Build the selected Soroban contract for wasm32' \
	  'clean      Remove Cargo build artifacts'

bootstrap:
	@$(CARGO) --version
	@$(PYTHON) --version
	@rustup target list --installed | grep -qx wasm32-unknown-unknown || rustup target add wasm32-unknown-unknown

fmt:
	$(CARGO) fmt --all -- --check

fixture:
	$(PYTHON) scripts/test_economic_vectors.py

check: fmt fixture
	$(CARGO) check --workspace --locked

test: check
	$(CARGO) test --workspace --locked

wasm:
	$(CARGO) build --locked --release --target wasm32-unknown-unknown -p $(CONTRACT)

clean:
	$(CARGO) clean
