# Chapter map / 章节地图

The map is stable enough to start. Chapter names may improve, but the dependency
order should change only with a written reason.

| Ch | Chapter / 章节 | Rust idea | Engineering idea | Harness result |
|---:|---|---|---|---|
| 00 | Run one complete turn / 先跑通一个回合 | functions, values, tests | primitive whole | offline echo turn |
| 01 | Toolchain as evidence / 工具链就是证据 | Cargo, Edition 2024 | reproducibility | pinned workspace |
| 02 | Tests before code / 先写测试 | assertions, modules | TDD | first Red/Green contract |
| 03 | Read a large Rust repo / 阅读大型源码 | crates, navigation | evidence gathering | Codex flow map |
| 04 | Values and types / 值与类型 | inference, immutability | least privilege | typed inputs |
| 05 | Structs and newtypes / 结构体与新类型 | structs, tuple structs | domain modelling | IDs and records |
| 06 | Enums and match / 枚举与匹配 | ADTs, exhaustive match | invalid states | lifecycle states |
| 07 | Option and Result / 缺失与失败 | `Option`, `Result`, `?` | explicit failure | validation errors |
| 08 | Modules, crates, boundaries / 模块与边界 | visibility, workspaces | dependency direction | split core and CLI |
| 09 | Stack, heap, ownership / 堆栈与所有权 | move, `Drop` | clear ownership | owned thread history |
| 10 | Borrowing and lifetimes / 借用与生命周期 | references, lifetimes | local reasoning | borrowed transcript views |
| 11 | Strings, slices, collections / 字符串与容器 | `str`, slices, `Vec` | bounded data | context buffer |
| 12 | Smart pointers / 智能指针 | `Box`, `Rc`, `Arc` | shared ownership costs | service handles |
| 13 | Interior mutability / 内部可变性 | `Cell`, `RefCell`, locks | controlled mutation | registries and caches |
| 14 | Leaks, panic, unsafe boundaries / 泄漏与边界 | panic safety, `unsafe` | guarantee boundaries | failure policy |
| 15 | Traits as interfaces / Trait 即接口 | traits, methods | small contracts | model port |
| 16 | Generics and static dispatch / 泛型与静态分派 | bounds, associated types | zero-cost abstraction | generic tests/adapters |
| 17 | Trait objects / 动态分派 | `dyn Trait` | runtime extensibility | tool registry |
| 18 | Closures and iterators / 闭包与迭代器 | captures, iterator chains | composable transforms | scripted fakes |
| 19 | Errors and wire types / 错误与协议类型 | Serde, error layers | compatibility | versioned items/events |
| 20 | Futures and async / Future 与异步 | `Future`, `async/await` | mechanism vs policy | async provider |
| 21 | Channels and backpressure / 通道与背压 | bounded channels | overload control | command/event queues |
| 22 | Model → Tool → Model / 智能体循环 | async loop, matching | explicit orchestration | real agent loop |
| 23 | Cancellation and timeouts / 取消与超时 | task ownership | bounded work | interruptible turns |
| 24 | Send and Sync / 并发契约 | `Send`, `Sync` | capabilities in types | worker-safe services |
| 25 | Untrusted tool input / 不可信工具输入 | parsing, validation | trust boundaries | safe tool routing |
| 26 | Permissions and approvals / 权限与审批 | domain enums, oneshot | fail closed | approval protocol |
| 27 | Process execution policy / 进程执行策略 | process I/O | policy vs sandbox | restricted runner |
| 28 | Event log and replay / 事件日志与重放 | files, folds | event sourcing | durable thread state |
| 29 | Recovery and idempotency / 恢复与幂等 | partial failure | safe retry | resumable work |
| 30 | Context budget and compaction / 上下文预算 | iterators, budgets | bounded context | compaction policy |
| 31 | Tracing and diagnostics / 追踪与诊断 | structured tracing | observability | correlated traces |
| 32 | JSONL app server / JSONL 应用服务器 | protocol framing | client/server boundary | local app server |
| 33 | CLI and compatibility / CLI 与兼容性 | Clap, snapshots | stable product surface | usable client |
| 34 | Live provider and replay evals / 实时模型与重放评测 | feature gates | deterministic vs semantic checks | optional live adapter |
| 35 | MCP and extensions / MCP 与扩展 | plugin boundaries | open/closed trade-off | small extension API |
| 36 | Hardening and release / 加固与发布 | Miri, fuzzing, audit | evidence-based release | harness 0.1 receipt |

## Part milestones

- **Part 0:** a tiny complete turn
- **Part 1:** a typed domain model
- **Part 2:** clear ownership and bounded state
- **Part 3:** replaceable model and tool boundaries
- **Part 4:** a real async agent loop
- **Part 5:** controlled side effects
- **Part 6:** durable, observable operation
- **Part 7:** a client-facing local product
- **Part 8:** optional providers and extensions, followed by release evidence
