# Contributing / 参与方式

## One coherent change at a time

A chapter change should be small enough to review and large enough to add one
real behaviour.

## Required evidence

- Behaviour or invariant added
- Red test and expected failure
- Green implementation
- Earlier tests still passing
- Source note and Codex comparison where relevant
- New bilingual terms registered
- No unrelated formatting changes
- Chapter evidence receipt updated

## Source hierarchy

1. Current official Rust, Cargo, mdBook, Typst, and OpenAI documentation
2. Pinned public source code
3. Reference books for teaching-method analysis
4. Secondary sources only when primary sources do not answer the question

## Review questions

- Is the design easy to explain?
- Is the API hard to misuse?
- Is the test checking public behaviour?
- Is the example useful to the growing harness?
- Is the comparison with another language fair and current?
- Are costs, limits, and failure modes clear?
