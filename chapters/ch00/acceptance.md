# Chapter 0 acceptance contract

## Scenario 1: one valid prompt

Given a new harness and a non-empty prompt:

- one thread starts;
- one turn starts;
- one user item is recorded;
- one assistant item is recorded;
- the turn completes;
- all events use the same thread and turn IDs;
- event order is deterministic.

## Scenario 2: blank prompt

Given a blank or whitespace-only prompt:

- validation fails;
- no turn starts;
- no item is recorded;
- no model is called.

## Demo

```sh
cargo run -p harness-cli -- "hello"
```

The command must work without network access or an API key.
