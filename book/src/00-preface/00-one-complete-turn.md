<!--
chapter_id: ch00
chapter_status: completed
chapter_goal: A tiny offline turn and its first behavioural tests.
rust: 1.98.0
codex_commit: 2df67054232090af8d2fa197c46b994bc2b0dda1
new_terms: [agent-harness, thread, turn, item, event, test-driven-development, newtype-pattern, functional-core-imperative-shell, black-box-test]
examples: [ch00-domain-types, ch00-run-turn, ch00-ordered-events, ch00-blank-prompt, ch00-render-event, ch00-cli]
-->

# 先跑通一个回合 / Run One Complete Turn

> **Status / 状态:** Complete
> **Harness milestone / Harness 里程碑:** A tiny offline turn and its first behavioural tests.

<div class="learning-card">
<p class="card-label">Outcome / 本章成果</p>

完成本章后，你可以：

- 运行一个完全离线的 **智能体运行框架** <span class="term-en">Agent Harness</span>；
- 看懂 **会话** <span class="term-en">Thread</span>、**回合** <span class="term-en">Turn</span>、**条目** <span class="term-en">Item</span> 和 **事件** <span class="term-en">Event</span>；
- 用测试证明事件顺序、ID 一致性和输入验证；
- 解释为什么核心代码返回数据，而不是直接打印；
- 运行：

```sh
cargo run -p harness-cli -- "hello"
```

你会看到：

```text
thread/started thread=thread-1
turn/started thread=thread-1 turn=turn-1
item/completed thread=thread-1 turn=turn-1 item=item-1 role=user text="hello"
item/completed thread=thread-1 turn=turn-1 item=item-2 role=assistant text="Echo: hello"
turn/completed thread=thread-1 turn=turn-1
```

</div>

## Predict / 先预测

先别看实现。想一想：一条提示词进入 Harness 后，最少需要发生什么？

一个常见答案是：“读入字符串，然后打印回复。”这能工作，但它把很多事情藏起来了：

- 这条消息属于哪一次对话？
- 一次请求何时开始、何时结束？
- 用户消息和助手消息如何区分？
- UI、日志和测试如何观察过程？
- 输入不合法时，模型有没有被错误调用？

我们的第一个版本不追求聪明。它先追求**可观察、可测试、可解释**。

## Red test / 红灯测试

我们先写行为，再写实现。

第一个测试描述完整事件列表：

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-core/tests/turn_flow.rs:ch00_ordered_events_test}}
```

第一次运行时，`Event`、`Item`、ID 类型和 `run_turn` 都不存在。编译器报告未解析导入。这是 **Red 1**：公开能力还不存在。

接着，我们只加入类型和函数签名，让测试可以编译，但暂时返回空事件列表。测试得到：

```text
left:  []
right: [ThreadStarted, TurnStarted, User Item, Assistant Item, TurnCompleted]
```

这是 **Red 2**：API 已出现，但行为仍不正确。

这两个失败都有价值：

- Red 1 帮我们确定需要什么公开接口；
- Red 2 帮我们确认测试真的在观察行为。

## Read the failure / 读懂失败

编译失败不一定是坏事。关键是失败原因是否准确。

好的 Red：

- 缺少我们正准备实现的 API；
- 实际事件列表与期望列表不同；
- 错误位置指向测试中的行为契约。

不好的 Red：

- 路径拼错；
- 测试文件有语法错误；
- fixture 没准备好；
- 断言永远不会执行。

**测试驱动开发** <span class="term-en">Test-Driven Development (TDD)</span> 不是“先制造任何失败”。它是先得到一个**有意义的失败**。

## Concept / 概念

<div class="learning-card">
<p class="card-label">Agent Harness / 智能体运行框架</p>

模型只是生成候选输出。Harness 负责把一次真实工作组织起来，例如：

- 保存对话状态；
- 调用模型；
- 路由工具；
- 发出进度事件；
- 管理审批、取消和恢复；
- 控制时间、输出和上下文大小。

本章只实现最小竖切：一个输入、一个离线回复、五个事件。

</div>

### Thread、Turn 和 Item

- **Thread / 会话**：一段持续的对话容器。这里不是操作系统线程。
- **Turn / 回合**：从一次用户输入开始，到一次助手回复完成。
- **Item / 条目**：回合中的具体内容，例如用户消息或助手消息。
- **Event / 事件**：告诉外部“刚刚发生了什么”。

关系很简单：

```text
Thread
└── Turn
    ├── User Item
    └── Assistant Item
```

真实 Codex 还会有推理、命令、文件修改和工具调用等 Item。我们暂时不实现它们。

### 用类型区分 ID

下面是本章的领域类型：

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-core/src/lib.rs:ch00_domain_types}}
```

`ThreadId(1)`、`TurnId(1)` 和 `ItemId(1)` 内部都使用 `u64`，但它们是不同类型。

这样做比到处传裸 `u64` 更安全：

- 函数签名能表达用途；
- 编译器能阻止把 `ItemId` 误传成 `ThreadId`；
- 以后改变某一种 ID 的表示方式，不必同时改动所有 ID。

这叫 **新类型模式** <span class="term-en">Newtype Pattern</span>。我们会在第 5 章正式深入。

### 用 enum 表达发生了什么

我们没有使用这些布尔值：

```text
thread_started: bool
turn_started: bool
item_completed: bool
turn_completed: bool
```

因为多个布尔值可以形成混乱组合，例如“回合已完成，但从未开始”。

`Event` 枚举把每一种事件和它需要的数据放在一起。`match` 还会要求调用方处理每一种变体。第 6 章会深入讨论这种设计。

## Why Rust chose this / 为什么这样设计

Rust 经常要求我们把重要状态写进类型里。

它带来的核心好处不是“代码看起来高级”，而是：

- **问题更早暴露**：错误 ID 和遗漏状态更容易在编译期发现；
- **局部推理更容易**：看到函数签名，就知道输入和输出是什么；
- **重构更可靠**：新增枚举变体后，相关 `match` 会提示需要更新的位置；
- **边界更清楚**：核心逻辑返回领域数据，CLI 只负责显示。

代价也真实存在：

- 类型和枚举需要多写一些代码；
- 初学时需要理解更多概念；
- 过早建模会变成不必要的复杂度。

所以我们只建模当前行为，不预建未来十章的架构。

## Fair comparison / 公平对比

用 Python 或 TypeScript 做一个 Echo CLI，通常几行代码就够了。

它们的优势：

- 起步快；
- 原型简洁；
- 对小脚本非常合适。

Rust 的选择：

- 先写更多类型；
- 先明确错误和事件；
- 用编译器保护后续重构。

这里没有绝对赢家。

- 一次性脚本：Python 往往更直接；
- 会持续加入工具、审批、持久化和并发的 Harness：Rust 的显式类型会越来越有价值。

## Green implementation / 最小实现

实现保持同步、离线、标准库优先：

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-core/src/lib.rs:ch00_run_turn}}
```

流程只有四步：

1. `trim` 并验证输入；
2. 调用具体的 `EchoModel`；
3. 构造五个有序事件；
4. 返回 `TurnOutcome`。

### 为什么不是 Provider trait？

我们现在只有一个模型形状。此时创建复杂 Trait，只是在猜未来。

`run_turn_with_model` 接受一个简单函数指针：

```text
fn(&str) -> String
```

它给测试留下一个小接缝，也保持代码容易读。等真正出现多个模型实现和异步流后，我们再引入 Trait。

### 为什么核心不直接打印？

核心返回 `Vec<Event>`，CLI 再决定如何显示：

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-cli/src/render.rs:ch00_render_event}}
```

这叫 **函数式核心、命令式外壳** <span class="term-en">Functional Core, Imperative Shell</span>：

- 核心计算结果；
- 外壳处理终端和进程退出码。

以后同一组事件可以被 CLI、GUI、JSONL 服务或测试使用。

## Refactor / 重构

Green 之后，我们只做了小型重构：

- 把终端渲染放进 `render.rs`；
- 给三种 ID 实现一致的显示格式；
- 保留一个具体 `EchoModel`；
- 不添加 async、Serde、UUID、数据库或插件系统。

重构的安全网不是“我觉得没问题”，而是此前已经通过的行为测试。

## Edge cases / 边界情况

最重要的边界是空输入。

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-core/tests/turn_flow.rs:ch00_blank_prompt_test}}
```

测试传入一个一旦执行就会 `panic!` 的模型函数。

如果验证发生在模型调用之后，测试会立刻失败。因此它证明了：

- 空白输入返回类型化错误；
- 没有事件产生；
- 模型没有被调用。

当前还故意保留一些限制：

- ID 是固定的演示值；
- 只能运行一个回合；
- 没有流式输出；
- 没有持久化；
- 没有真实模型或工具。

## Codex source reading / Codex 源码阅读

**Source fact / 来源事实**：固定版本的 Codex app-server 文档把交互描述为 `Thread → Turn → Item`。它会发送 `thread/started`、`turn/started`、Item 进度和 `turn/completed` 等通知。

**Pinned source / 固定来源**：

- `openai/codex@2df67054232090af8d2fa197c46b994bc2b0dda1`
- [`codex-rs/app-server/README.md`](https://github.com/openai/codex/blob/2df67054232090af8d2fa197c46b994bc2b0dda1/codex-rs/app-server/README.md)
- [`codex-rs/core/src/codex_thread.rs`](https://github.com/openai/codex/blob/2df67054232090af8d2fa197c46b994bc2b0dda1/codex-rs/core/src/codex_thread.rs)

**Our decision / 本书决定**：保留这些公开概念，但重新写一个更小的教学实现。

真实 Codex 还处理：

- 流式增量；
- 工具和命令；
- 审批和权限；
- 中断；
- 持久化与恢复；
- 多种客户端协议。

本章的五个事件只证明生命周期模型可运行，不证明 Codex 兼容性。

## AI candidate review / AI 候选方案审核

假设 AI 一次生成了这样的方案：

```text
读取参数 → 调用在线模型 → 在一个大函数里直接打印 → 使用四个 bool 记录状态
```

它看起来完成得更快，但有几个问题：

- 默认测试需要网络和 API key；
- 核心逻辑与终端输出耦合；
- 多个布尔值允许矛盾状态；
- 裸整数 ID 容易混用；
- 无法精确断言事件顺序；
- 一开始就加入 async、Trait 和 UUID，学习负担过大。

我们不是因为“AI 说错了”就拒绝它，而是用行为测试和设计原则检查它。

AI 给出候选方案。证据决定是否接受。

## Verification / 验证

本章的验证命令：

```sh
cargo test -p harness-core --test turn_flow
cargo test -p harness-cli --test cli
cargo test --workspace
cargo run -p harness-cli -- "hello"
mdbook test book -L target/debug/deps
mdbook build book
```

CLI 测试是真实的 **黑盒测试** <span class="term-en">Black-Box Test</span>：

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-cli/tests/cli.rs:ch00_cli_test}}
```

它不调用内部函数，而是启动编译后的 `harness-cli`，再检查退出状态和完整标准输出。

## Harness status / 当前 Harness 状态

现在 Harness 可以：

- 接收一个非空提示词；
- 生成确定性的离线回复；
- 返回五个有序事件；
- 拒绝空白输入；
- 通过 CLI 显示结果；
- 在没有网络和 API key 的环境中测试。

Parity / 对照级别：**P0 — concept demonstrated**。

下一章不会立刻加入真实模型。我们先确认每个人使用的是同一套可复现工具链。

## Decision record / 决策记录

- **决定**：先构建一个小而完整的同步回合。
- **原因**：读者先看到完整流程，再逐步替换内部部件。
- **决定**：核心返回事件，CLI 负责打印。
- **原因**：分离领域行为和 I/O。
- **决定**：使用函数指针，不提前引入 Provider Trait。
- **原因**：具体实现优先，真实变化出现后再抽象。
- **决定**：固定演示 ID。
- **原因**：本章只验证生命周期；最终 ID 策略属于后续章节。

## Stretch / 进一步练习

不改架构，只做一个小练习：

- 输入 `"  hello  "`；
- 预测 User Item 和 Assistant Item 中保存的文本；
- 写一个测试确认你的预测；
- 再判断“自动 trim”是否应该成为长期产品规则。

不要急着加入 UUID 或真实模型。先练习把产品决定写成可执行测试。
