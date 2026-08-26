# Rust by Design · 可验证智能体 Harness

[![CI](https://github.com/xicv/rust-harness-book/actions/workflows/ci.yml/badge.svg)](https://github.com/xicv/rust-harness-book/actions/workflows/ci.yml)

> Working title / 暂定书名
> **《Rust 设计与实战：从零构建可验证的智能体 Harness》**
> **Rust by Design: Building a Verifiable Agent Harness**

This repository contains a bilingual, why-first Rust book and the cumulative
educational agent harness built throughout it.

## Project promise / 项目承诺

- Explain **why** Rust is designed this way, not only what the syntax means.
- Compare Rust fairly with modern C++, Go, Java, TypeScript, Swift, and Python.
- Add one useful harness capability in every completed chapter.
- Use **test-driven development (TDD / 测试驱动开发)** across the whole book.
- Keep every Rust example runnable, compilable, or intentionally compile-fail.
- Teach code reading, design decisions, verification, and AI-assisted engineering.
- Publish mobile-first HTML and a carefully typeset PDF from one editorial source.
- Keep Chinese and English technical terms consistent through a checked glossary.

## Current status / 当前状态

**Chapters 0 and 1 are complete: 2 of 37 planned chapters.**

- Chapter 0 builds one real, deterministic offline turn with typed lifecycle
  events, input validation, and black-box CLI tests.
- Chapter 1 pins the Rust workspace and adds a read-only `--doctor` command that
  compares source-controlled settings with the active Rust and Cargo tools.
- Every displayed Rust example in these chapters is sourced from checked code
  and registered in `book/examples.toml`.
- mdBook builds the complete mobile-first HTML reading output.
- `book-render` now converts the same canonical Markdown into generated Typst,
  and Typst compiles the full PDF. Completed Chapters 0 and 1 therefore have
  HTML/PDF content parity.
- The reader-delivery pipeline builds verified Chapter 0 and Chapter 1 code
  packs and a generated GitBook-compatible Markdown mirror. These local
  artifacts are verified; hosted Pages and GitBook availability remain separate
  post-merge checks.

Start here:

1. `book/src/00-preface/00-one-complete-turn.md`
2. `book/src/00-preface/01-toolchain-as-evidence.md`
3. `chapters/ch00/receipt.toml`
4. `chapters/ch01/receipt.toml`
5. `docs/01-BOOK-CONSTITUTION.md`
6. `docs/07-PUBLISHING-PIPELINE.md`
7. `AGENTS.md`

## Pinned tools / 固定工具版本

- Rust `1.98.0`, Edition 2024, Cargo resolver 3
- mdBook `0.5.4`
- Typst `0.15.1`
- Codex source snapshot `2df67054232090af8d2fa197c46b994bc2b0dda1`

The exact pins live in `rust-toolchain.toml` and `book/sources.lock.toml`.

## Try the harness / 运行 Harness

```sh
cargo run -p harness-cli -- "hello"
cargo run -p harness-cli --locked -- --doctor
```

Both commands are deterministic and require no API key.

## Build the book / 构建书籍

```sh
mdbook build book
make pdf
```

Build every reader-delivery artifact with:

```sh
make reader-delivery
```

That command builds `dist/html/`, reconstructs and verifies the Chapter 0 and
Chapter 1 ZIPs under `dist/html/downloads/`, and generates `dist/gitbook/` from
the same strict Markdown parser used by the PDF renderer. Chapter packs require
the pinned historical commits, so a shallow Git clone fails closed.

The PDF path is:

```text
dist/rust-harness-book.pdf
```

The generated Typst project lives under `dist/typst/`. It is build output and
must not be edited or committed. `book/src/` remains the only prose source.

## Reader editions / 读者版本

The configured authoritative interactive URL is
[GitHub Pages](https://xicv.github.io/rust-harness-book/). The repository must
first be configured to use GitHub Actions as its Pages source, and a successful
deployment must be visited before the URL is described as live.

GitBook is a secondary renderer. The workflow regenerates `dist/gitbook/` only
from verified `main` and updates `gitbook-publish` without force-pushing.
Connecting that branch to a GitBook space, choosing the project directory, and
publishing the space remain manual account-side steps. GitBook does not provide
the mdBook inline Playground experience; its generated Introduction links back
to the interactive edition.

## Local verification / 本地验证

```sh
make scaffold-check
make fmt-check
make check
make test
make lint
make book-test
make reader-delivery
make pdf
```

Or run the complete check-only pipeline:

```sh
make verify
```

`fmt-check` checks formatting without rewriting files. Cargo verification uses
the committed lockfile through `--locked`.

## Important boundary / 重要边界

This is an original clean-room teaching project. It studies the teaching
strengths of two Chinese Rust books and the public architecture of OpenAI Codex,
but it does not copy their prose, diagrams, exercises, or large code sections.

The renderer is purpose-built for the documented book Markdown contract. It
rejects unsupported HTML, repository-escaping includes, recursive includes, and
remote images instead of silently producing incomplete output.

The current harness does not claim real model intelligence, tools, streaming,
persistence, Codex compatibility, or production sandbox security.

Original software and source code use the [MIT code licence](LICENSE). Original
book prose and diagrams use the separate [MIT book licence](LICENSE-BOOK).
Third-party material retains its own terms; see
`docs/08-SOURCE-AND-RIGHTS-POLICY.md` before reusing material.
