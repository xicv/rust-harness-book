# Book-render implementation receipt / 书籍渲染器实现凭证

Reviewed: **2026-08-25**

## Result

`book-render` converts the canonical mdBook Markdown source into a generated
Typst project, and Typst compiles the full book PDF. The old manually maintained
`typst/book.typ` file has been removed.

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

The first renderer contract failed for the intended reason: the placeholder
returned the Markdown unchanged and did not emit the required `#card(` Typst
structure.

### Green

First complete repository pass:

```text
2e31c4f3ae3d4fcef7f515b8492e952a9f908f2f
```

GitHub Actions run:

```text
32811487873
```

That run passed:

- scaffold validation;
- check-only Rust formatting;
- workspace check and tests;
- Clippy with warnings denied;
- mdBook example tests;
- production HTML build;
- canonical Markdown-to-Typst generation;
- full Typst PDF compilation.

The main implementation began in
`eadaa4d385f0d1131987b4e983bb95326717fc24`; subsequent focused commits fixed
formatting, test expectations, escaping, Clippy findings, and Typst locale
metadata. No project-wide formatting command or automatic fix command was run.

## Canonical-source contract

- `book/src/` remains the only editable prose source.
- `SUMMARY.md` controls publication order.
- Rust examples stay in tested source files and are expanded from mdBook include
  directives.
- `dist/typst/book.typ` is generated and ignored.
- `typst/template.typ` contains presentation rules, not chapter prose.
- HTML and PDF therefore use the same chapter content.

## Preserved structures

The renderer covers the structures currently used by the book:

- parts, chapters, and headings;
- paragraphs and thematic breaks;
- learning cards and card labels;
- strong text, emphasis, strikethrough, inline code, links, and bilingual term
  spans;
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
- the Typst template;
- the generated-output parent directory.

It rejects repository escapes, recursive includes, missing or duplicated
anchors, unsupported HTML, malformed structures, and remote images. User,
model, and book text remain data: the renderer does not execute a shell command,
run included code, call a model, or fetch a network asset.

Generated output is prepared in a staging directory and replaces the prior
output only after the complete source render succeeds.

## PDF content parity

**PDF content parity: pass for completed Chapters 0 and 1.**

The PDF target compiles `dist/typst/book.typ`, generated from the same Markdown
chapters used by mdBook. The previous title-page-only output is not counted as
parity.

The two chapter receipts now record:

```toml
[verification]
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
  not a promise to implement every CommonMark feature or arbitrary HTML.
- Unsupported structures fail closed and require a new Red test before support
  is added.
- Heading-fragment links are not yet supported in PDF output.
- Remote images must be vendored and rights-reviewed before publication.
- PDF visual quality still needs chapter-by-chapter human review as the book
  grows.
- The renderer has no network, shell, model, or code-execution capability.
