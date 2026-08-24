# Codex source map / Codex 源码地图

Pinned snapshot:

```text
openai/codex
2df67054232090af8d2fa197c46b994bc2b0dda1
```

| Topic | Pinned source target | Book use |
|---|---|---|
| Workspace and Edition 2024 | `codex-rs/Cargo.toml` | crate boundaries and ecosystem map |
| Public primitives | `codex-rs/app-server/README.md` | thread, turn, item, lifecycle |
| Thread runtime | `codex-rs/core/src/codex_thread.rs` | ownership, submission, cancellation |
| Turn loop | `codex-rs/core/src/session/turn.rs` | model → tool → model flow |
| Protocol types | `codex-rs/protocol/src/protocol.rs` | commands and events |
| Tool routing | `codex-rs/core/src/tools/router.rs` | untrusted tool-call parsing |
| Tool registry | `codex-rs/core/src/tools/registry.rs` | traits and dynamic dispatch |
| Sandboxing policy | `codex-rs/core/src/tools/sandboxing.rs` | allow, deny, approve decisions |
| Engineering rules | `AGENTS.md` | tests, bounded context, small APIs |

Rules:

- Confirm each path at the pinned commit before writing the chapter.
- Read nearby tests before drawing conclusions.
- Record source fact and project inference separately.
- Use the smallest excerpt that proves the point.
