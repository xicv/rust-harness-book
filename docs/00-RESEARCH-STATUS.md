# Research status / 研究状态

Reviewed: **2026-08-24**

## Decision

**Go: the reviewed scaffold is ready for Chapter 0.**

The research is complete enough to freeze the first architecture and editorial
contract. Waiting for a perfect page-by-page audit before starting would now add
delay without reducing the main risks.

## Verified for this scaffold

- Book purpose, audience, tone, and non-goals
- Why-first and software-engineering-first teaching model
- Cumulative TDD harness strategy
- Offline deterministic tests versus live model evaluations
- Clean-room and attribution boundaries
- Rust 1.98.0, Edition 2024, and Cargo resolver 3 baseline
- mdBook 0.5.4 and Typst 0.15.1 publishing pins
- Codex source-reading snapshot and all paths in the initial source map
- HTML/PDF publishing direction
- Bilingual terminology rules
- Mobile-first card design
- Full chapter dependency path
- Chapter 0 behaviour and acceptance contract

## Research that continues chapter by chapter

- Exact official Rust sources for the topic
- Exact pinned Codex excerpts and nearby tests
- Current ecosystem choices and crate versions
- Runnable comparison examples in other languages
- Accessibility checks for each new visual component
- Exact terminology review
- Rights and attribution review for every external asset

## Honest build status

- The 37 chapter files are labelled planning shells.
- Chapter 0 prose and behaviour are not implemented yet.
- The Rust workspace contains only a compiling scaffold marker.
- The Typst file is a PDF shell; `book-render` does not exist yet.
- The local review runtime did not contain Rust, mdBook, or Typst binaries.
  GitHub CI runs those checks with the pinned versions.

## Gate before each chapter

A chapter cannot start until it has:

- one clear behaviour;
- one engineering principle;
- one fair language comparison;
- one source-reading target;
- one Red test;
- one harness milestone;
- one verification plan.
