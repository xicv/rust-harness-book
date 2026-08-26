# Chapter 1 acceptance contract

## Scenario 1: matching pinned toolchain

Given the source-controlled toolchain and Workspace files:

- `harness-cli --doctor` reads the exact Rust pin;
- it probes the active `rustc` and `cargo` versions;
- it reports Edition 2024 and resolver 3;
- it reports the root `Cargo.lock` as present;
- it exits successfully when the active versions match the pin.

## Scenario 2: mismatch or probe failure

- a completed mismatch exits with code `1`;
- a probe or source-reading failure exits with code `2`;
- the command reports only and never edits the user's machine.

## Cumulative behaviour

The Chapter 0 Echo turn and every earlier test must remain Green.
