./target/release/wleave: src/**.rs
	cargo build --frozen --release --all-features

.PHONY: wleave
wleave: ./target/release/wleave

.PHONY: completions
completions: wleave
	mkdir -p completions
	OUT_DIR=completions cargo run --package wleave_completions --bin wleave_completions

.PHONY: all
all: wleave

.PHONY: clean
clean:
	rm -rf ./target ./completions_generated
