# 章节代码包 / Chapter code packs

每个已完成章节都有一个完整、可运行的 **章节代码包** *Chapter code pack*。 它不是从当前 `main` 目录随手复制出来的快照，而是从该章通过评审时记录的 Git commit 重新导出。

代码包包含：

- 该章结束时的完整 Cargo Workspace；
- 固定 Rust 工具链和 `Cargo.lock`；
- 该章的行为测试；
- 本地验证步骤和预期输出；
- MIT 代码许可证。

下载页会同时提供 ZIP 和 SHA-256 校验值。发布流水线会在上传之前重新解压 每个包，在无网络模式下运行测试和示例，并逐字核对关键输出。

- [第 0 章代码包 / Chapter 0 code pack](ch00.md)
- [第 1 章代码包 / Chapter 1 code pack](ch01.md)

> **发布边界 / Publication boundary**
>
> 这些页面记录的是固定发布地址。只有 `main` 上的发布工作流成功、GitHub Pages 已启用，而且线上文件经过检查之后，才可以说下载地址已经可用。 本地构建或 Pull Request 通过不等于已经上线。
