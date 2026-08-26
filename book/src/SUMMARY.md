# Summary

- [Introduction / 导读](index.md)

# Part 0 · Start with a working whole / 从完整流程开始
- [先跑通一个回合 / Run One Complete Turn](00-preface/00-one-complete-turn.md)
- [工具链就是证据 / The Toolchain Is Evidence](00-preface/01-toolchain-as-evidence.md)
- [先写测试 / Tests Before Code](00-preface/02-tests-before-code.md)
- [阅读大型 Rust 源码 / Read a Large Rust Codebase](00-preface/03-read-large-rust.md)

# Part 1 · Model the problem / 建模问题
- [值与类型 / Values and Types](01-model/04-values-and-types.md)
- [结构体与新类型 / Structs and Newtypes](01-model/05-structs-and-newtypes.md)
- [枚举与匹配 / Enums and Match](01-model/06-enums-and-match.md)
- [缺失与失败 / Option and Result](01-model/07-option-and-result.md)
- [模块与边界 / Modules and Boundaries](01-model/08-modules-and-boundaries.md)

# Part 2 · Own the resources / 管理资源
- [堆栈与所有权 / Stack, Heap, and Ownership](02-ownership/09-stack-heap-ownership.md)
- [借用与生命周期 / Borrowing and Lifetimes](02-ownership/10-borrowing-and-lifetimes.md)
- [字符串、切片与容器 / Strings, Slices, and Collections](02-ownership/11-strings-slices-collections.md)
- [智能指针 / Smart Pointers](02-ownership/12-smart-pointers.md)
- [内部可变性 / Interior Mutability](02-ownership/13-interior-mutability.md)
- [失败边界 / Failure Boundaries](02-ownership/14-failure-boundaries.md)

# Part 3 · Design boundaries / 设计边界
- [Trait 即接口 / Traits as Interfaces](03-boundaries/15-traits-as-interfaces.md)
- [泛型与静态分派 / Generics and Static Dispatch](03-boundaries/16-generics.md)
- [动态分派 / Trait Objects](03-boundaries/17-trait-objects.md)
- [闭包与迭代器 / Closures and Iterators](03-boundaries/18-closures-and-iterators.md)
- [错误与协议类型 / Errors and Wire Types](03-boundaries/19-errors-and-wire-types.md)

# Part 4 · Build the agent loop / 构建智能体循环
- [Future 与异步 / Futures and Async](04-agent-loop/20-futures-and-async.md)
- [通道与背压 / Channels and Backpressure](04-agent-loop/21-channels-and-backpressure.md)
- [Model → Tool → Model](04-agent-loop/22-model-tool-model.md)
- [取消与超时 / Cancellation and Timeouts](04-agent-loop/23-cancellation-and-timeouts.md)
- [Send 与 Sync / Send and Sync](04-agent-loop/24-send-and-sync.md)

# Part 5 · Control effects / 控制副作用
- [不可信工具输入 / Untrusted Tool Input](05-effects/25-untrusted-tool-input.md)
- [权限与审批 / Permissions and Approvals](05-effects/26-permissions-and-approvals.md)
- [进程执行策略 / Process Execution Policy](05-effects/27-process-policy.md)

# Part 6 · Survive failure / 经受失败
- [事件日志与重放 / Event Log and Replay](06-durability/28-event-log-and-replay.md)
- [恢复与幂等 / Recovery and Idempotency](06-durability/29-recovery-and-idempotency.md)
- [上下文预算 / Context Budget](06-durability/30-context-budget.md)
- [追踪与诊断 / Tracing and Diagnostics](06-durability/31-tracing.md)

# Part 7 · Expose a product / 形成产品
- [JSONL 应用服务器 / JSONL App Server](07-product/32-jsonl-app-server.md)
- [CLI 与兼容性 / CLI and Compatibility](07-product/33-cli-and-compatibility.md)

# Part 8 · Verify and harden / 验证与加固
- [实时模型与评测 / Live Provider and Evaluations](08-hardening/34-live-provider-and-evals.md)
- [MCP 与扩展 / MCP and Extensions](08-hardening/35-mcp-and-extensions.md)
- [加固与发布 / Hardening and Release](08-hardening/36-release-evidence.md)

---

# Reader resources / 读者资源
- [章节代码包 / Chapter code packs](_delivery/chapter-code.md)
  - [第 0 章代码包 / Chapter 0 code pack](_delivery/ch00.md)
  - [第 1 章代码包 / Chapter 1 code pack](_delivery/ch01.md)
- [Book license / 本书许可](_delivery/book-license.md)
