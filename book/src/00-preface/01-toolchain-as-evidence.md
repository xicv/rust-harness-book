<!--
chapter_id: ch01
chapter_status: completed
chapter_goal: Pin and verify the Rust workspace.
rust: 1.98.0
codex_commit: 2df67054232090af8d2fa197c46b994bc2b0dda1
new_terms: [toolchain, workspace, msrv, lockfile, rust-edition, dependency-resolver, crate, reproducibility]
examples: [ch01-toolchain-report, ch01-doctor-test]
-->

# 工具链就是证据 / The Toolchain Is Evidence

> **Status / 状态:** Complete
> **Harness milestone / Harness 里程碑:** Pin and verify the Rust workspace.

<div class="learning-card">
<p class="card-label">Outcome / 本章成果</p>

完成本章后，你可以：

- 解释 **工具链** <span class="term-en">Toolchain</span>、**Rust 版本代际** <span class="term-en">Rust Edition</span>、**最低支持 Rust 版本** <span class="term-en">Minimum Supported Rust Version (MSRV)</span> 和 **依赖解析器** <span class="term-en">Dependency Resolver</span> 的区别；
- 看懂根目录的 `rust-toolchain.toml`、`Cargo.toml` 和 `Cargo.lock`；
- 运行一个真实的工具链检查：

```sh
cargo run -p harness-cli --locked -- --doctor
```

预期输出：

```text
toolchain/status ok
rust/pinned 1.98.0
rustc/active 1.98.0
cargo/active 1.98.0
edition 2024
resolver 3
lockfile present
```

- 使用 check-only 命令验证项目，而不制造格式化噪音；
- 理解“在我的机器上可以运行”为什么还不够。

</div>

## Predict / 先预测

Chapter 0 已经通过测试。那我们能否直接说：“所有读者都能得到同样结果”？

还不能。

下面任何差异都可能改变结果：

- Rust 编译器版本；
- Edition；
- Cargo 依赖解析规则；
- Clippy 和 rustfmt 组件；
- 依赖版本；
- 操作系统和 CPU；
- 本地环境变量。

测试结果只有连同环境一起记录，才有 **可复现性** <span class="term-en">Reproducibility</span>，也才是可复查的证据。

## Red test / 红灯测试

我们先为 CLI 定义一个新行为：`--doctor` 必须读取项目中的固定配置，并与实际运行的 `rustc`、`cargo` 对照。

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-cli/tests/doctor.rs:ch01_doctor_test}}
```

第一次运行时，CLI 还不知道 `--doctor`。它把这个参数当成普通提示词，输出五个 Harness 事件。

测试失败的原因很清楚：

```text
expected: toolchain/status ok ...
actual:   thread/started ...
```

这就是本章的 Red：产品缺少一个可观察的工具链检查。

## Read the failure / 读懂失败

这个失败告诉我们两件事：

- Chapter 0 的旧行为仍然正常；
- Chapter 1 的新命令还没有路由。

这比“某个 shell 脚本在我的电脑上跑过”更可靠，因为测试启动的是 Cargo 构建出的真实二进制。

本章 Green 后，原来的 Echo 回合测试也必须继续通过。新增能力不能破坏旧能力。

## Concept / 概念

### Toolchain / 工具链

Rust 工具链不只是 `rustc`。

本项目至少使用：

- `rustc`：编译器；
- `cargo`：构建、依赖和测试工具；
- `rustfmt`：格式检查；
- `clippy`：常见问题和代码质量检查；
- 标准库和目标平台支持。

`rustup` 负责安装和选择这些工具链组件。

我们的固定文件是：

```toml
{{#include ../../../rust-toolchain.toml}}
```

它表达：

- 使用 Rust `1.98.0`；
- 只安装 minimal profile；
- 加上 Clippy 和 rustfmt。

### Edition 不是编译器版本

本项目使用 **Rust Edition 2024**：

```toml
[workspace.package]
edition = "2024"
rust-version = "1.98"
```

两者作用不同：

- `edition = "2024"` 选择语言兼容边界和部分默认规则；
- `rust-version = "1.98"` 声明包支持的最低 Rust 版本，也叫 **最低支持 Rust 版本** <span class="term-en">Minimum Supported Rust Version (MSRV)</span>；
- `rust-toolchain.toml` 的 `1.98.0` 固定本项目实际使用的精确工具链。

简单记忆：

```text
Edition        → 这份源码按哪一代规则理解
rust-version   → 至少需要多新的 Rust
rust-toolchain → 这个仓库现在实际使用哪一版工具
```

### Workspace / 工作空间

根目录是一个虚拟 **工作空间** <span class="term-en">Workspace</span>：

```toml
[workspace]
members = [
    "crates/harness-cli",
    "crates/harness-core",
    "crates/harness-test-support",
]
resolver = "3"
```

Workspace 让多个 **编译单元** <span class="term-en">Crate</span> 共享：

- 一个 `Cargo.lock`；
- 一个 `target` 目录；
- 统一的 Edition、MSRV 和 lint 设置；
- 从根目录运行的累计检查。

### Resolver 3 / 依赖解析器 3

Edition 2024 的包默认使用 **依赖解析器** <span class="term-en">Dependency Resolver</span> 3。我们的根目录是虚拟 Workspace，没有自己的 `[package]`，因此显式写出：

```toml
resolver = "3"
```

Resolver 3 会在依赖选择时考虑依赖声明的 Rust 版本要求。它不是安全扫描器，也不会替你判断某个 crate 是否设计良好。

### Lockfile / 锁文件

`Cargo.toml` 表达允许的依赖范围，`Cargo.lock` 记录一次具体解析结果。

本项目包含可执行程序，所以提交根目录 **锁文件** <span class="term-en">Lockfile</span>：

- 本地、CI 和其他开发者更容易使用同一依赖图；
- `--locked` 可以阻止 Cargo 静默改写解析结果；
- 依赖更新会成为可审查的 diff。

锁文件不等于供应链安全。它固定“下载什么”，但不证明“这些代码值得信任”。后续章节会加入审计和来源检查。

## Why Rust chose this / 为什么这样设计

Rust 希望同时支持小程序、库、CLI、服务器和系统软件。单靠一个全局编译器版本很难满足所有项目。

所以它把几层决定分开：

- rustup 管理机器上的多个工具链；
- Toolchain file 把选择放进仓库；
- Cargo manifest 描述包、Workspace 和依赖；
- Lockfile 固定一次解析结果；
- CI 在干净环境中重复验证。

这体现一个通用工程原则：

> **重要环境不要只存在于某个人的电脑或记忆里。把它写进版本控制。**

收益：

- 新成员少猜一步；
- CI 与本地更接近；
- 升级变成显式变更；
- 旧版本可以重建和调查。

成本：

- 需要维护多个版本字段；
- 工具更新不能完全自动接受；
- 平台差异仍然需要测试；
- 过度固定会错过安全修复。

所以“固定”和“更新”要同时存在：固定当前证据，定期审查新版本。

## Fair comparison / 公平对比

其他生态也有类似做法：

- Node.js 常用版本管理器、`package.json` 和 lockfile；
- Python 常用虚拟环境、项目元数据和锁定工具；
- Go 在模块文件中记录语言或工具链要求；
- Swift Package Manager 也会记录解析结果。

Rust 的优势不是“只有 Rust 才能复现”。它的特点是 rustup、Cargo、Edition、组件和编译器诊断结合得很紧。

另一方面：

- Go 的默认工具链体验通常更少选择、更容易上手；
- Python 对一次性脚本更轻；
- Node.js 的前端生态集成更成熟。

Rust 提供更明确的控制，但也要求项目维护者做更多决定。

## Green implementation / 最小实现

`--doctor` 不维护硬编码的第二份配置。它从当前目录向上寻找项目根目录，然后读取版本控制中的文件：

- `rust-toolchain.toml`；
- 根 `Cargo.toml`；
- `Cargo.lock`。

接着，它运行 `rustc --version` 和 `cargo --version`，对比精确版本。它只读取和报告，不会安装、升级或修改工具。

```rust,ignore,noplayground
{{#rustdoc_include ../../../crates/harness-cli/src/doctor.rs:ch01_toolchain_report}}
```

这个实现没有引入 TOML 解析库，因为当前只读取几个简单、受控的键。未来配置变复杂时，应改用正式解析器，而不是继续扩展手写解析。

退出码也有意义：

- `0`：配置和实际工具链匹配；
- `1`：检查完成，但版本不匹配；
- `2`：检查本身无法完成。

## Refactor / 重构

本章只做了三项小重构：

- 在 CLI 中把普通提示词和 `--doctor` 路由分开；
- 把工具链读取放进 `doctor.rs`；
- 为检查错误保留来源错误，方便诊断。

我们没有：

- 引入 CLI 框架；
- 写通用配置系统；
- 自动安装或修改用户工具链；
- 在程序中执行 `rustup update`；
- 把工具链逻辑塞进 `harness-core`。

`harness-core` 继续只关心领域行为。

## Edge cases / 边界情况

工具链检查需要面对几种失败：

- `rustc` 不在 `PATH`；
- `cargo --version` 返回失败；
- 输出不是 UTF-8；
- 固定文件缺少预期字段；
- 版本不同；
- Lockfile 缺失。

当前 `--doctor` 只检查最小基线。它还没有检查：

- macOS、Linux 和 Windows 的目标三元组；
- rust-analyzer；
- mdBook 或 Typst；
- C 编译器和链接器；
- CPU 架构；
- 依赖来源或漏洞。

不要把一个绿色 `doctor` 结果理解为“整台机器完全相同”。

## Codex source reading / Codex 源码阅读

**Source fact / 来源事实**：固定版本的 Codex Rust Workspace 使用 Edition 2024，并由根 `codex-rs/Cargo.toml` 管理大量 crate。它把协议、核心、工具、沙箱和客户端拆成清楚的包边界。

**Pinned source / 固定来源**：

- `openai/codex@2df67054232090af8d2fa197c46b994bc2b0dda1`
- [`codex-rs/Cargo.toml`](https://github.com/openai/codex/blob/2df67054232090af8d2fa197c46b994bc2b0dda1/codex-rs/Cargo.toml)
- [`AGENTS.md`](https://github.com/openai/codex/blob/2df67054232090af8d2fa197c46b994bc2b0dda1/AGENTS.md)

**Our decision / 本书决定**：从三个 crate 开始，只在出现真实边界时再拆分。

大项目的 Workspace 不是目标本身。它是管理真实依赖方向的工具。过早拆成几十个 crate，只会让初学者迷路。

## AI candidate review / AI 候选方案审核

AI 可能建议：

```text
永远使用 latest stable
删除 Cargo.lock
让 CI 自动 cargo update
运行 cargo fmt 自动修正所有文件
如果本地失败，就换成 nightly
```

这些建议听起来省事，但会削弱证据：

- “latest” 会随时间改变；
- 自动更新会把依赖变化混进功能变更；
- 全仓格式化会制造无关 diff；
- 随意切 nightly 会改变问题，而不是解释问题；
- 删除 Lockfile 会让可执行程序的依赖解析漂移。

更好的做法：

- 固定当前基线；
- 用独立变更升级；
- 查看 release notes；
- 只接受通过测试和审查的新解析结果；
- 格式只检查，不在本任务中批量重写。

## Verification / 验证

本章使用这些命令：

```sh
cargo run -p harness-cli --locked -- --doctor
cargo check --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
mdbook test book -L target/debug/deps
mdbook build book
typst compile typst/book.typ dist/rust-harness-book.pdf
```

它们的角色不同：

- `cargo check`：快速类型检查，不完成所有代码生成；
- `cargo test`：运行行为测试；
- `cargo clippy`：检查常见问题和项目 lint；
- `cargo fmt -- --check`：只检查格式，不修改文件；
- `mdbook test`：测试书中的 Rust block；
- `mdbook build`：构建 HTML；
- `typst compile`：当前只验证 PDF shell 可以构建。

`make verify` 把这些检查组合成一个项目入口。

## Harness status / 当前 Harness 状态

Chapter 0 的离线回合仍然通过。

Chapter 1 新增：

- 固定 Rust 1.98.0；
- Edition 2024；
- resolver 3；
- Workspace 共享 Lockfile；
- `--locked` 累计检查；
- 可执行的 `--doctor` 证据命令。

Harness 功能没有变聪明，但项目变得更可靠。

## Decision record / 决策记录

- **决定**：精确固定 Rust 1.98.0。
- **原因**：章节、错误信息和示例需要稳定基线。
- **决定**：同时声明 `rust-version = "1.98"`。
- **原因**：区分精确开发工具链和包的最低支持版本。
- **决定**：提交 Workspace Lockfile。
- **原因**：仓库包含可执行程序。
- **决定**：累计 Cargo 检查使用 `--locked`。
- **原因**：验证时不允许静默改变依赖图。
- **决定**：Doctor 只报告，不自动修复。
- **原因**：诊断和修改机器环境是不同权限。

## Stretch / 进一步练习

在不修改任何文件的前提下：

1. 运行 `rustup show active-toolchain`；
2. 运行 `cargo run -p harness-cli --locked -- --doctor`；
3. 找出两个命令各自证明了什么；
4. 写下它们没有证明什么。

然后阅读下一章的测试。我们会把本章使用的 TDD 过程单独拆开，学会如何写一个真正有用的 Red。

## Sources / 本章来源

- [Rust 1.98.0 release notes](https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/)
- [The rustup toolchain file](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file)
- [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Cargo resolver versions](https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions)
- [Cargo `--locked` and `--frozen`](https://doc.rust-lang.org/cargo/commands/cargo-build.html#manifest-options)
