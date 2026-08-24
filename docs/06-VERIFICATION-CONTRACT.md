# Verification contract / 验证契约

## Code block classes

Every Rust-looking block must be one of:

- **Run / 可运行:** compiles and runs
- **Compile-only / 仅编译:** compiles, not executed
- **Compile-fail / 预期失败:** must fail to compile
- **Pseudocode / 伪代码:** visibly not Rust source

Unlabelled Rust-looking pseudocode is not allowed.

## Source of truth

- Teaching snippets come from real source files.
- Use mdBook file includes and anchors.
- Do not paste a second copy into Markdown.
- Register every example in `book/examples.toml`.

## Test layers

- Pure unit tests for state transitions
- Integration tests for public harness behaviour
- Compile-fail tests for ownership and API misuse
- Property tests for parsers and state machines
- Snapshot tests for protocol and CLI output
- Deterministic scripted-model scenarios
- Replay evaluations from captured traces
- Optional live-provider evaluations
- Later hardening: concurrency model tests, Miri, fuzzing, coverage, mutation tests

## Default environment

The normal chapter path must not require:

- an API key;
- network access;
- a paid model;
- Docker;
- a cloud database;
- an external MCP server.

## Test quality rules

- Test observable behaviour.
- Prefer small fakes over heavy mocking.
- Compare complete domain values when practical.
- Avoid wall-clock sleeps.
- Inject clocks, IDs, filesystems, and process runners.
- Turn every reported bug into a deterministic regression test.
- Never use a successful live model run as the only proof.
