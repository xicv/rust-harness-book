# ADR 0001: One cumulative harness

## Decision

Every chapter changes the same harness rather than starting a new unrelated
sample project.

## Why

- Concepts stay connected.
- Earlier tests protect later refactors.
- The reader finishes with a useful product.
- Architecture trade-offs become real rather than theoretical.

## Cost

- Chapter states need careful versioning.
- Later refactors can make early explanations stale.

## Control

Use chapter start/done tags and source includes pinned to chapter states.
