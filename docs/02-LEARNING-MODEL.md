# Learning model / 学习模型

## Main loop

```text
Problem → Predict → Red → Explain → Green → Refactor → Verify → Compare → Record
```

## Three paths through each chapter

- **Build / 构建:** make the next behaviour work
- **Understand / 理解:** learn the Rust idea and its trade-offs
- **Source read / 源码阅读:** compare with pinned production code

## Nine questions for every major idea

- What problem are we solving?
- How do other languages solve it?
- What did Rust choose?
- Why did Rust choose it?
- What do we gain?
- What does it cost?
- Which engineering principle does it teach?
- What does it add to the harness?
- Which test proves it?

## AI-native engineering loop

- Ask AI for options, not a final truth.
- List the assumptions.
- Check official documentation.
- Read the relevant source and tests.
- Predict the failure.
- Run the test.
- Review the diff.
- Check failure, cancellation, and security paths.
- Record the decision and remaining uncertainty.

## Difficulty policy

- Preview advanced syntax only when it helps the reader run a useful whole.
- Label previews clearly.
- Return later for the full explanation.
- Prefer one new hard idea per chapter.
- Keep the default path offline and deterministic.
