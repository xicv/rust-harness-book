# Typst PDF shell

This directory contains the current PDF shell. It is real and buildable, but it
is not yet a full book renderer.

The editorial source remains Markdown. Do not copy full chapters into Typst by
hand. A later, separately tested `book-render` tool may transform the canonical
chapter structure into Typst input.

Release pin: Typst 0.15.1. The exact pin lives in
`book/sources.lock.toml` and CI.
