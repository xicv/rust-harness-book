# Scaffold publication receipt / 脚手架发布凭证

Published for review: **2026-08-24**

## Reviewed baseline

```text
8e28ffbfddf8b966e35c1cccecb2fcaed317771d
```

## Scope

- 88 reviewed source files
- 37 visibly labelled planned chapters
- 0 chapters presented as complete
- 3 buildable Rust crate shells
- 1 mdBook HTML source tree
- 1 Typst PDF smoke-build shell
- 1 Chapter 0 Red/Green planning packet

## Local evidence before publication

```text
python3 scripts/check_scaffold.py
python3 -m py_compile scripts/check_scaffold.py
git diff --cached --check
```

The local runtime did not contain Rust, mdBook, or Typst. The pull-request CI
therefore runs the pinned Rust, Clippy, mdBook, HTML, and Typst checks.

## Review boundaries

- No formatter or automatic fix command was run.
- No Chapter 0 behaviour has been implemented.
- No real model, tool, approval, persistence, async, or app-server feature is
  claimed.
- The Typst output is a build smoke test, not a visual proof of a finished PDF.
- The repository has no project-wide reuse licence yet.

## Next valid milestone

Write `one_prompt_produces_ordered_events`, observe the intended Red state, and
add only the smallest Chapter 0 implementation needed to turn it Green.
