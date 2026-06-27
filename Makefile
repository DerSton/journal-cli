.PHONY: all fmt check clippy test

# Run all CI checks (default target)
# Run with 'make -j' to execute them in parallel (note: cargo might lock target directory temporarily)
all: fmt check clippy test

fmt:
	cargo fmt --all -- --check

check:
	cargo check

clippy:
	cargo clippy --all-targets -- -D warnings

test:
	cargo test
