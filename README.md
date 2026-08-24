# Rust by Design · 可验证智能体 Harness

[![CI](https://github.com/xicv/rust-harness-book/actions/workflows/ci.yml/badge.svg)](https://github.com/xicv/rust-harness-book/actions/workflows/ci.yml)

> Working title / 暂定书名
> **《Rust 设计与实战：从零构建可验证的智能体 Harness》**
> **Rust by Design: Building a Verifiable Agent Harness**

This repository is the reviewed scaffold for a bilingual, why-first Rust book.
The reader learns modern Rust by building one real local agent harness from
start to finish.

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

**The scaffold has passed its first content and structure review and is ready for
Chapter 0.** Phase 0 research is complete enough to freeze the initial direction.
Each chapter still receives a focused source refresh before writing starts.

The 37 chapter files are planning shells, not finished book content. They are
labelled clearly so the repository never presents a placeholder as a completed
chapter.

Start here:

1. `docs/00-RESEARCH-STATUS.md`
2. `docs/01-BOOK-CONSTITUTION.md`
3. `docs/04-CHAPTER-MAP.md`
4. `docs/10-FIRST-CHAPTER-BRIEF.md`
5. `docs/11-SCAFFOLD-REVIEW.md`
6. `AGENTS.md`

## Pinned tools / 固定工具版本

- Rust `1.98.0`, Edition 2024, Cargo resolver 3
- mdBook `0.5.4`
- Typst `0.15.1`
- Codex source snapshot `2df67054232090af8d2fa197c46b994bc2b0dda1`

The exact pins live in `rust-toolchain.toml` and `book/sources.lock.toml`.

## Local verification / 本地验证

```sh
make scaffold-check
make fmt-check
make check
make test
make lint
make book-test
make book-build
make pdf
```

Or run the complete check-only pipeline:

```sh
make verify
```

`fmt-check` only checks formatting. It does not rewrite files.

## Important boundary / 重要边界

This is an original clean-room teaching project. It studies the teaching
strengths of two Chinese Rust books and the public architecture of OpenAI Codex,
but it must not copy their prose, diagrams, exercises, or large code sections.

No teaching implementation has been added yet. Chapter 0 will add the first
Red → Green behaviour and the first runnable harness flow.

No project-wide reuse licence has been selected. See
`docs/08-SOURCE-AND-RIGHTS-POLICY.md` before reusing material.
