# Rust 设计与实战

> 从零构建可验证的智能体 Harness
> *Building a Verifiable Agent Harness from First Principles*

这不是一本语法目录。

我们会一边学习 Rust，一边完成同一个真实项目：一个可验证的本地
Agent Harness。每一章都从问题出发，解释 Rust 为什么这样设计，再用
Red → Green → Refactor 的测试过程加入一个可运行能力。

当前已完成：

- Chapter 0：一个确定性的离线回合；
- Chapter 1：固定并检查项目工具链。

你不需要先懂完所有 Rust 才能开始。先运行完整流程，再逐层理解类型、
所有权、接口、异步、权限、持久化和验证。

每章都提供三条阅读路线：

- **构建 / Build**：让下一项行为工作；
- **理解 / Understand**：理解设计原因和代价；
- **源码阅读 / Source Read**：对照固定版本的 Codex 生产源码。

从 [先跑通一个回合](00-preface/00-one-complete-turn.md) 开始。
