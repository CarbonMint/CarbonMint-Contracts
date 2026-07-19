CONTRACT_NAME = carbonmint_contract
TARGET_DIR    = target/wasm32-unknown-unknown/release
WASM          = $(TARGET_DIR)/$(CONTRACT_NAME).wasm
NETWORK      ?= testnet
SOURCE       ?= default

.PHONY: all build check test fmt fmt-check clippy doc clean deploy optimize wasm-size

all: build

build:
	cargo build --target wasm32-unknown-unknown --release

check:
	cargo check --all-targets

test:
	cargo test

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

clippy:
	cargo clippy --all-targets -- -D warnings

doc:
	cargo doc --no-deps

clean:
	cargo clean

wasm-size: build
	@ls -lh $(WASM) | awk '{ print $$5, $$9 }'

optimize: build
	stellar contract optimize --wasm $(WASM)

deploy: build
	stellar contract deploy \
		--wasm $(WASM) \
		--source $(SOURCE) \
		--network $(NETWORK)

coverage:
	cargo tarpaulin --config tarpaulin.toml --workspace --timeout 120
