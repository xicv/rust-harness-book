# Terminology and style / 术语与文风

## Tone

- Smart casual
- Friendly, not childish
- Short paragraphs
- Concise dot points
- Easy, modern English
- Concrete verbs
- No unnecessary jargon
- No language-war tone

## Bilingual terms

First important use:

> **所有权** *Ownership*

Later uses may use the Chinese term alone unless the English form helps the
reader follow code, diagnostics, or documentation.

## Important distinctions

- Codex **会话** *Thread* is a conversation container, not an operating-system
  thread.
- An operating-system thread is **操作系统线程** *OS Thread*.
- Keep the word **Trait** visible. “特征” may help the first explanation, but
  Rust code and compiler messages use `trait`.
- Use **智能体运行框架** *Agent Harness* for the project concept. The shorter
  word “Harness” may remain in the title and code-facing names.

## Typography roles

- Chinese body: normal reading face
- English technical term: contrasting sans-serif style
- Rust identifier, command, path: monospace
- Warning and status: icon or text label as well as colour

## Comparison style

- Compare one problem at a time.
- Use the modern, strongest form of the other language.
- State where Rust is better.
- State where Rust costs more.
- State when the other language is the better choice.

## Explanation formula

```text
Problem → Other approaches → Rust choice → Why → Gain → Cost → Harness use → Test
```
