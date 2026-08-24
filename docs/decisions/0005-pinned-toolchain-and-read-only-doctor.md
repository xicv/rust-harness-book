# ADR 0005: Pin the toolchain and keep Doctor read-only

Status: accepted on 2026-08-25

## Context

Book examples, diagnostics, and dependency resolution can change across Rust
versions. A learner also needs a simple way to see whether the active tools match
the repository contract.

## Decision

- Pin Rust 1.98.0 in `rust-toolchain.toml`.
- Declare Edition 2024, `rust-version = "1.98"`, and resolver 3 separately.
- Commit the root `Cargo.lock` because the Workspace ships a CLI.
- Use `--locked` in cumulative Cargo checks.
- Provide `harness-cli --doctor` as a read-only report.
- Never let Doctor install, upgrade, reformat, or repair the learner's machine.

## Consequences

- Chapter evidence is more reproducible and upgrades become reviewable changes.
- A tool mismatch is visible through output and exit status.
- The project must deliberately maintain its pins.
- A green Doctor result covers only the checks it reports; it is not proof that
  every platform tool or dependency is safe.
