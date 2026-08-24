.PHONY: scaffold-check check test lint fmt-check book-test book-build pdf verify clean

scaffold-check:
	python3 scripts/check_scaffold.py

check:
	cargo check --workspace

test:
	cargo test --workspace

lint:
	cargo clippy --workspace --all-targets -- -D warnings

fmt-check:
	cargo fmt --all -- --check

book-test:
	mdbook test book -L target/debug/deps

book-build:
	mdbook build book

pdf:
	mkdir -p dist
	typst compile typst/book.typ dist/rust-harness-book.pdf

verify: scaffold-check fmt-check check test lint book-test book-build pdf

clean:
	rm -rf target dist
