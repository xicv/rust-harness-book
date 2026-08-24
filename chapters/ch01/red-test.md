# Chapter 1 Red test

Create a black-box integration test named:

```text
doctor_reports_pinned_and_active_toolchain
```

The test launches the real `harness-cli` binary with `--doctor` and compares the
complete output with the project pin.

## Observed Red

Commit `28b1ecff3d3883692cb2faaf72cdf019e54ecf9e`, CI run `32786979806`:

- Chapter 0 tests remained Green;
- the CLI treated `--doctor` as an ordinary prompt;
- actual output was the five-event Echo turn;
- expected output was the seven-line toolchain report.

This is the intended missing-behaviour failure.


## Observed Green

Final Chapter 1 code commit `decef67c89afba8e4eb095b0c16454e4aca97eb5`,
CI run `32788814516`:

- the real binary routed `--doctor` separately from the Echo prompt;
- the report matched Rust 1.98.0, Edition 2024, resolver 3, and the root lockfile;
- every Chapter 0 test remained Green;
- Rust formatting check, Cargo check, tests, Clippy, mdBook, HTML, and Typst
  smoke build all passed.
