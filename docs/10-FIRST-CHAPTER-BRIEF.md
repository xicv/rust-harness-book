# Chapter 0 brief / 第 0 章说明

## Working title

**先跑通一个回合 / Run One Complete Turn**

## 这一章为什么存在

读者还没有系统学过 Rust，但我们不希望第一章从变量、分号和关键字清单开始。

我们先认识全书要构建的产品：一个本地 Agent Harness。接着完成一个很小、但从输入到输出真的走通的回合。读者先看见完整流程，后面的类型、所有权、接口和异步才有落脚点。

## Reader outcome

本章结束后，读者可以：

- 用普通语言说明模型与 Harness 的区别；
- 解释 Thread、Turn、Item 和 Event；
- 运行一个完全离线的 Echo 回合；
- 看懂五个生命周期事件；
- 大致读懂领域类型、核心流程和 CLI 三层代码；
- 说出为什么核心返回事件，而不是直接打印；
- 运行聚焦测试，看到测试怎样保护事件顺序；
- 清楚说出这个版本还不是什么。

## Reader-facing order / 阅读顺序

正文必须采用：

```text
1. 全书和产品全景
2. Agent Harness 概念
3. Thread、Turn、Item、Event
4. 本章要完成的最小产品
5. 先运行并读懂输出
6. 分层阅读类型、核心流程、CLI
7. 回看 Red → Green → Refactor
8. 为什么 Rust 用类型表达这些概念
9. 公平比较
10. 自己动手
11. Codex 源码阅读
12. 验证、限制和下一步
```

不要先用 Red 日志或长篇成果列表挡住读者。

## Development order / 开发顺序

代码历史仍保留真实 TDD：

```text
Given one non-empty user prompt,
when the offline harness runs one turn,
then it records one user item and one assistant item,
and emits lifecycle events in a stable order.
```

### Red 1

公开类型和 `run_turn` 不存在，因此测试为目标 API 编译失败。

### Red 2

只加入签名和空结果后，测试可以编译，但完整事件断言失败。

### Green

加入最小实现：

- `ThreadId`、`TurnId`、`ItemId`
- `Item::User` 与 `Item::Assistant`
- `Event` 生命周期枚举
- 具体的 `EchoModel`
- `run_turn` 与小型函数接缝
- 由 CLI 渲染事件

### Refactor

只做由当前代码支持的整理：

- 核心与显示分开；
- ID 显示格式一致；
- 不提前加入 Trait、async、Serde、UUID 或持久化。

## Concept explanation

必须先用非语法语言解释：

- 模型负责生成候选回复；
- Harness 负责组织状态、步骤和边界；
- Thread 是持续对话；
- Turn 是一次用户请求及随后的工作；
- Item 是其中一项输入或输出；
- Event 让外部知道发生了什么。

然后才展示 Rust 类型。

## Official references

在相关段落附近提供：

- Rust Book: Functions
- Rust Book: Structs
- Rust Book: Enums and Pattern Matching
- Rust Book: Recoverable Errors with `Result`
- Rust Book: Writing Automated Tests
- Codex app-server Thread / Turn / Item documentation

链接帮助读者继续查询，不替代正文解释。

## Required tests

- `one_prompt_produces_ordered_events`
- `blank_prompt_is_rejected_before_turn_start`
- `completed_event_reuses_thread_and_turn_ids`
- CLI black-box smoke test

## Hands-on experience

读者至少完成：

```sh
cargo run -p harness-cli -- "hello"
cargo test -p harness-core --test turn_flow
```

建议实验：

- 更换提示词；
- 输入前后空格，观察归一化；
- 故意交换用户和助手事件，观察测试失败；
- 恢复代码，再运行测试。

## Not yet claimed

- 真实模型智能；
- streaming；
- tools；
- concurrency；
- persistence；
- Codex compatibility；
- production sandbox security。

本章的价值不是“我们已经造好一个智能体”，而是让第一个可以验证的完整回合成为后续所有章节的地基。
