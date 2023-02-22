./target/release/wleave: src/**.rs
	cargo build --frozen --release --all-features

.PHONY: wleave
wleave: ./target/release/wleave

.PHONY: all
all: wleave

.PHONY: clean
clean:
	rm -rf ./target
