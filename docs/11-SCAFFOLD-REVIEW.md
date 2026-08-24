# Scaffold review / 脚手架审查

Reviewed: **2026-08-24**

## Outcome

**Ready for Chapter 0, with clear limits.**

The repository contains a real project scaffold, not a finished book or agent.
Every planned chapter is visibly labelled as a placeholder. The Rust workspace,
mdBook source, Typst shell, source ledger, and Chapter 0 acceptance packet are
real files with checkable contracts.

## Source verification

The review confirmed:

- Rust 1.98.0 as the release baseline;
- Edition 2024 and Cargo resolver 3;
- mdBook 0.5.4 for HTML and code-example testing;
- Typst 0.15.1 for PDF;
- Codex snapshot `2df67054232090af8d2fa197c46b994bc2b0dda1`;
- every initial Codex path listed in `docs/source-map/codex.md` at that commit;
- full commit pins for every third-party GitHub Action used by CI.

## Problems found and fixed

- Removed an obsolete mdBook `multilingual` setting.
- Replaced the placeholder author with `xicv and contributors`.
- Added the real repository and edit URLs to mdBook.
- Pinned mdBook, Typst, Codex, Rust, and CI actions in one source ledger.
- Corrected the Part 7/Part 8 milestone split.
- Corrected the Evaluation glossary chapter from `ch33` to `ch34`.
- Distinguished Codex conversation Thread from an OS thread.
- Clarified the two useful Red states: missing API, then failing behaviour.
- Added CI for Rust, Clippy, mdBook, HTML, and Typst PDF checks.
- Expanded the scaffold validator to check JSON, chapters, glossary references,
  source pins, action pins, local Markdown links, and generated-file hygiene.
- Removed generated Python bytecode from the source tree.

## Checks performed locally

The review runtime could run:

```text
python3 scripts/check_scaffold.py
python3 -m py_compile scripts/check_scaffold.py
git diff --check
```

It did not contain Rust, mdBook, or Typst binaries. Those checks are therefore
performed by the pinned GitHub CI workflow after publication.

## Deliberately not implemented

- Chapter 0 prose
- Chapter 0 Red/Green Rust behaviour
- A real model provider
- Tools, approvals, persistence, async execution, or app-server protocol
- The Markdown-to-Typst `book-render` tool
- A project-wide reuse licence

These are roadmap items, not hidden stubs. The next valid change is Chapter 0,
beginning with `one_prompt_produces_ordered_events`.
