CONTRACT_NAME = carbonmint_contract
TARGET_DIR    = target/wasm32-unknown-unknown/release
WASM          = $(TARGET_DIR)/$(CONTRACT_NAME).wasm
NETWORK      ?= testnet
SOURCE       ?= default

.PHONY: all build test fmt fmt-check clippy clean deploy optimize

all: build

build:
	cargo build --target wasm32-unknown-unknown --release

test:
	cargo test

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

clippy:
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean

optimize: build
	stellar contract optimize --wasm $(WASM)

deploy: build
	stellar contract deploy \
		--wasm $(WASM) \
		--source $(SOURCE) \
		--network $(NETWORK)
