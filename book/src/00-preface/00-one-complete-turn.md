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

## 先看全局 / Big picture

我们最终想做的，不是一个只会把文字送给模型、再把答案打印出来的聊天脚本。

我们要做的是一个本地 **智能体运行框架** <span class="term-en">Agent Harness</span>。

它以后要负责：

- 保存一段持续的对话；
- 组织一次次用户请求；
- 接收模型产生的消息和工具调用；
- 把进度告诉 CLI 或 GUI；
- 在危险操作前请求批准；
- 处理失败、取消和恢复；
- 控制上下文、时间和输出大小；
- 留下足够证据，让我们知道任务是否真的完成。

这听起来很多。我们当然不会在第一章一次做完。

本章只做一个很小、但完整的版本：

```text
用户输入
→ Harness 开始一个 Thread
→ 开始一个 Turn
→ 记录用户 Item
→ 产生离线回复
→ 记录助手 Item
→ 完成 Turn
```

我们先让这条路真的走通。后面的每一章，再替换或加深其中一层。

<div class="learning-card">
<p class="card-label">现在只要知道 / For now</p>

我们还不需要懂所有 Rust 语法。

这一章先抓住三个重点：

1. Agent Harness 不是模型本身；
2. 一次工作可以被拆成清楚的生命周期；
3. 核心逻辑返回数据，外层程序再决定怎样显示。

</div>

## 概念先行 / Concept first

### 模型和 Harness 有什么不同

模型负责生成候选内容。

Harness 负责把模型放进一个可以工作的系统里。

可以把模型想成一台很有能力的发动机。发动机很重要，但一辆可以安全驾驶的车还需要方向、刹车、仪表、车身和规则。Agent Harness 就是模型周围那套运行系统。

这只是帮助理解的类比，不是严格定义。真正的关键是：

> 模型回答“接下来可能做什么”；Harness 决定这次工作怎样开始、怎样进行、能做什么、何时结束，以及怎样被验证。

OpenAI 当前的 Codex 平台文档也把 Harness 描述为模型周围的执行系统：它管理上下文、工具、进度、失败、审批和跨回合工作。

### Thread、Turn、Item 和 Event

我们先认识四个会贯穿全书的概念。

#### Thread / 会话

**会话** <span class="term-en">Thread</span> 是一段可以持续下去的对话。

这里的 Thread 不是操作系统线程。它更像一个项目对话容器：前面的消息和结果可以成为后面回合的上下文。

#### Turn / 回合

**回合** <span class="term-en">Turn</span> 从一次用户输入开始，包含模型和工具随后完成的工作，最后以完成、失败或中断结束。

本章只有最简单的完成状态。

#### Item / 条目

**条目** <span class="term-en">Item</span> 是回合中的一项具体内容。

本章只有两种：

- 用户消息；
- 助手消息。

真实 Codex 还会有命令执行、文件修改、推理摘要、工具调用等 Item。

#### Event / 事件

**事件** <span class="term-en">Event</span> 表示“刚刚发生了什么”。

例如：

- Thread 已开始；
- Turn 已开始；
- 一个 Item 已完成；
- Turn 已完成。

Event 让 CLI、GUI、日志和测试都可以观察同一条运行过程，而不用钻进核心函数内部猜状态。

它们的关系可以先这样理解：

```text
Thread
└── Turn
    ├── User Item
    └── Assistant Item

运行过程
├── ThreadStarted
├── TurnStarted
├── ItemCompleted(User)
├── ItemCompleted(Assistant)
└── TurnCompleted
```

如果这几个词现在还有点陌生，没有关系。我们马上运行一次，看到实际输出后会更具体。

## 本章要完成什么 / Product goal

本章要让下面这条命令在完全离线的情况下运行：

```sh
cargo run -p harness-cli -- "hello"
```

我们暂时使用一个确定性的 Echo model。它不会访问网络，也不需要 API key，只会把输入变成：

```text
Echo: <输入>
```

这样做不是为了假装它有智能，而是为了先把 Harness 生命周期做对。

本章的公开行为是：

1. 非空输入产生五个有序事件；
2. 用户和助手 Item 使用不同 ID；
3. 同一个回合中的 ThreadId 和 TurnId 保持一致；
4. 空白输入在调用模型之前被拒绝；
5. 核心只返回事件，CLI 负责打印。

本章不会加入：

- 真实模型；
- streaming；
- async runtime；
- tools；
- permissions；
- persistence；
- MCP；
- 随机或全局 ID。

这些都会在有真实需要时再出现。

## 先看运行结果 / See it run

现在先运行：

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

我们一行一行看。

### 第一行：Thread 开始

```text
thread/started thread=thread-1
```

Harness 创建了一个对话容器。现在 ID 固定为 `thread-1`，只是为了让第一个例子确定、容易测试。

### 第二行：Turn 开始

```text
turn/started thread=thread-1 turn=turn-1
```

同一段对话里，第一次用户请求开始执行。

### 第三行：用户 Item 完成

```text
item/completed ... item=item-1 role=user text="hello"
```

用户输入已经被规范化并记录。它属于 `thread-1` 和 `turn-1`。

### 第四行：助手 Item 完成

```text
item/completed ... item=item-2 role=assistant text="Echo: hello"
```

离线模型产生回复。它有不同的 ItemId，但仍属于同一个 Thread 和 Turn。

### 第五行：Turn 完成

```text
turn/completed thread=thread-1 turn=turn-1
```

这次回合到这里结束。

我们现在已经有了一个小而完整的运行结果。接下来再拆开代码，就知道每个类型和函数在服务什么。

## 拆开代码 / Read the code

代码分成三层：

```text
harness-core
├── 领域类型
└── 运行一个回合

harness-cli
└── 把事件变成人能读的文字
```

核心不认识终端。CLI 也不决定生命周期。这个边界会在后面越来越重要。

### 第一层：用类型表示领域概念

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-core/src/lib.rs:ch00_domain_types}}
```

这段代码看起来比一个简单 Echo 程序多了不少内容。现在不用逐行记住。

先看它表达了什么：

- `ThreadId`、`TurnId`、`ItemId` 是三种不同 ID；
- `Item` 只能是 `User` 或 `Assistant`；
- `Event` 列出了四类生命周期事件；
- 每个事件同时携带它需要的数据。

#### 为什么不用一个 `u64`

三种 ID 内部暂时都只是 `u64`。

但下面两个值表达不同意思：

```text
ThreadId(1)
ItemId(1)
```

如果所有函数都接收裸 `u64`，我们很容易把 Item ID 传到 Thread ID 的位置，而且编译器无法帮助。

把基础类型包成不同类型，叫 **新类型模式** <span class="term-en">Newtype Pattern</span>。完整语法会在第 5 章再讲。现在只需要知道：相同的数字，不等于相同的业务含义。

#### 为什么 Event 用 enum

我们也可以用几个布尔值：

```text
thread_started
turn_started
item_completed
turn_completed
```

问题是，布尔值可以组成很多没有意义的组合，例如：

```text
turn_completed = true
turn_started = false
```

`enum` 让我们列出“可能发生的事件”。每种事件还能带上自己的数据。

想继续了解，可以查看：

- [Rust Book：Structs](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [Rust Book：Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html)

现在不需要一次读完。先知道这些类型是在描述 Harness，而不是为了展示语法。

### 第二层：运行一个回合

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-core/src/lib.rs:ch00_run_turn}}
```

把细节放在一边，这个函数只做四件事：

1. 用 `trim()` 去掉输入前后的空白；
2. 空输入立即返回 `RunTurnError::BlankPrompt`；
3. 调用传入的模型函数；
4. 按顺序构造五个 Event 并返回。

`Result<T, E>` 表示函数可能成功，也可能失败：

- `Ok(TurnOutcome)`：回合完成；
- `Err(RunTurnError::BlankPrompt)`：输入不合法。

可以先把它理解成“成功结果或失败原因”。完整错误处理会在第 7 章展开：

- [Rust Book：Recoverable Errors with Result](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)

#### 为什么先用函数指针，不马上做 Trait

`run_turn_with_model` 接受：

```text
fn(&str) -> String
```

它的意思是：给我一个接收字符串切片、返回 `String` 的函数。

我们现在只有一个真正的模型形状，也没有异步流。此时马上设计完整 Provider Trait，只是在猜未来。

这个小接缝已经足够：

- 默认运行使用 `EchoModel::respond`；
- 测试可以换入一个确定性函数；
- 空输入测试可以证明模型没有被调用。

等到后面出现多个 Provider、异步返回和流式事件时，再让真实需求推动 Trait 设计。

关于普通函数，可以先看：

- [Rust Book：Functions](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html)

### 第三层：CLI 只负责显示

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-cli/src/render.rs:ch00_render_event}}
```

`render_event` 使用 `match`，把每一种 Event 转成一行文字。

核心返回数据，外层处理 I/O，这种组织方式常被称为 **函数式核心、命令式外壳** <span class="term-en">Functional Core, Imperative Shell</span>。

它的好处很实际：

- 测试可以直接比较 Event，不需要截取终端输出；
- 将来 GUI 可以使用同一组 Event；
- JSONL app server 可以换一种序列化方式；
- 核心逻辑不会因为终端颜色或排版改变而改变。

这里的 `match` 还会要求我们处理所有 Event 变体。以后新增事件时，编译器会指出哪些显示代码需要更新。

## 这段代码怎样长出来 / TDD story

看到完整结果和代码以后，我们再回看它的开发过程。

这正是本书会一直使用的 **测试驱动开发** <span class="term-en">Test-Driven Development (TDD)</span>。

Kent Beck 在 TDD 的实例中不断使用很短的循环：先加入一个小测试，看见它失败；做一个小改动；重新运行；最后去掉重复。重点不是“红灯越多越专业”，而是让每一步都有清楚反馈。

### 行为契约

第一条行为不是“实现 `Event` 枚举”，而是：

> 给定一个非空提示词，运行一次离线回合后，应产生一条用户 Item、一条助手 Item，并按稳定顺序发出生命周期事件。

测试从使用者角度写出完整期望：

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-core/tests/turn_flow.rs:ch00_ordered_events_test}}
```

### Red 1：公开能力还不存在

第一次运行时：

- `Event` 不存在；
- `Item` 不存在；
- 三种 ID 不存在；
- `run_turn` 不存在。

测试无法编译。

这不是最终失败，但它有意义：测试正在要求一个我们确实准备提供的公开 API。

### Red 2：API 有了，行为还没有

下一步只加入类型和函数签名，并暂时返回空事件。

测试能够编译，却得到：

```text
actual:   []
expected: [ThreadStarted, TurnStarted, User Item, Assistant Item, TurnCompleted]
```

这个失败比 Red 1 更具体。它证明测试正在观察事件内容和顺序，而不是只检查“程序能编译”。

### Green：加入最小正确行为

然后才加入：

- 输入验证；
- Echo 回复；
- 五个事件；
- 固定教学 ID；
- `TurnOutcome`。

测试转绿。

“最小”不等于故意写一段烂代码。它表示只处理当前已经定义的行为，不提前加入 UUID、数据库或异步框架。

### Refactor：在绿色安全网下整理

行为稳定后，再把显示逻辑放到 CLI，并统一 ID 的显示方式。

每次整理后重新运行测试。这样我们不是靠记忆确认“应该没有破坏”，而是让既有行为替自己说话。

### 边界：空输入不能启动模型

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-core/tests/turn_flow.rs:ch00_blank_prompt_test}}
```

测试传入一个只要被调用就会 `panic!` 的模型函数。

如果 `run_turn_with_model` 先调用模型、后检查输入，测试马上失败。

因此这条测试支持三个结论：

- 空白输入返回类型化错误；
- 没有回合事件；
- 模型工作没有开始。

它仍然不能证明所有输入处理都正确。测试只覆盖了自己明确检查的条件。Rust 官方测试章节也提醒我们：类型系统和测试可以承担很多正确性工作，但都不能单独证明所有缺陷不存在。

想了解 Rust 测试机制，可以查看：

- [Rust Book：Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust Book：Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)

## 为什么 Rust 这样设计 / Why Rust

本章出现的 Rust 代码有一个共同方向：把重要区别写得更明确。

### 类型让含义进入接口

`ThreadId` 和 `ItemId` 不再只是注释中的约定。它们成为不同类型。

收益：

- 更难把 ID 放错位置；
- 读函数签名时更容易理解；
- 重构一种 ID 时影响更集中。

成本：

- 类型数量增加；
- 初学时需要多认识几个名字；
- 如果没有真实区别，过度包装会制造噪音。

所以我们只为已经存在的领域概念建模。

### enum 让可能性集中在一个地方

`Event` 告诉我们系统目前有哪些公开事件。`match` 让调用方正面处理这些可能性。

这比到处散落字符串或布尔值更适合长期演进，但小脚本可能根本不需要这层建模。

### Result 让失败出现在返回类型里

空输入不是隐藏异常，也不是打印一行错误后继续。

调用方必须面对：

```text
成功完成
或
BlankPrompt
```

它增加了显式处理，却减少了“失败被悄悄忽略”的空间。

### 核心与 I/O 分开

让核心返回 Event，多写了一层边界。但它换来更容易测试、更容易复用，也更容易将来接入不同客户端。

这不是 Rust 独有的思想。Rust 的类型系统只是让这个边界更容易被表达和检查。

## 公平对比 / Fair comparison

用 Python 或 TypeScript 写一个 Echo CLI，可以短很多。

如果目标只是：

```text
读取一句话
→ 打印 Echo
→ 程序结束
```

它们往往更合适：

- 启动更快；
- 样板更少；
- 修改小脚本很直接。

Rust 在本章提前付出了建模成本：

- 三种 ID；
- Item；
- Event；
- Result；
- Core 和 CLI 边界。

这个成本只有在系统继续成长时才逐渐回报。

当 Harness 开始加入工具、审批、恢复和并发，显式类型与穷尽匹配可以帮助我们更安全地改动。反过来，如果项目永远只有十行，Rust 的结构可能显得过重。

好的工程判断不是“Rust 永远更好”，而是知道当前问题是否值得这些约束。

## 自己动手 / Try it

### 练习一：换一个输入

```sh
cargo run -p harness-cli -- "explain this repository"
```

观察：

- 哪些 ID 没有改变；
- 哪两行文字改变；
- 事件顺序有没有改变。

### 练习二：看看输入规范化

```sh
cargo run -p harness-cli -- "   hello   "
```

用户 Item 和 Echo 回复中的前后空白会被去掉。

回到 `run_turn_with_model`，找出负责这件事的一行。现在不用深入 `&str` 和 `String` 的所有权差异，第 9 到第 11 章会专门处理。

### 练习三：让测试真的抓到回归

临时把用户 Item 和助手 Item 的顺序交换，然后运行：

```sh
cargo test -p harness-core --test turn_flow one_prompt_produces_ordered_events
```

你应该看到完整值比较失败。

恢复代码，再运行同一条测试。不要提交故意破坏的版本。

### 练习四：运行真实 CLI 黑盒测试

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-cli/tests/cli.rs:ch00_cli_test}}
```

这个测试不是直接调用 `render_event`。它启动 Cargo 构建出的真实二进制，再比较标准输出，所以它是 **黑盒测试** <span class="term-en">Black-Box Test</span>。

## Codex 源码阅读 / Source reading

本书的教学实现不是凭空选择 Thread、Turn 和 Item。

**Source fact / 来源事实：** Codex app-server 把 Thread 定义为一段对话，把 Turn 定义为一次用户请求和随后发生的智能体工作，把 Item 定义为其中一项输入或输出。Turn 会流式产生 Item 与生命周期事件。

当前官方入口：

- [Codex app-server](https://developers.openai.com/codex/app-server)

本书固定阅读的源码版本：

- `openai/codex@2df67054232090af8d2fa197c46b994bc2b0dda1`
- [`codex-rs/app-server/README.md`](https://github.com/openai/codex/blob/2df67054232090af8d2fa197c46b994bc2b0dda1/codex-rs/app-server/README.md)
- [`codex-rs/core/src/codex_thread.rs`](https://github.com/openai/codex/blob/2df67054232090af8d2fa197c46b994bc2b0dda1/codex-rs/core/src/codex_thread.rs)

**Our decision / 本书决定：** 保留这些容易理解的公开概念，重新写一个同步、离线、只有五个事件的教学版本。

真实 Codex 还处理：

- 增量 streaming；
- command 和 file change Item；
- tools；
- sandbox 与 approval；
- interruption；
- persistence；
- context compaction；
- 多种客户端和协议。

我们的代码只能说明“这个最小生命周期可以运行和测试”，不能说明它兼容 Codex。

## AI 候选方案审核 / Review an AI candidate

假设 AI 一次生成以下方案：

```text
在 main 函数中读取参数
→ 调用在线模型
→ 直接 println!
→ 用四个 bool 记录开始和完成状态
→ 顺便加入 async、Trait、Serde 和 UUID
```

看起来很完整，但我们先问几个问题。

### 默认测试能离线运行吗

不能。它需要网络和 API key。

### 可以直接比较生命周期吗

很难。核心只打印，没有返回 Event。

### 状态会不会矛盾

会。多个布尔值可以形成没有意义的组合。

### 现在真的需要这些抽象吗

未必。一个 Provider 还不足以证明 Trait 形状，固定单回合也不需要 UUID 服务。

### 怎样做更小的选择

- 用 EchoModel 保持离线；
- 用 Event 表达公开过程；
- 核心返回数据；
- 用函数接缝支持测试；
- 等真实需求出现，再加入 async 和 Trait。

我们不是因为 AI “不可信”就拒绝它。我们是把它当作候选设计，再用行为、成本和测试检查。

## 验证与边界 / Verify and limits

运行本章聚焦验证：

```sh
cargo test -p harness-core --test turn_flow
cargo test -p harness-cli --test cli
cargo run -p harness-cli -- "hello"
```

再运行累计验证：

```sh
cargo test --workspace --locked
mdbook test book -L target/debug/deps
mdbook build book
make pdf
```

<div class="learning-card">
<p class="card-label">当前 Harness 状态 / Harness status</p>

现在它可以：

- 接收一个非空提示词；
- 运行一个确定性的离线回合；
- 返回五个有序 Event；
- 由 CLI 输出事件；
- 通过行为测试验证顺序、ID 和空输入。

现在它还不能：

- 继续第二个 Turn；
- 使用真实模型；
- streaming；
- 调用工具；
- 保存历史；
- 取消或恢复；
- 生成全局唯一 ID。

</div>

## 本章决定 / Decision record

- 先完成一个小而完整的竖切，不从语法目录开始。
- Thread、Turn、Item 和 Event 借鉴公开 Codex 概念，但实现原创且范围更小。
- 核心返回领域事件，不直接打印。
- 使用新类型区分三种 ID。
- 使用 enum 表达公开事件。
- 使用函数指针作为当前最小模型接缝。
- 默认测试离线、确定。
- 固定 ID 只服务本章教学，不是最终设计。

## 本章小结与下一步 / Recap and next step

这一章最重要的，不是记住 `struct`、`enum` 或 `match` 的全部语法。

我们先建立了一张地图：

```text
Agent Harness
├── 组织 Thread
├── 运行 Turn
├── 产生 Item
└── 发出 Event
```

然后让它真的运行，拆开了类型、核心和 CLI 三层，也看见测试怎样从一个行为契约推动最小实现。

下一章，我们暂时不增加新的 Agent 功能。

我们会先把开发环境变成可复查的证据：为什么同一份代码在不同机器上可能得到不同结果，以及 Rust toolchain、Edition、MSRV、Workspace 和 Lockfile 分别在控制什么。
