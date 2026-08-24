# ADR 0004: Pinned but living sources

## Decision

Pin Rust, dependencies, and Codex source per book release. Refresh sources before
writing each chapter and before publishing a new release.

## Why

- “Latest” is not reproducible.
- An unpinned source path can silently change meaning.
- A permanently frozen book becomes stale.

## Cost

Maintainers must keep a source ledger and review upstream changes.
