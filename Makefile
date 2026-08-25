.PHONY: scaffold-check check test lint fmt-check book-test book-build book-render pdf verify clean

scaffold-check:
	python3 scripts/check_scaffold.py

check:
	cargo check --workspace --locked

test:
	cargo test --workspace --locked

lint:
	cargo clippy --workspace --all-targets --locked -- -D warnings

fmt-check:
	cargo fmt --all -- --check

book-test:
	mdbook test book -L target/debug/deps

book-build:
	mdbook build book

book-render:
	cargo run -p book-render --locked -- --book book --template typst/template.typ --output-dir dist/typst

pdf: book-render
	mkdir -p dist
	typst compile dist/typst/book.typ dist/rust-harness-book.pdf

verify: scaffold-check fmt-check check test lint book-test book-build pdf

clean:
	rm -rf target dist
