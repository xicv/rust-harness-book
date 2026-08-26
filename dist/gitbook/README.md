# Introduction / 导读

我们先把目标说清楚。

这不是一本把 Rust 语法从头排到尾、让读者逐条记忆的手册。它也不是一本“作者已经全部懂了，再把标准答案交给你”的书。

我们自己也是学习者。

我们一边学习 Rust，一边完成一个真实项目：一个能在本地运行、接受测试、讲得清楚，也能逐步验证的 **智能体运行框架** *Agent Harness*。

## 我们到底要做什么

现在很多人已经可以让模型生成代码。

真正困难的部分，往往发生在模型回答之外：

- 这次工作属于哪段对话？
- 一次任务什么时候开始，什么时候结束？
- 模型可以读取什么、修改什么？
- 工具调用失败后怎样处理？
- 哪个动作需要人批准？
- 中途取消后，状态还能不能恢复？
- 上下文越来越长时，怎样控制预算？
- AI 说“已经完成”，我们怎样独立验证？

负责组织这些事情的系统，就是 Agent Harness。

全书从一个最小离线回合开始，再逐章加入类型、所有权、接口、异步、工具、权限、持久化、恢复、协议和评测。最后得到一个真的可以运行和验证的本地 Harness。

我们也会持续阅读 OpenAI Codex 的公开源码和 app-server 协议。不是为了照抄一个缩小版 Codex，而是学习一个大型 Rust 工程怎样处理真实的状态、边界和失败。

## 为什么用 Rust 来学这些

Rust 值得学，不只是因为它运行快。

它会不断追问一些软件工程里的基本问题：

- 谁拥有这个值？
- 谁可以修改它？
- 失败有没有写进接口？
- 这个状态是不是可能自相矛盾？
- 这个对象能不能安全地跨线程？
- 资源什么时候释放？
- 一个抽象的成本在哪里？

这些问题在其他语言里一样存在，只是 Rust 更常要求我们提早把答案写进类型和接口。

这会带来额外工作。Rust 的代码有时比 Python、TypeScript 或 Go 更早出现类型和边界设计。编译也可能更慢，学习曲线也更陡。

但这种“先把重要问题说清楚”的习惯，正适合用来学习怎样建设一个长期演进的 Agent Harness。

## 我们怎样学

我们不要求先把 Rust 全部搞懂再开始。

第一步是跑通一个完整流程。看到结果之后，我们再一层一层回来理解：

```text
先认识概念
→ 看见真实结果
→ 拆开代码
→ 修改一个行为
→ 用测试确认
→ 再深入一层
```

有些语法第一次出现时，你可能还不完全明白。没关系。正文会告诉你“现在只要知道什么”，并给出 Rust 官方文档链接。相关概念会在后面的真实问题里再次出现。

学习 Rust 很少是一次读完就全部掌握。[Rust 项目 2026 年发布的学习研究](https://blog.rust-lang.org/2026/06/25/vision-doc-journeys-to-learning-rust/)也指出，不同背景的学习者会走不同路线，很多人需要用官方文档、练习、视频、真实项目和编译器反馈多次建立同一个心智模型。

所以这本书允许我们回头，允许我们先做后懂，也允许我们清楚地说：“这一部分我现在还不知道。”

## 随手运行 Rust / Run it as you read

读代码最直观的方法，不是只看，而是马上改一点，再看看结果怎样变化。

这本书的 HTML 版本会为适合单文件运行的例子提供 **即时实验** *Interactive Lab*：

发布后的权威互动版位于 [GitHub Pages](https://xicv.github.io/rust-harness-book/)。如果该地址还没有完成首次 部署，请先在本地运行 `mdbook build book`；配置好的地址不等于已经上线。

1. 直接修改代码；
2. 点击运行按钮；
3. 在代码下面查看输出；
4. 点击撤销，回到原来的版本。

先试一下。把 `"hello"` 改成你的名字，或者把 `"Echo:"` 改成另一段文字，然后重新运行。

```rust
#[derive(Debug)]
enum Event {
    ThreadStarted,
    TurnStarted,
    ItemCompleted {
        role: &'static str,
        text: String,
    },
    TurnCompleted,
}

fn run_turn(prompt: &str) -> Vec<Event> {
    let prompt = prompt.trim();

    vec![
        Event::ThreadStarted,
        Event::TurnStarted,
        Event::ItemCompleted {
            role: "user",
            text: prompt.to_owned(),
        },
        Event::ItemCompleted {
            role: "assistant",
            text: format!("Echo: {prompt}"),
        },
        Event::TurnCompleted,
    ]
}

fn main() {
    let events = run_turn("hello");

    assert_eq!(events.len(), 5);

    for event in events {
        println!("{event:?}");
    }
}
```

这段代码是一个为了浏览器实验而缩小的生命周期模型，不是生产 Harness 的第二份实现。它保留五个事件，让我们可以直观看到“改输入、改输出、改顺序”会发生什么。

> **Playground 边界 / Playground limits**
>
> 浏览器实验很方便，但它不是本书的最终验证证据：
>
> - 点击运行会把这段代码发送到 Rust Playground；
> - Playground 使用读者访问时的 stable Rust，不保证永远等于本书固定的 Rust `1.98.0`；
> - Playground 内运行的程序没有外部网络，并受到时间和内存限制；
> - 它只适合单文件例子，也不能引用本仓库的本地 crate；
> - 不要把密码、API key、私人代码或公司代码贴进在线 Playground。
>
> 本书固定版本的 Cargo 测试和 CI 仍然是发布证据。完整 Harness、跨 crate 测试和文件系统实验要在本地项目中运行。详细规则见 [`docs/15-INTERACTIVE-LABS.md`](https://github.com/xicv/rust-harness-book/blob/main/docs/15-INTERACTIVE-LABS.md)。

这和 Jupyter Notebook 的学习感觉相近：改一小段，立刻看反馈。不同的是，Rust Playground 适合独立小例子；需要保持多段状态或操作完整 Workspace 时，就使用本地 Cargo、浏览器云开发环境，或可选的 Evcxr Jupyter lab。

## 每章怎样展开

章节的阅读顺序会保持自然：

1. 先从全局说明为什么要做；
2. 介绍必要概念；
3. 说明本章要给产品加入什么；
4. 先运行，看看结果；
5. 分层读代码；
6. 回看代码怎样通过 TDD 长出来；
7. 解释 Rust 为什么这样设计；
8. 与其他语言或方案公平比较；
9. 亲手做一个小修改；
10. 阅读固定版本的 Codex 源码；
11. 验证结果，写清边界和下一步。

代码的开发顺序仍然是 **Red → Green → Refactor**。但阅读顺序不需要从失败日志开始。我们先知道问题和概念，再看测试怎样帮助我们把设计做实。

## TDD 在这本书里是什么意思

**测试驱动开发** *Test-Driven Development (TDD)* 不是为了追求更多测试，也不是先写一段注定失败的代码做仪式。

我们采用很小的循环：

```text
写下一个想要的行为
→ 运行并确认它为正确原因失败
→ 加入最小正确实现
→ 再运行测试
→ 在绿色状态下去掉重复、改善结构
```

测试让我们的下一步更容易判断，也给重构带来安全网。但一个通过的测试只说明它检查到的条件成立，并不证明软件没有其他缺陷。

在 AI-native 开发里，这一点更重要。AI 可以一次生成很多代码；测试帮助我们把变化重新切回可观察、可验证的小步。

## 这本书负责什么，官方文档负责什么

我们负责建立心智模型、工程上下文和完整项目。

我们不会逐行解释所有语法。遇到具体语言机制时，会链接：

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust Standard Library](https://doc.rust-lang.org/std/)
- [The Cargo Book](https://doc.rust-lang.org/cargo/)
- [The rustup book](https://rust-lang.github.io/rustup/)
- 固定 commit 的 [OpenAI Codex](https://github.com/openai/codex)

正文先让你理解“为什么需要它”。官方资料帮助你继续查询“完整规则是什么”。

## 我们的证据规则

无论代码来自人还是 AI，都不能因为“看起来合理”就算完成。

我们的证据包括：

- 编译器；
- 聚焦测试；
- 累计集成测试；
- 真实 CLI 输出；
- HTML 和 PDF 构建；
- 固定版本官方文档；
- 固定 commit 的源码和测试；
- 清楚的限制说明。

AI 提供候选方案。证据决定是否接受。

## 最后会得到什么

完成这本书时，我们希望手里有两个成果。

一个是能够工作的 Rust Agent Harness。

另一个更重要：面对一个复杂软件问题时，我们知道怎样从概念开始，怎样做小而完整的设计，怎样用测试推进，怎样阅读大型源码，怎样审查 AI 生成的改动，并且怎样诚实地说清楚我们已经证明了什么、还没有证明什么。

现在，我们从第一个很小但完整的回合开始：

[先跑通一个回合 / Run One Complete Turn](00-preface/00-one-complete-turn.md)。
