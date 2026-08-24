# ADR 0003: mdBook for HTML, Typst for PDF

## Decision

Use Markdown as canonical editorial source, mdBook for production HTML, and
Typst for the PDF renderer.

## Why

- mdBook tests Rust snippets and includes source by anchors.
- mdBook is strong for search, navigation, and static hosting.
- Typst is strong for print-quality PDF layout.

## Cost

A small renderer is needed to avoid maintaining duplicate prose.
