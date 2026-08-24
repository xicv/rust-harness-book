# Research status / 研究状态

Reviewed: **2026-08-25**

## Decision

**Go: Chapters 0 and 1 are complete. Chapter 2 is the next valid milestone.**

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

## Research that continues chapter by chapter

- Exact official Rust sources for the topic
- Exact pinned Codex excerpts and nearby tests
- Current ecosystem choices and crate versions
- Runnable comparison examples in other languages
- Accessibility checks for each new visual component
- Exact terminology review
- Rights and attribution review for every external asset

## Honest build status

- 2 of 37 planned chapters are complete.
- The harness supports one deterministic offline turn and a read-only toolchain
  diagnostic command.
- No real model provider, tools, async runtime, persistence, approval flow, MCP,
  or production sandbox is implemented yet.
- The HTML chapters build from the canonical Markdown source.
- The Typst file remains a PDF smoke shell; `book-render` does not exist yet.

## Gate before each chapter

A chapter cannot start until it has:

- one clear behaviour;
- one engineering principle;
- one fair language comparison;
- one source-reading target;
- one Red test;
- one harness milestone;
- one verification plan.
