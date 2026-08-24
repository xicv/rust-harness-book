# Legacy topic delta / 旧书主题更新表

This is a coverage map, not permission to reuse text.

| Legacy topic | Treatment |
|---|---|
| Nightly-first setup | Replace with stable Rust 1.98 and Edition 2024 |
| RLS and Racer | Replace with rust-analyzer |
| Rust 2018 as future | Replace with modern edition migration model |
| Basic syntax catalogue | Reframe through harness behaviours |
| Ownership and borrowing | Keep as the conceptual core |
| NLL as a separate future feature | Integrate as normal compiler behaviour |
| `lazy_static` as default | Prefer explicit ownership, `OnceLock`, or `LazyLock` |
| `failure` crate | Replace with standard errors, typed errors, and context |
| Trait aliases and specialization | Move to clearly marked horizon cards |
| Generators | Teach `Future` state machines; raw coroutine material is horizon |
| Old `Vec` source snapshot | Replace with a bounded event buffer and pinned current source |
| Channels | Expand to bounded queues, backpressure, closure, and draining |
| Parallel-crate catalogue | Replace with a decision framework |
| FFI and unsafe syntax | Rewrite for Rust 2024 rules and written safety invariants |
| Testing as final chapter | Distribute TDD and verification across every chapter |
