# Chapter 0 brief / 第 0 章说明

## Working title

**先跑通一个回合 / Run One Complete Turn**

## Reader outcome

By the end, the reader can run a tiny offline agent flow and explain its four
observable stages:

```text
thread started → turn started → items completed → turn completed
```

## Why this is first

- A working whole gives the later concepts a home.
- The reader sees a useful result before learning every rule.
- The project starts with behaviour, not a syntax catalogue.
- The first test becomes the safety net for every later refactor.

## First Red behaviour

```text
Given one non-empty user prompt,
when the offline harness runs one turn,
then it records one user item and one assistant item,
and emits lifecycle events in a stable order.
```

## Red sequence

The first run may fail to compile because the planned public API does not exist.
That is an intentional Red state, not an accidental typo or broken test fixture.

Then add only enough type and function signatures for the test to compile. The
behaviour assertion must still fail. Implement the behaviour only after that
second, clearer Red result has been observed.

## Second Red behaviour

```text
An empty or whitespace-only prompt is rejected before a turn starts.
```

## Minimal Green design

- `ThreadId`, `TurnId`, and `ItemId` newtypes
- `Item::User` and `Item::Assistant`
- `Event` lifecycle enum
- `EchoModel` concrete implementation
- one pure `run_turn` function
- CLI shell that prints the events

No async, traits, network, tools, persistence, or model API yet.

## Why cards

- Why use typed IDs instead of raw integers?
- Why return events instead of printing inside the core?
- Why start concrete instead of creating a provider trait immediately?
- Why model lifecycle with an enum instead of several booleans?

## Fair comparison

Compare the same tiny flow with a short TypeScript or Python version:

- those versions start faster;
- Rust adds more up-front modelling;
- Rust gives stronger refactoring and exhaustive-state checks;
- neither approach is universally better.

## Required tests

- `one_prompt_produces_ordered_events`
- `blank_prompt_is_rejected_before_turn_start`
- `completed_event_reuses_thread_and_turn_ids`
- CLI black-box smoke test

## Lockfile note

The first successful Cargo command creates the workspace `Cargo.lock`. Because the
project ships executables, review and commit that lockfile with the first chapter.

## Chapter 0 exit

```sh
cargo run -p harness-cli -- "hello"
cargo test -p harness-core
cargo test --workspace
mdbook test book -L target/debug/deps
```

## Not yet claimed

- real model intelligence;
- streaming;
- tools;
- concurrency;
- Codex compatibility.
