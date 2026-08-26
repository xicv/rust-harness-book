# Interactive labs / 互动实验

## Purpose / 目的

Readers should be able to change a small Rust example and see the result while
the idea is still fresh.

The book uses three different lab levels. They solve different problems and must
not be presented as interchangeable.

## Level 1 — Inline Playground / 书内即时实验

Use an editable mdBook Rust block when the idea can be expressed as one
self-contained file.

The HTML book sends the code to the public Rust Playground and displays the
result below the block. The reader can edit, run, and undo changes without
installing Rust.

Authoring form:

````markdown
```rust,editable,edition2024
{{#rustdoc_include _labs/example.rs:example}}
```
````

Rules:

- keep the example focused on one idea;
- prefer the standard library;
- keep a real `fn main()` so the code is easy to understand outside mdBook;
- source the block from a checked file rather than duplicating it in prose;
- run it through `mdbook test`;
- state when it is a simplified lab rather than production Harness code;
- preserve a useful expected output or assertion;
- do not make browser execution the only way to complete a chapter.

## Level 2 — Project lab / 完整工程实验

Use the real Cargo workspace when the exercise needs:

- local crates;
- multiple files;
- integration tests;
- command-line arguments;
- file-system access;
- subprocesses;
- exact dependency versions;
- the pinned Rust toolchain;
- the complete Agent Harness.

The normal path is local Cargo. A browser cloud environment such as GitHub
Codespaces may later provide the same repository and commands for readers who
cannot install the toolchain locally.

Project labs remain the source of release evidence.

Every completed chapter also publishes a verified code pack representing the
exact reviewed workspace at that chapter boundary. This is the preferred reader
starting point when the current repository has already moved ahead. The ZIP,
SHA-256 sidecar, source commit, and clean-directory commands are linked from the
chapter page. Browser Playground output still does not replace that pinned
workspace evidence.

## Level 3 — Notebook lab / Notebook 实验

Evcxr provides a Rust REPL and a Jupyter kernel. It is useful for longer
exploration where later cells build on earlier values, or where a lesson benefits
from tables, plots, or rich HTML output.

Notebook labs are optional enrichment, not the main book format:

- they require a local or hosted Jupyter environment;
- they are not a replacement for Cargo workspace tests;
- notebook state can hide dependencies between cells;
- generated notebook JSON should not become the only readable source;
- any notebook added later must have a reproducible source and execution check.

JupyterLite runs kernels in the browser through WebAssembly, but a maintained
first-party Rust kernel is not currently part of this book's baseline. Do not
promise an in-browser Rust notebook until that path is independently verified.

## Version and evidence boundary / 版本与证据边界

The book release is pinned to Rust 1.98.0. The public Rust Playground offers the
stable, beta, and nightly channels available when the reader visits it; it does
not preserve our exact historical stable compiler.

Therefore:

- every inline lab must pass the pinned local `mdbook test`;
- the public Playground is for rapid feedback, not release proof;
- a future stable compiler may produce different diagnostics or output;
- when browser and book evidence disagree, reproduce with the pinned toolchain;
- edition 2024 must be explicit on editable blocks;
- dependency-heavy examples belong in the project lab unless the dependency is
  intentionally supported and separately verified.

## Privacy and security / 隐私与安全

Clicking Run sends the edited source to an external compilation service.

Readers must never paste:

- passwords;
- tokens or API keys;
- private company source;
- personal information;
- unpublished security findings;
- data they are not allowed to share.

The public Rust Playground runs code in isolated containers with no external
network and bounded memory and execution time. Those controls reduce risk; they
do not turn uploaded source into private data.

The book must remain useful with JavaScript disabled or without internet access.
All important explanations, expected results, and local commands stay visible.

## Chapter rule / 单章规则

Every chapter includes a hands-on path, but not every chapter must force its
production code into the public Playground.

Choose one:

1. **Inline Playground:** a truthful one-file experiment exists;
2. **Project lab:** the behavior needs the real workspace;
3. **Both:** a small conceptual experiment plus the real cumulative command.

When an inline version would duplicate too much production code, remove
important boundaries, require unsupported local crates, or give a false sense of
parity, use a project lab and explain why.

## Definition of done / 完成标准

An inline lab is complete when:

- its source file exists and is registered;
- the visible block is marked `rust,editable,edition2024`;
- `mdbook test` compiles and runs it;
- the HTML build contains the editor and Run control;
- the generated PDF contains a readable static version;
- the chapter explains what to change and what to observe;
- online-service and version limitations are visible;
- no secret, network, or exact-version claim is implied.

The first lab lives at `book/src/_labs/00-first-playground.rs`.
