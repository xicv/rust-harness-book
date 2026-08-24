# Harness 0.1 contract / Harness 0.1 契约

## What we are building

A local, educational agent runtime that can:

- create and resume threads;
- start and interrupt turns;
- store typed user, assistant, and tool items;
- call a model through a narrow provider interface;
- stream events;
- route validated tool calls;
- request and enforce approvals;
- run a restricted local process adapter;
- persist and replay an event log;
- recover from interrupted work;
- keep model context bounded;
- expose a small JSONL app-server protocol;
- operate through a CLI;
- run deterministic offline tests and optional live evaluations.

## Architectural direction

```text
Client
  ↓ commands
Coordinator / imperative shell
  ↓ input
Pure domain state machine
  ↓ events + requested effects
Adapters: model · tools · approval · store · process · clock
```

## Core principles

- Functional core, imperative shell
- Explicit state and effects
- Narrow ports, concrete implementations first
- Bounded work and backpressure
- Fail closed around permissions
- Replayable and observable behaviour
- Portable core with OS-specific adapters isolated and tested before support is claimed

## Honest limits

Version 0.1 will not claim:

- full Codex API or behaviour compatibility;
- production sandbox isolation;
- complete MCP support;
- cloud task execution;
- multi-agent parity;
- deterministic LLM semantics;
- freedom from logical leaks or deadlocks.

## Parity labels

- **P0:** concept demonstrated
- **P1:** interface subset
- **P2:** behaviour-tested subset
- **P3:** adversarially hardened subset
- **NC:** not claimed
