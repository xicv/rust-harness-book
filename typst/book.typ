#import "template.typ": book

#show: book.with(
  title: [Rust 设计与实战],
  subtitle: [从零构建可验证的智能体 Harness],
)

= PDF pipeline smoke build

The canonical book source lives in `book/src/`.

Chapters 0 and 1 are complete in the HTML source. Full PDF content parity remains
pending until the planned `book-render` tool converts the canonical Markdown to
Typst.
