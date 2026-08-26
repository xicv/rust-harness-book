#let book(
  title: none,
  subtitle: none,
  authors: none,
  language: "zh",
  region: "CN",
  body,
) = {
  set page(
    paper: "a5",
    margin: (x: 18mm, y: 20mm),
    numbering: "1",
  )
  set text(
    font: "Noto Sans CJK SC",
    size: 10.5pt,
    lang: language,
    region: region,
  )
  set par(justify: true, leading: 0.72em)
  show raw: set text(font: ("DejaVu Sans Mono", "Noto Sans CJK SC"))

  align(center)[
    #text(size: 22pt, weight: "bold")[#title]
    #v(8pt)
    #text(size: 12pt)[#subtitle]
    #if authors != none {
      v(12pt)
      text(size: 9pt)[#authors]
    }
  ]

  pagebreak()
  outline(title: [目录 / Contents], depth: 3)
  pagebreak()
  body
}

#let part(title) = {
  pagebreak(weak: true)
  heading(title, level: 1, outlined: true)
}

#let chapter(title, level: 2) = {
  pagebreak(weak: true)
  heading(title, level: level, outlined: true)
}

#let section(title, level: 3) = heading(title, level: level, outlined: true)

#let prose(body) = block(below: 0.65em, body)

#let card(body) = block(
  width: 100%,
  fill: rgb("#f5f6f8"),
  inset: 10pt,
  radius: 4pt,
  below: 0.8em,
  breakable: true,
  body,
)

#let card-label(body) = block(
  below: 4pt,
  text(size: 8pt, weight: "bold", fill: rgb("#4b5563"), body),
)

#let strong(body) = text(weight: "bold", body)
#let emph(body) = text(style: "italic", body)
#let strike-text(body) = strike(body)
#let term(body) = text(style: "italic", fill: rgb("#374151"), body)
#let inline-code(source) = raw(source)
#let book-link(target, body) = link(target, body)
#let book-ref(target, body) = link(target, body)

#let code-block(language, source) = {
  let rendered = if language == "" {
    raw(source, block: true)
  } else {
    raw(source, block: true, lang: language)
  }
  block(
    width: 100%,
    fill: rgb("#f3f4f6"),
    inset: 8pt,
    radius: 3pt,
    below: 0.8em,
    breakable: true,
    rendered,
  )
}

#let bullet-list(items) = list(..items)
#let ordered-list(start, items) = enum(start: start, ..items)

#let quote-block(body) = block(
  width: 100%,
  fill: rgb("#f9fafb"),
  inset: 8pt,
  below: 0.8em,
  breakable: true,
  body,
)

#let book-table(columns, cells) = table(
  columns: columns,
  inset: 4pt,
  ..cells,
)

#let book-image(path, alt, caption: none) = {
  let picture = image(path, width: 100%, alt: alt)
  if caption == none {
    picture
  } else {
    figure(picture, caption: caption)
  }
}

#let thematic-break() = line(length: 100%)