<!--
chapter_id: chXX
chapter_status: planned
chapter_goal: One clear reader and harness outcome.
rust: 1.98.0
codex_commit: 2df67054232090af8d2fa197c46b994bc2b0dda1
new_terms: []
examples: []
-->

# 中文标题 / English Title

> **Status / 状态:** Planned
> **Harness milestone / Harness 里程碑:** One clear reader and harness outcome.

## 先看全局 / Big picture

这一章在全书和产品路线中的位置。

## 概念先行 / Concept first

先用普通语言解释概念，再引入 Rust 术语和语法。

<div class="learning-card">
<p class="card-label">现在只要知道 / For now</p>

- 当前必须理解的内容
- 可以留到后面再深入的内容

</div>

## 本章要完成什么 / Product goal

- Harness 新增的能力
- 公开行为
- 明确不做的范围

## 先看运行结果 / See it run

```sh
# One real command
```

解释输出表达了什么。

## 拆开代码 / Read the code

### 第一层

- 输入
- 输出
- 设计原因
- 官方资料

### 第二层

继续按职责拆开，不逐行翻译。

## 这段代码怎样长出来 / TDD story

### Behaviour / 行为契约

### Red / 先看见正确的失败

### Green / 最小正确实现

### Refactor / 在绿色安全网下整理

### Edge cases / 边界情况

## 为什么 Rust 这样设计 / Why Rust

- Rust 的选择
- 得到什么
- 付出什么

## 公平对比 / Fair comparison

比较同一个问题，不做语言战争。

## 自己动手 / Try it

### 浏览器即时实验 / Inline Playground

如果概念可以诚实地缩成一个独立文件，加入：

````markdown
```rust,editable,edition2024
{{#rustdoc_include path/to/lab.rs:anchor}}
```
````

说明要改哪一行、观察什么，并写清 Playground 的版本和隐私边界。

如果真实行为依赖 Workspace、本地 crate、文件、进程或固定依赖，则不要伪造
Playground 版本。改用 Project lab，并说明原因。

### 完整工程实验 / Project lab

- 聚焦命令
- 小实验
- 可选深入练习

## Codex 源码阅读 / Source reading

- Source fact
- Pinned path
- Our decision
- Difference and limit

## AI 候选方案审核 / Review an AI candidate

用测试和工程原则审查一个看似合理但有缺陷的方案。

## 验证与边界 / Verify and limits

- 聚焦测试
- 累计测试
- 当前能做什么
- 当前不能做什么

## 本章小结与下一步 / Recap and next step

用自己的话总结概念、产品增量和下一章连接。
