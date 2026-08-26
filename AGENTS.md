# Project instructions

## Mission

Build a bilingual, why-first Rust book and a real educational agent harness.
The book and the code are one cumulative project.

## Non-negotiable rules

- Research before writing a chapter.
- Use primary technical sources for current Rust, Cargo, mdBook, Typst, and Codex.
- Treat the versions in `book/sources.lock.toml` as release pins.
- Keep the Rust baseline stable-first and pinned in `rust-toolchain.toml`.
- Use TDD: write a meaningful failing test before implementation.
- Keep the main branch green. Store chapter Red-state instructions separately.
- Every shown Rust block must be runnable, compile-only, compile-fail, or clearly
  labelled pseudocode.
- Prefer code includes from tested files. Do not duplicate code manually in prose.
- When a concept can be expressed honestly as one self-contained file, provide an
  editable browser Playground sourced from a checked file.
- Do not force workspace, multi-file, file-system, process, or local-crate behavior
  into a misleading Playground example. Use a project lab instead.
- Treat online Playground runs as learning feedback, not release evidence. Pinned
  local tests and CI remain authoritative.
- Default tests must be deterministic and offline.
- A live model test never replaces a deterministic harness test.
- Explain why a design exists, its trade-offs, and where another language may win.
- Treat model output and external tool output as untrusted input.
- Keep model-visible context, queues, output, time, and retries bounded.
- Do not claim full Codex compatibility or production sandbox security.
- Do not copy protected prose, diagrams, exercises, or code from reference books.
- Do not present a planned chapter shell as finished content.

## Change discipline

- Do not create unrelated formatting changes.
- Never run a mutating formatter such as `cargo fmt`, `rustfmt` without
  `--check`, or another tool's write/fix mode.
- For changed Rust files, use check-only formatting:

  ```sh
  rustfmt --edition 2024 --check path/to/changed.rs
  ```

- Lint the changed crate first:

  ```sh
  cargo clippy -p <changed-crate> --all-targets -- -D warnings
  ```

- Run the changed crate tests, then the cumulative workspace tests before a
  chapter is considered complete.
- `make verify` is the canonical check-only project verification command.
- Do not commit, push, open a pull request, or publish unless explicitly asked.

## Chapter workflow

1. Refresh sources.
2. Write the behaviour contract.
3. Add the Red test.
4. Confirm the expected failure.
5. Add the smallest Green implementation.
6. Refactor under tests.
7. Add edge and negative tests.
8. Compare with pinned Codex source.
9. Record trade-offs and limits.
10. Verify code, inline labs, HTML, PDF source, glossary, links, and the chapter receipt.

## Writing style

- Smart casual.
- Native, modern, easy English.
- Short paragraphs and concise dot points.
- Chinese term first, canonical English term beside it on first use.
- Avoid hype, vague claims, and language wars.
- Show evidence. State uncertainty.
