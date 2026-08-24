# Chapters 0–1 research notes / 第 0–1 章研究记录

Reviewed: **2026-08-25**

## Scope

This note records the source refresh used to finish:

- Chapter 0 — `Run One Complete Turn`
- Chapter 1 — `The Toolchain Is Evidence`

The book continues to cite the pinned Codex snapshot:

```text
openai/codex
2df67054232090af8d2fa197c46b994bc2b0dda1
```

Current Codex material was checked for drift, but the book pin was not silently
changed.

## Source facts

### Rust and Cargo

- Rust 1.98.0 is the stable baseline selected for this book revision.
- Edition 2024 and Cargo resolver 3 are separate from the exact toolchain pin.
- A virtual Workspace should state its resolver explicitly.
- Workspace members share the root lockfile and output directory.
- `--locked` rejects verification that would need to change `Cargo.lock`.
- Cargo integration tests compile separately and can exercise a crate's public
  interface.
- Cargo exposes built binaries to integration tests through
  `CARGO_BIN_EXE_<name>`.

Primary sources are recorded in `book/sources.lock.toml`.

### mdBook

- Chapter code is included from real source files through named anchors.
- Displayed Rust is marked non-runnable inside mdBook when it is only a partial
  region; the registered Cargo command remains its verification source.

### Codex

At the pinned snapshot:

- app-server describes `Thread`, `Turn`, and `Item` as its top-level interaction
  primitives;
- lifecycle notifications include thread start, turn start, item progress, and
  turn completion;
- the Rust implementation is a large Edition 2024 Workspace with explicit crate
  boundaries.

## Project decisions

- Chapter 0 implements only a deterministic offline echo turn.
- It returns domain events rather than printing inside `harness-core`.
- Fixed IDs are acceptable for this one-turn teaching slice and are not the final
  identity design.
- A function pointer is used before a provider Trait because there is not yet a
  real second provider shape.
- Chapter 1 adds a read-only `--doctor`; it reports evidence but never changes the
  learner's machine.
- Cargo verification uses the committed lockfile through `--locked`.
- No third-party Rust dependency is needed for either chapter.

## Reference-book boundary

The two supplied Chinese books informed teaching-method and gap analysis only.
Their prose, diagrams, exercises, and code were not copied into these chapters.
The new chapters independently preserve two useful ideas:

- explain the design reason, not only the syntax;
- learn through a useful whole, deliberate practice, and reflection.

## Remaining uncertainty

- Platform-specific toolchain details need dedicated macOS, Linux, and Windows
  coverage before portability is claimed.
- The current Doctor does not inspect rust-analyzer, linkers, targets, mdBook, or
  Typst.
- The PDF pipeline still lacks canonical Markdown-to-Typst content conversion.
