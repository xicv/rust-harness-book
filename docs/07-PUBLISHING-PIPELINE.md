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

## One source, three reader outputs

```text
book/book.toml
book/src/SUMMARY.md
book/src/**/*.md
        │
        ├── mdBook 0.5.4 → dist/html/ → GitHub Pages
        │
        ├── book-render GitBook backend
        │       ├── expands tested includes
        │       ├── converts only known semantic HTML wrappers
        │       └── dist/gitbook/ → gitbook-publish → GitBook Git Sync
        │
        └── book-render Typst backend
                └── dist/typst/book.typ
                          └── Typst 0.15.1
                                  └── dist/rust-harness-book.pdf
```

`SUMMARY.md` controls order for all three formats. The two `book-render` backends share
the strict SUMMARY parser, include expander, Markdown AST, raw-HTML allowlist,
and project-root containment checks. GitBook is therefore generated output, not
a hand-maintained second prose copy.

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

The project Pages base path is `/rust-harness-book/`, pinned in both
`book/book.toml` and `book/publication.toml`. The publishing workflow uploads
`dist/html/`, whose root contains `index.html` and the chapter downloads.

Pull requests build and verify these files but never deploy them. Only a
successful `main` build can enter the `github-pages` environment, where the
deploy job alone receives `pages: write` and `id-token: write`.

Repository owners must still choose **GitHub Actions** under
**Settings → Pages → Build and deployment**. A configured URL or successful
local build is not live-site evidence; visit the deployed desktop and mobile
pages before recording that claim.

## GitBook Markdown mirror

GitBook renders Markdown from Git Sync; it does not host arbitrary mdBook HTML.
Generate the checked mirror with:

```sh
make gitbook-export
```

The export maps `index.md` to `README.md`, rewrites SUMMARY links, expands all
mdBook includes, normalises fenced-code attributes to their language, turns
learning cards into visible Markdown quotes, copies local images, and rejects
unknown raw HTML. The Introduction keeps a visible link to the authoritative
interactive Pages edition because GitBook does not reproduce the inline Rust
Playground controls.

Generated `dist/gitbook/` is ignored on `main`. After the exact `main` reader
build succeeds, a separate write-scoped job merges that reviewed commit into
`gitbook-publish`, regenerates the mirror, and performs a normal non-force push.
Pull-request jobs never receive that write scope.

GitBook still needs account-side setup:

1. create or choose a GitBook space;
2. enable GitHub Sync for this repository;
3. select `gitbook-publish`;
4. use the repository-root `.gitbook.yaml`, which selects `dist/gitbook`;
5. perform the initial GitHub-to-GitBook sync and publish the space;
6. visit the final URL and compare the Introduction and completed chapters.

GitBook Sync is bidirectional. Treat `dist/gitbook/` as generated and do not edit
it in the GitBook UI; the next verified branch update replaces generated prose.

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
dist/gitbook/
dist/typst/
dist/rust-harness-book.pdf
```

The old manually maintained `typst/book.typ` file has been removed.

## Verified chapter code packs

Each completed chapter has a manifest under `chapter-packs/`. The manifest pins
the full reviewed Git commit, allowlists every historical source path, adds the
current README, verification guide, expected output, and MIT code licence, and
declares shell-free Cargo commands.

```sh
make chapter-packs
```

`chapter-pack` requires the pinned commit to exist locally and to be an ancestor
of the current reviewed tree. It accepts only regular Git blobs, writes sorted
stored ZIP entries with fixed metadata, publishes a SHA-256 sidecar, safely
unpacks the result into a fresh directory, compares every byte, and runs the
declared commands with `CARGO_NET_OFFLINE=true`.

Chapter 0 is exported from
`3fed46defa0189e4e1a8f5b7dc3ab61743209b08`; Chapter 1 is exported from
`decef67c89afba8e4eb095b0c16454e4aca97eb5`. This prevents later chapter code
from leaking backwards into an earlier learner pack.

A passing pack build proves those declared commands and exact outputs worked in
the verification environment. It does not prove every learner machine, target,
or optional tool will behave identically.

## Visual rules

- Selectable code, not screenshots of code
- SVG diagrams where possible
- Alt text for every meaningful figure
- Grayscale-safe meaning
- No meaning conveyed by colour alone
- Small-screen HTML checked before chapter completion
- Print cards expand in normal reading order
- New PDF layouts receive human visual review as chapters are added
