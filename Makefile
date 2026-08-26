.PHONY: scaffold-check check test lint fmt-check book-test book-build book-render pdf chapter-packs gitbook-export reader-delivery verify clean

.NOTPARALLEL: reader-delivery

scaffold-check:
	python3 scripts/check_scaffold.py
	python3 scripts/check_book_render.py
	python3 scripts/check_interactive_labs.py
	python3 scripts/check_reader_delivery.py

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

chapter-packs:
	mkdir -p dist/html/downloads/ch00 dist/html/downloads/ch01
	cargo run -p chapter-pack --locked -- --manifest chapter-packs/ch00.toml --output-dir dist/html/downloads/ch00
	cargo run -p chapter-pack --locked -- --manifest chapter-packs/ch01.toml --output-dir dist/html/downloads/ch01

gitbook-export:
	cargo run -p book-render --bin gitbook-export --locked -- --book book --output-dir dist/gitbook

reader-delivery: book-build chapter-packs gitbook-export
	python3 scripts/check_reader_delivery.py --artifacts

book-render:
	cargo run -p book-render --locked -- --book book --template typst/template.typ --output-dir dist/typst

pdf: book-render
	mkdir -p dist
	typst compile dist/typst/book.typ dist/rust-harness-book.pdf

verify: scaffold-check fmt-check check test lint book-test reader-delivery pdf

clean:
	rm -rf target dist
