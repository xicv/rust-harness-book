# Research status / 研究状态

Reviewed: **2026-08-25**

## Decision

**Go: Chapters 0 and 1 and the dual-output publishing foundation are complete. Chapter 2 is the next valid milestone.**

The initial architecture and editorial contracts remain stable. Research now
continues in small, chapter-specific passes before each new implementation.

## Verified through Chapter 1

- Book purpose, audience, tone, and non-goals
- Why-first and software-engineering-first teaching model
- Cumulative TDD harness strategy
- Offline deterministic tests versus live model evaluations
- Clean-room and attribution boundaries
- Rust 1.98.0, Edition 2024, and Cargo resolver 3 baseline
- mdBook 0.5.4 and Typst 0.15.1 publishing pins
- Codex source-reading snapshot and Chapter 0/1 source paths
- Bilingual terminology and mobile-first card rules
- A real offline `Thread → Turn → Item` teaching slice
- Meaningful Red 1, Red 2, and Green evidence for Chapter 0
- A real Red and Green cycle for the Chapter 1 toolchain doctor
- Source-controlled example registration and chapter receipts
- A standard-library-only `book-render` tool that generates Typst from the
  canonical Markdown source
- Full HTML and PDF content parity for completed Chapters 0 and 1

## Publishing research now implemented

The publishing pipeline uses one editable prose source:

```text
book/src/**/*.md
        ├── mdBook → dist/html/
        └── book-render → dist/typst/book.typ
                              └── Typst → dist/rust-harness-book.pdf
```

`SUMMARY.md` controls order. Tested source includes are expanded for both output
paths. Unsupported HTML, path escapes, recursive includes, and remote images fail
closed.

## Research that continues chapter by chapter

- Exact official Rust sources for the topic
- Exact pinned Codex excerpts and nearby tests
- Current ecosystem choices and crate versions
- Runnable comparison examples in other languages
- Accessibility checks for each new visual component
- Exact terminology review
- Rights and attribution review for every external asset
- Human visual review of new PDF layouts and diagrams

## Honest build status

- 2 of 37 planned chapters are complete.
- The harness supports one deterministic offline turn and a read-only toolchain
  diagnostic command.
- No real model provider, tools, async runtime, persistence, approval flow, MCP,
  or production sandbox is implemented yet.
- HTML and PDF are generated from the same canonical Markdown chapters.
- The renderer intentionally supports the documented book subset rather than all
  CommonMark and arbitrary HTML.

## Gate before each chapter

A chapter cannot start until it has:

- one clear behaviour;
- one engineering principle;
- one fair language comparison;
- one source-reading target;
- one Red test;
- one harness milestone;
- one verification plan.
