#let book(
  title: none,
  subtitle: none,
  body,
) = {
  set page(
    paper: "a5",
    margin: (x: 18mm, y: 20mm),
  )
  set text(size: 10.5pt)
  set par(justify: true, leading: 0.72em)

  align(center)[
    #text(size: 22pt, weight: "bold")[#title]
    #v(8pt)
    #text(size: 12pt)[#subtitle]
  ]

  pagebreak()
  body
}
