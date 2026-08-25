# Book-render implementation receipt / 书籍渲染器实现凭证

Reviewed: **2026-08-25**

## Result

`book-render` now converts the canonical mdBook Markdown source into a generated
Typst project and compiles the full book PDF. The old manually maintained
`typst/book.typ` smoke shell has been removed.

```text
book/book.toml
book/src/SUMMARY.md
book/src/**/*.md
        │
        ▼
crates/book-render
        │
        ├── dist/typst/book.typ
        ├── dist/typst/template.typ
        └── dist/typst/assets/**
                │
                ▼
        dist/rust-harness-book.pdf
```

## TDD evidence

### Red

Commit:

```text
cb3d8edd70e976a2dac8c53617625e0c6b6d85ba
```

GitHub Actions run:

```text
32795217992
```

The renderer contract test failed for the intended reason: the initial
placeholder did not preserve a learning card and therefore did not emit
`#card(`.

### Green

Initial Green implementation:

```text
c399ac25e564989159ef24d10a847bd922f3b202
```

The Green implementation added the renderer library, CLI, complete-book fixture,
path-containment regression, generated Typst template, Makefile integration, and
CI integration. Subsequent commits tightened formatting, parser behaviour,
documentation, and full-book compilation without rewriting unrelated files.

## Canonical-source contract

- `book/src/` remains the only editable prose source.
- `SUMMARY.md` controls publication order.
- Rust examples stay in tested source files and are expanded from mdBook include
  directives.
- `dist/typst/book.typ` is generated and ignored.
- `typst/template.typ` contains presentation rules, not chapter prose.
- HTML and PDF therefore use the same chapter content.

## Preserved structures

The implemented renderer covers the structures currently used by the book:

- parts, chapters, and headings;
- paragraphs and thematic breaks;
- learning cards and card labels;
- strong text, emphasis, inline code, links, and bilingual term spans;
- fenced source blocks with language names;
- unordered, ordered, and nested lists;
- block quotes;
- tables;
- local chapter links;
- standalone local images and captions;
- mdBook full-file, anchor, and line-range includes.

## Path containment and trust boundaries

Path containment is tested as a first-class invariant.

The renderer canonicalises and validates:

- the mdBook source directory;
- every chapter selected by `SUMMARY.md`;
- every included source file;
- every copied local image;
- the Typst template.

It rejects repository escapes, recursive includes, missing or duplicated anchors,
unsupported block HTML, malformed block structures, and remote images. Model or
user text is data; it never becomes an instruction to execute a shell command or
fetch a network asset.

## PDF content parity

**PDF content parity: pass for completed Chapters 0 and 1.**

The PDF target now compiles `dist/typst/book.typ`, which is generated from the
same Markdown chapters used by mdBook. The earlier title-page-only smoke build is
not counted as parity.

The two chapter receipts record:

```toml
book_render = "pass"
pdf_content_parity = "pass"
```

## Verification gates

The repository-level gate requires:

```sh
python3 scripts/check_scaffold.py
python3 scripts/check_book_render.py
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
mdbook test book -L target/debug/deps
mdbook build book
cargo run -p book-render --locked -- \
  --book book \
  --template typst/template.typ \
  --output-dir dist/typst
typst compile dist/typst/book.typ dist/rust-harness-book.pdf
```

`make verify` runs the complete check-only sequence.

## Honest limits

- This is a purpose-built renderer for the book's documented Markdown contract,
  not a promise to implement every CommonMark or arbitrary HTML extension.
- Unsupported structures fail closed and require a new Red test before support is
  added.
- Remote images must be vendored and rights-reviewed before publication.
- PDF visual quality still needs chapter-by-chapter human review as the book grows.
- The renderer does not execute code, call a model, or access the network.
