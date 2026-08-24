# Publishing pipeline / 出版流程

## Pinned renderers

- mdBook 0.5.4 for production HTML
- Typst 0.15.1 for PDF

The exact versions are recorded in `book/sources.lock.toml` and CI.

## Canonical source

Markdown under `book/src/` is the editorial source of truth.
Tested Rust source lives under `crates/` and is included into chapters by path
and anchor.

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

## PDF

Typst is the PDF renderer. The intended long-term pipeline is:

```text
Markdown + metadata + tested source
                 ↓
          book-render tool
          ↙             ↘
      mdBook HTML      Typst source → PDF
```

The current repository contains a buildable PDF shell only. The `book-render`
tool has not been implemented, so the project does not yet claim automatic
full-book Markdown-to-Typst conversion.

Do not manually maintain a second prose copy of every chapter in Typst.

## Visual rules

- Selectable code, not screenshots of code
- SVG diagrams where possible
- Alt text for every meaningful figure
- Grayscale-safe meaning
- No meaning conveyed by colour alone
- Small-screen layout checked before chapter completion
- Print cards expand in normal reading order
