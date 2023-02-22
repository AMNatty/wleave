./target/release/wleave: src/**.rs
	cargo build --frozen --release --all-features

.PHONY:
install: ./target/release/wleave
	install -Dm0755 -t "${DESTDIR}/bin" "./target/release/wleave"

.PHONY: wleave
wleave: ./target/release/wleave

.PHONY: all
all: wleave

.PHONY: clean
clean:
	rm -rf ./target
