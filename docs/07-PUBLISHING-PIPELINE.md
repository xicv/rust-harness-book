# Publishing pipeline / 出版流程

## Pinned renderers

- mdBook 0.5.4 for production HTML
- Typst 0.15.1 for production PDF

The exact versions are recorded in `book/sources.lock.toml` and CI.

## Canonical source

Canonical Markdown under `book/src/` is the editorial source of truth. Tested Rust source
lives under `crates/` and is included into chapters by path and anchor.

Do not maintain a second prose copy in Typst. `typst/template.typ` contains
presentation rules only. `dist/typst/book.typ` is generated output.

## One source, two outputs

```text
book/book.toml
book/src/SUMMARY.md
book/src/**/*.md
        │
        ├── mdBook 0.5.4
        │       └── dist/html/
        │
        └── book-render
                ├── expands tested includes
                ├── copies reviewed local assets
                └── dist/typst/book.typ
                          │
                          └── Typst 0.15.1
                                  └── dist/rust-harness-book.pdf
```

`SUMMARY.md` controls publication order for both formats.

## HTML

mdBook is the production HTML renderer because it provides:

- tested Rust code blocks;
- file and anchor includes;
- search;
- themes;
- print view;
- a static, easy-to-host output.

The custom CSS is mobile-first. Cards stack vertically by default. No important
content may depend on a swipe gesture or JavaScript.

Build it with:

```sh
mdbook test book -L target/debug/deps
mdbook build book
```

## PDF

`book-render` is a standard-library-only Rust crate. It reads the canonical book
configuration and Markdown, then emits a generated Typst project.

```sh
cargo run -p book-render --locked -- \
  --book book \
  --template typst/template.typ \
  --output-dir dist/typst

typst compile dist/typst/book.typ dist/rust-harness-book.pdf
```

Or run:

```sh
make pdf
```

The renderer currently preserves the structures used by this book:

- parts, chapters, and headings;
- paragraphs, thematic breaks, quotes, lists, and tables;
- learning cards and card labels;
- strong text, emphasis, strikethrough, inline code, and bilingual term spans;
- external links and local chapter links;
- fenced code blocks with language metadata;
- standalone local images and captions;
- mdBook full-file, anchor, and line-range includes.

## Includes and code truth

Both `{{#include ...}}` and `{{#rustdoc_include ...}}` are expanded relative to
the containing source file. The renderer supports:

- complete files;
- named `ANCHOR` / `ANCHOR_END` regions;
- one-based line selections and ranges;
- nested includes with a bounded depth.

Recursive includes, missing or duplicated anchors, and malformed selectors are
errors. This keeps book code tied to the same files compiled and tested by Cargo.

## Assets

Only local assets are accepted. The renderer canonicalises each image path,
confirms it stays inside the project root, copies it into `dist/typst/assets/`,
and reuses one generated asset for repeated references.

Remote images fail closed. They must first be vendored, rights-reviewed, given
meaningful alt text, and added to source control.

## Trust boundary

The renderer does not execute code, invoke a shell, access a model, or fetch the
network. Input text is escaped before becoming Typst source.

It fails closed for:

- unsupported block or inline HTML;
- source, include, template, or image paths outside the project root;
- recursive includes;
- remote images;
- malformed tables, links, cards, fences, or metadata;
- chapter sources that do not match `SUMMARY.md`.

A newly needed Markdown feature starts with a focused failing test. Silently
removing content is never an acceptable fallback.

## Generated-output rule

These paths are build output and must not be edited or committed:

```text
dist/html/
dist/typst/
dist/rust-harness-book.pdf
```

The old manually maintained `typst/book.typ` file has been removed.

## Visual rules

- Selectable code, not screenshots of code
- SVG diagrams where possible
- Alt text for every meaningful figure
- Grayscale-safe meaning
- No meaning conveyed by colour alone
- Small-screen HTML checked before chapter completion
- Print cards expand in normal reading order
- New PDF layouts receive human visual review as chapters are added
