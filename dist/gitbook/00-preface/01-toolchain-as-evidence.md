# 工具链就是证据 / The Toolchain Is Evidence

> **Status / 状态:** Complete **Harness milestone / Harness 里程碑:** Pin and verify the Rust workspace.

## 先看全局 / Big picture

上一章的 Harness 已经在我们的机器上运行了。

这是一件好事，但还不够。

当我们说“代码可以运行”，其实省略了很多条件：

- 使用哪个 Rust 编译器；
- 按哪一代 Rust 规则解释源码；
- Cargo 怎样选择依赖；
- 实际锁定了哪些版本；
- Clippy 和 rustfmt 是否可用；
- 命令从哪个目录执行；
- 本地和 CI 是否使用同一组基线。

如果这些条件只存在于某个人的电脑和记忆里，我们很难让另一个读者重复同样结果，也很难在几个月后调查“为什么以前能过，现在不能”。

所以本章先不增加新的 Agent 行为。

我们要做一件更基础的工程工作：把项目的构建环境写进仓库，并让程序可以检查它。

> **本章核心 / Core idea**
>
> “在我的机器上可以运行”是一条观察。
>
> “在已记录的工具链和依赖条件下，可以重复运行，并且检查全部通过”才更接近工程证据。

## 概念先行 / Concept first

“Rust 版本”这句话经常混合了几种不同概念。

这一章先把它们分开。

### Toolchain / 工具链

**工具链** *Toolchain* 不是只有 `rustc`。

一个普通 Rust 项目通常会使用：

- `rustc`：把 Rust 源码编译成目标程序；
- `cargo`：管理项目、构建、测试和依赖；
- 标准库；
- `rustfmt`：检查或统一格式；
- `clippy`：发现常见问题和不理想写法；
- 某个目标平台需要的标准库组件。

`rustup` 负责安装和选择这些工具链。

可以把它理解成“这个项目实际使用哪一套 Rust 开发工具”。

### Rust Edition / Rust 版本代际

**Rust 版本代际** *Rust Edition* 决定一份源码使用哪一代语言规则和惯用法。

本项目使用：

```text
Edition 2024
```

Edition 不是 `rustc 1.98.0` 的另一种写法。

同一个现代编译器可以编译使用不同 Edition 的 crate。Edition 让 Rust 在继续演进时，保留旧代码的兼容路径。

Rust 官方 Book 当前也使用 Edition 2024，并假设 Rust 1.90 或更新版本。

### MSRV / 最低支持版本

**最低支持 Rust 版本** *Minimum Supported Rust Version (MSRV)* 表示这个项目承诺至少支持多新的 Rust。

本项目写：

```text
rust-version = "1.98"
```

它是一条兼容性声明，不是精确工具链 pin。

### Exact toolchain / 精确工具链

仓库中的 `rust-toolchain.toml` 写：

```text
channel = "1.98.0"
```

这告诉 rustup：进入这个目录运行 `cargo` 或 `rustc` 时，优先选择精确的 `1.98.0` 工具链。

简单记忆：

```text
Edition        → 这份源码按哪一代规则理解
MSRV           → 最低支持到哪一版 Rust
Toolchain pin  → 这个仓库现在用哪一个精确工具集验证
```

### Workspace / 工作空间

一个 Rust 项目可以包含多个 package 和 crate。

**工作空间** *Workspace* 把相关 package 放在一个共同根目录管理。它们可以共享：

- `Cargo.lock`；
- `target` 构建目录；
- Edition 与 MSRV 设置；
- lint 规则；
- 从根目录执行的累计测试。

我们的 Workspace 目前包含：

```text
harness-core
harness-cli
harness-test-support
book-render
```

它们不是为了“看起来像大项目”才拆开。

- `harness-core` 保存领域行为；
- `harness-cli` 负责进程和终端；
- `harness-test-support` 为后续共享测试做准备；
- `book-render` 把 canonical Markdown 生成 Typst。

这些边界会随着真实需要继续调整。

### Crate / 编译单元

**Crate** *Crate* 是 Rust 编译器一次处理的编译单元。

一个 package 可以包含 library crate 和 binary crate。当前 `harness-cli` 是可执行程序，`harness-core` 是可以被其他 crate 使用的库。

现在不用记住 Cargo 所有术语。先知道：Workspace 管理一组相关 package，而 crate 是 Rust 编译的基本单元。

### Resolver / 依赖解析器

Cargo 需要根据各 package 的依赖要求，选出实际使用的版本和 feature 组合。

本项目显式使用 **依赖解析器 3** *Dependency Resolver 3*：

```text
resolver = "3"
```

Resolver 3 会在选择依赖时考虑依赖声明的 Rust 版本要求。

它不是安全扫描器，也不会判断一个 crate 的设计质量。它只负责依赖解析规则。

### Cargo.toml 和 Cargo.lock

这两个文件经常放在一起，但作用不同。

`Cargo.toml` 是我们写的项目描述：

- Workspace 成员；
- package 信息；
- 允许的依赖范围；
- Edition；
- MSRV；
- lint 配置。

**锁文件** *Lockfile* `Cargo.lock` 记录某一次实际解析得到的精确依赖结果。

可以这样理解：

```text
Cargo.toml → 我允许什么
Cargo.lock → 这次实际选了什么
```

Cargo 官方文档建议通常把 lockfile 放进版本控制，尤其是可执行项目。它让依赖变化变成可以审查的 diff。

Lockfile 不证明依赖安全。它只固定了我们正在使用的版本和来源。供应链审计会在后面的加固章节处理。

## 本章要完成什么 / Product goal

我们要让仓库清楚记录：

- Rust `1.98.0`；
- Edition `2024`；
- MSRV `1.98`；
- resolver `3`；
- Clippy 和 rustfmt 组件；
- 当前精确依赖解析。

同时让 CLI 提供一条只读检查命令：

```sh
cargo run -p harness-cli --locked -- --doctor
```

它要：

1. 找到 Workspace 根目录；
2. 读取版本控制中的 toolchain 和 Cargo 配置；
3. 运行当前 `rustc --version`；
4. 运行当前 `cargo --version`；
5. 检查 `Cargo.lock` 是否存在；
6. 清楚报告匹配或不匹配；
7. 不自动安装、升级或修改任何东西。

这条命令不是整台电脑的健康检查。它只验证当前最小构建契约。

## 先看运行结果 / See it run

运行：

```sh
cargo run -p harness-cli --locked -- --doctor
```

在符合本项目 pin 的环境中，会看到：

```text
toolchain/status ok
rust/pinned 1.98.0
rustc/active 1.98.0
cargo/active 1.98.0
edition 2024
resolver 3
lockfile present
```

这几行分别回答：

- 仓库要求什么；
- 当前实际使用什么；
- 源码按哪代规则理解；
- Cargo 使用哪套依赖解析规则；
- lockfile 是否存在。

如果版本不同，命令不应该偷偷修改系统。它报告 `mismatch`，让人决定下一步。

### 再用 rustup 查看选择过程

可以运行：

```sh
rustup show
```

rustup 会显示当前目录最终选择了哪个工具链。

rustup 的选择有优先级。例如命令行 `+toolchain`、环境变量、目录 override、`rust-toolchain.toml` 和默认工具链都可能影响结果。官方 rustup book 对这套覆盖规则有完整说明：

- [rustup book：Overrides](https://rust-lang.github.io/rustup/overrides.html)

本章的 `--doctor` 只比较最终实际版本与仓库 pin，不重写 rustup 的选择逻辑。

## 先看三个配置文件 / Read the project files

### rust-toolchain.toml：这个仓库实际用什么

```toml
[toolchain]
channel = "1.98.0"
profile = "minimal"
components = ["clippy", "rustfmt"]

```

我们关心三项：

- `channel = "1.98.0"`：精确工具链；
- `profile = "minimal"`：先安装最小组件集合；
- `components = ["clippy", "rustfmt"]`：额外加入检查工具。

这不是说其他工具没有价值。它只是当前 CI 和本地验证所需的最小明确集合。

### Cargo.toml：Workspace 共同契约

根 `Cargo.toml` 中最重要的部分可以先这样读：

```text
[workspace]         → 管理哪些成员
resolver = "3"      → 怎样解析依赖

[workspace.package]
edition = "2024"    → 源码规则
rust-version = "1.98" → 最低 Rust 版本
```

Rust Book 和 Cargo Book 可以继续查这些概念：

- [Rust Book：Packages and Crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)
- [Cargo Book：Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Rust Book：Editions](https://doc.rust-lang.org/book/appendix-05-editions.html)

### Cargo.lock：实际依赖结果

不要手工编辑 `Cargo.lock`。

Cargo 在解析依赖时生成和维护它。使用 `--locked` 运行命令时，如果 Cargo 需要改写 lockfile，命令会失败，而不是静默改变依赖结果。

继续阅读：

- [Cargo Book：Cargo.toml vs Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)

这给我们一个很实用的审查边界：功能改动不应顺便带入没有解释的依赖漂移。

## 拆开 `--doctor` 代码 / Read the code

本章的实现集中在 `ToolchainReport`。

```rust
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ToolchainReport {
    pinned_rust: String,
    active_rustc: String,
    active_cargo: String,
    edition: String,
    resolver: String,
    lockfile_present: bool,
}

impl ToolchainReport {
    pub(crate) fn collect() -> Result<Self, DoctorError> {
        let root = workspace_root()?;
        Self::collect_from(&root)
    }

    fn collect_from(root: &Path) -> Result<Self, DoctorError> {
        let toolchain = read_source(root.join("rust-toolchain.toml"))?;
        let manifest = read_source(root.join("Cargo.toml"))?;

        Ok(Self {
            pinned_rust: quoted_value(&toolchain, "channel")?.to_owned(),
            active_rustc: command_version("rustc")?,
            active_cargo: command_version("cargo")?,
            edition: quoted_value(&manifest, "edition")?.to_owned(),
            resolver: quoted_value(&manifest, "resolver")?.to_owned(),
            lockfile_present: root.join("Cargo.lock").is_file(),
        })
    }

    #[must_use]
    pub(crate) fn is_match(&self) -> bool {
        self.active_rustc == self.pinned_rust
            && self.active_cargo == self.pinned_rust
            && self.lockfile_present
    }
}

impl fmt::Display for ToolchainReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_match() { "ok" } else { "mismatch" };
        let lockfile = if self.lockfile_present {
            "present"
        } else {
            "missing"
        };
        let pinned_rust = &self.pinned_rust;
        let active_rustc = &self.active_rustc;
        let active_cargo = &self.active_cargo;
        let edition = &self.edition;
        let resolver = &self.resolver;

        writeln!(formatter, "toolchain/status {status}")?;
        writeln!(formatter, "rust/pinned {pinned_rust}")?;
        writeln!(formatter, "rustc/active {active_rustc}")?;
        writeln!(formatter, "cargo/active {active_cargo}")?;
        writeln!(formatter, "edition {edition}")?;
        writeln!(formatter, "resolver {resolver}")?;
        writeln!(formatter, "lockfile {lockfile}")
    }
}
```

先不用看每个 helper。主要流程是：

```text
找到仓库根目录
→ 读取 rust-toolchain.toml
→ 读取 Cargo.toml
→ 执行 rustc --version
→ 执行 cargo --version
→ 检查 Cargo.lock
→ 形成一份 ToolchainReport
```

### 为什么返回 Report，不直接边检查边打印

`ToolchainReport` 先保存结构化结果，再由 `Display` 决定怎样输出。

这样有几个好处：

- “收集事实”和“展示事实”分开；
- 测试可以比较报告行为；
- 以后 GUI 或 JSON 输出可以复用收集逻辑；
- 退出码可以根据 `is_match()` 决定。

它沿用上一章的原则：核心判断先返回数据，进程边界再处理 I/O。

### 为什么现在手写少量 TOML 读取

当前程序只读取几个简单、由本仓库控制的键，所以实现使用很小的解析逻辑，没有马上加入 TOML 依赖。

这是一项有边界的决定，不是一条普遍建议。

如果未来要处理：

- 嵌套表；
- 数组；
- 多种等价 TOML 写法；
- 用户输入的任意 manifest；
- 完整 Cargo 语义；

就应该使用正式 TOML 解析器，而不是继续扩大手写代码。

### 失败也有不同类型

`DoctorError` 区分：

- 找不到当前目录；
- 找不到 Workspace；
- 文件读取失败；
- 配置值缺失；
- 无法启动 `rustc` 或 `cargo`；
- 命令返回失败；
- 版本输出格式异常。

我们暂时不深入每个错误变体。第 7 章会专门讨论怎样设计 `Result` 和错误类型。

现在只需要看到：失败不是一条模糊字符串，它有来源和类别。

## 这段代码怎样长出来 / TDD story

这章的 TDD 起点是一条真实产品行为：

> 当读者运行 `--doctor` 时，CLI 应报告仓库 pin、当前工具链、Edition、resolver 和 lockfile 状态。

### Red：旧 CLI 不认识 `--doctor`

测试启动真实 `harness-cli`：

```rust
#[test]
fn doctor_reports_pinned_and_active_toolchain() -> Result<(), Box<dyn Error>> {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_harness-cli"))
        .arg("--doctor")
        .current_dir(workspace_root)
        .output()?;

    assert!(
        output.status.success(),
        "doctor command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout)?,
        concat!(
            "toolchain/status ok\n",
            "rust/pinned 1.98.0\n",
            "rustc/active 1.98.0\n",
            "cargo/active 1.98.0\n",
            "edition 2024\n",
            "resolver 3\n",
            "lockfile present\n",
        )
    );

    Ok(())
}
```

在实现前，CLI 把 `--doctor` 当作普通提示词，于是输出上一章的五个 Harness 事件。

测试期待工具链报告，却得到：

```text
thread/started ...
turn/started ...
```

这个 Red 很有信息量：

- Chapter 0 行为仍然正常；
- 新命令还没有路由；
- 测试观察的是真实二进制输出。

### Green：加入最小只读检查

最小实现只做：

- 识别 `--doctor`；
- 读取三个仓库文件；
- 执行两个版本命令；
- 输出报告；
- 使用有意义的退出码。

没有加入：

- 自动安装器；
- 通用系统诊断框架；
- 网络访问；
- Homebrew 检查；
- IDE 配置；
- 跨平台 SDK 管理。

### Refactor：把环境检查留在 CLI

工具链检查属于应用和开发环境边界，不属于 Agent 领域核心。

因此它留在 `harness-cli`，没有塞进 `harness-core`。

这是一项架构判断：不是所有“项目需要的功能”都应该进入核心库。

### 测试支持了什么

这条黑盒测试支持：

- `--doctor` 被正确路由；
- 当前固定环境输出精确报告；
- 命令成功退出；
- 关键配置没有与代码报告漂移。

它没有证明：

- 每个错误分支都在每个平台运行过；
- 所有系统工具都可用；
- 依赖没有漏洞；
- macOS、Linux、Windows 完全相同；
- 未来 Rust 版本一定兼容。

测试是证据的一部分，不是所有风险的终点。

## 为什么 Rust 这样设计 / Why Rust

这一章表面在讲工具，背后是几个更大的工程原则。

### 把环境写进版本控制

如果重要版本只存在于 README 之外的口头约定里，它很容易漂移。

`rust-toolchain.toml`、`Cargo.toml` 和 `Cargo.lock` 把不同层次的决定写进仓库。

收益：

- 新读者少猜；
- 本地与 CI 更接近；
- 升级成为明确 diff；
- 旧问题更容易重现；
- AI 不容易偷偷把“latest”当成一个稳定值。

成本：

- 多个字段需要保持一致；
- pin 需要定期升级；
- 固定太久会错过安全修复；
- 平台差异仍需要真实测试。

所以正确策略不是“永远固定”或“永远最新”。

而是：

```text
固定当前证据
→ 定期研究新版本
→ 独立升级
→ 重新运行全部验证
```

### Edition 把演进和兼容分开

Rust 希望语言继续改进，又不希望每次更新编译器都破坏旧项目。

Edition 提供一个明确的源码兼容边界。编译器版本与 Edition 分开，让同一工具链能够支持不同年代的代码。

这个设计比只有一个“语言版本”概念复杂一些，但它给长期项目更平滑的升级路线。

### Workspace 把共同规则放在正确层级

多个 crate 如果分别复制 Edition、MSRV 和 lint 设置，很容易不一致。

Workspace 让共同规则集中管理，同时保留 crate 边界。

但 Workspace 不是越大越专业。过早拆几十个 crate 会增加编译、导航和依赖管理成本。我们只在出现真实职责边界时拆分。

### Lockfile 让依赖变化可见

没有 lockfile 时，即使源码没变，重新解析依赖也可能选择不同版本。

Lockfile 让一次依赖图变成可审查的事实。

但它不是安全证明。一个被锁定的恶意依赖仍然是恶意依赖。后面还需要来源、许可证和漏洞审查。

## 公平对比 / Fair comparison

其他生态也有对应方案。

### Go

Go 的工具链和模块体验通常更统一，选择更少，上手更直接。对许多服务项目，这是明显优势。

Rust 给维护者更细的控制，例如多个 toolchain、Edition、组件和 target。控制更多，也意味着决定更多。

### Python

Python 小脚本可以非常轻，不需要先建立 Workspace。

但大型 Python 项目同样需要虚拟环境、项目元数据和 lock 工具。选择通常来自多个工具，团队需要自己组合。

### Node.js

Node.js 生态很熟悉 `package.json`、lockfile 和运行时版本管理。

前端工具链整合常常比 Rust 更成熟，但不同 package manager 和运行时组合也可能带来更多变体。

Rust 的特点不是“只有它能复现”，而是 rustup、Cargo、编译器、测试和文档形成了一套相对一致的工具链。

## 自己动手 / Try it

### 练习一：查看当前工具链

```sh
rustup show
rustc --version
cargo --version
```

比较它们与：

```sh
cargo run -p harness-cli --locked -- --doctor
```

报告中的版本是否一致。

### 练习二：理解 `--locked`

先确认工作树干净，然后运行：

```sh
cargo check --workspace --locked
```

`--locked` 不是“更快”的参数。它表示如果这次构建需要修改 `Cargo.lock`，Cargo 应该失败。

### 练习三：只读查看三个配置

打开：

```text
rust-toolchain.toml
Cargo.toml
Cargo.lock
```

用自己的话回答：

- 哪个文件由人主要维护？
- 哪个文件由 Cargo 维护？
- 哪个字段是精确工具链？
- 哪个字段是最低版本？
- 哪个字段是源码 Edition？

### 练习四：观察一个可控失败

不要提交这个修改。

临时把 `rust-toolchain.toml` 中的 channel 改成另一个已安装版本，再运行 `--doctor`，观察报告和退出状态。完成后恢复文件，并再次运行：

```sh
cargo run -p harness-cli --locked -- --doctor
```

如果本地没有另一个工具链，可以只阅读测试和代码，不需要为了实验额外下载。

## Codex 源码阅读 / Source reading

**Source fact / 来源事实：** 固定版本的 Codex Rust 代码使用一个大型 Workspace 管理协议、核心、工具、沙箱、客户端和测试等多个 crate。

固定来源：

- `openai/codex@2df67054232090af8d2fa197c46b994bc2b0dda1`
- [`codex-rs/Cargo.toml`](https://github.com/openai/codex/blob/2df67054232090af8d2fa197c46b994bc2b0dda1/codex-rs/Cargo.toml)
- [`AGENTS.md`](https://github.com/openai/codex/blob/2df67054232090af8d2fa197c46b994bc2b0dda1/AGENTS.md)

Codex 的 crate 数量反映了它已经承担的真实职责。我们不应该从结果倒推：“专业 Rust 项目一开始就要拆成这么多 crate。”

**Our decision / 本书决定：** 从少量清楚边界开始。只有当一项能力拥有独立职责、依赖方向或验证需要时，再拆出新 crate。

这也是我们阅读大型源码时要保持的判断：学习它为什么演进成现在的结构，而不是只复制目录形状。

## AI 候选方案审核 / Review an AI candidate

AI 可能给出以下“保持最新”方案：

```text
永远使用 latest stable
删除 Cargo.lock
CI 每次先 cargo update
本地失败就换 nightly
运行 cargo fmt 自动修改全仓
```

逐项检查。

### “latest” 是稳定值吗

不是。时间一变，它代表的版本也变。

### 自动更新依赖会发生什么

功能 diff 与依赖 diff 混在一起。失败后更难判断原因。

### 删掉 lockfile 是否更干净

对于可执行 Workspace，它会让精确依赖解析漂移，而不是让项目更简单。

### 随意换 nightly 是否在解决问题

通常不是。它改变了环境，可能掩盖原来的兼容问题。

### 自动格式化全仓有什么代价

它可能制造大量无关 diff，让真正的行为变化更难审查。

更稳妥的选择：

- 固定当前工具链和 lockfile；
- 使用 check-only 格式命令；
- 把升级放进独立、可审查的变更；
- 阅读 release notes；
- 让聚焦测试和累计测试都通过；
- 记录平台差异。

AI 的建议不一定恶意。问题在于它把“方便”当成唯一目标，没有说明可复现性和审查成本。

## 验证与边界 / Verify and limits

本章聚焦命令：

```sh
cargo test -p harness-cli --test doctor
cargo run -p harness-cli --locked -- --doctor
```

Workspace 检查：

```sh
cargo check --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

Book 检查：

```sh
mdbook test book -L target/debug/deps
mdbook build book
make pdf
```

> **当前证据 / Current evidence**
>
> 仓库现在明确记录：
>
> - Rust `1.98.0`
> - Edition `2024`
> - MSRV `1.98`
> - resolver `3`
> - Clippy 与 rustfmt
> - 具体 `Cargo.lock`
> - 本地与 CI 的检查命令
>
> `--doctor` 能验证当前最小工具链契约。

### 当前还没有验证什么

- 所有目标平台；
- Xcode 或系统 linker；
- rust-analyzer；
- mdBook 和 Typst 的本地版本；
- CPU 架构差异；
- 依赖许可证和漏洞；
- 依赖源码是否值得信任；
- 环境变量对所有 crate 的影响。

后面会逐步扩大证据范围，但每次都要有真实需要。

## 本章决定 / Decision record

- Rust `1.98.0` 是当前精确验证工具链。
- Edition 2024 与 MSRV 1.98 分开记录。
- 虚拟 Workspace 显式设置 resolver 3。
- 可执行 Workspace 提交 `Cargo.lock`。
- `--doctor` 只读，不自动修改环境。
- 工具链检查留在 CLI，不进入领域核心。
- 当前简单 TOML 读取有明确范围；配置复杂后换正式解析器。
- 升级必须成为独立、可审查、重新验证的变化。

## 本章小结与下一步 / Recap and next step

这一章没有让 Agent 变得更聪明。

它让我们对“当前代码在什么条件下成立”说得更清楚。

现在可以区分：

```text
Edition        → 源码规则
MSRV           → 最低支持版本
Toolchain pin  → 当前精确工具
Workspace      → 多个 package 的共同管理层
Resolver       → 依赖选择规则
Cargo.lock     → 这次实际解析结果
```

这是一项很重要的工程习惯：先把环境和假设写下来，再讨论结果。

下一章正式把 TDD 拿出来仔细看。

不是重复“先写测试”这句口号，而是研究：

- 怎样写一个真正有意义的 Red；
- 为什么测试通过仍可能很弱；
- 怎样让测试观察公开行为；
- 怎样在不复制生产逻辑的情况下建立可复用契约；
- 测试怎样帮助我们约束 AI 生成的改动。

想单独保存或修改本章结束时的完整项目，可以使用 [第 1 章代码包 / Chapter 1 code pack](../_delivery/ch01.md)。下载页记录固定源 commit、校验值和从干净目录开始的验证步骤。
