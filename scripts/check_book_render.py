#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: str) -> str:
    target = ROOT / path
    if not target.is_file():
        fail(f"missing required book-render file: {path}")
    return target.read_text(encoding="utf-8")


def load_toml(path: str) -> dict:
    target = ROOT / path
    if not target.is_file():
        fail(f"missing required TOML file: {path}")
    try:
        with target.open("rb") as handle:
            value = tomllib.load(handle)
    except Exception as exc:  # noqa: BLE001 - validation boundary
        fail(f"invalid TOML in {path}: {exc}")
    if not isinstance(value, dict):
        fail(f"top-level TOML value must be a table: {path}")
    return value


def completed_chapter_count() -> int:
    path = ROOT / "scaffold.json"
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001 - validation boundary
        fail(f"invalid scaffold.json: {exc}")
    try:
        count = int(value["completed_chapter_count"])
    except (KeyError, TypeError, ValueError) as exc:
        fail(f"invalid completed_chapter_count: {exc}")
    if count < 0:
        fail("completed_chapter_count must not be negative")
    return count


def check_workspace() -> None:
    cargo = load_toml("Cargo.toml")
    members = cargo.get("workspace", {}).get("members", [])
    if "crates/book-render" not in members:
        fail("Cargo workspace does not include crates/book-render")

    package = load_toml("crates/book-render/Cargo.toml")
    if package.get("package", {}).get("name") != "book-render":
        fail("crates/book-render is not named book-render")
    dependencies = package.get("dependencies", {})
    if dependencies:
        fail("book-render must keep its deterministic standard-library-only core")

    for path in (
        "crates/book-render/src/lib.rs",
        "crates/book-render/src/main.rs",
        "crates/book-render/tests/render_contract.rs",
        "crates/book-render/tests/book_pipeline.rs",
    ):
        read(path)


def check_pipeline() -> None:
    makefile = read("Makefile")
    if not re.search(r"(?m)^book-render:\s*$", makefile):
        fail("Makefile has no book-render target")
    if not re.search(r"(?m)^pdf:\s+book-render\s*$", makefile):
        fail("PDF target does not depend on book-render")
    required_make_fragments = (
        "cargo run -p book-render --locked",
        "--book book",
        "--template typst/template.typ",
        "--output-dir dist/typst",
        "typst compile dist/typst/book.typ dist/rust-harness-book.pdf",
    )
    for fragment in required_make_fragments:
        if fragment not in makefile:
            fail(f"Makefile book-render pipeline is missing: {fragment}")

    workflow = read(".github/workflows/ci.yml")
    if "Render canonical Markdown and build full PDF" not in workflow:
        fail("CI does not name the canonical full-PDF step")
    if "run: make pdf" not in workflow:
        fail("CI does not run the canonical PDF target")

    if (ROOT / "typst/book.typ").exists():
        fail("typst/book.typ must be generated, not committed")
    template = read("typst/template.typ")
    for function in ("#let book(", "#let card(", "#let code-block("):
        if function not in template:
            fail(f"Typst template is missing {function}")


def check_contract() -> None:
    renderer = read("crates/book-render/src/lib.rs")
    required_capabilities = (
        "pub fn render_markdown",
        "pub fn render(request: &RenderRequest)",
        "SUMMARY.md",
        "rustdoc_include",
        "recursive include detected",
        "outside project root",
        "remote image must be vendored",
    )
    for capability in required_capabilities:
        if capability not in renderer:
            fail(f"renderer contract is missing evidence for: {capability}")

    test_source = read("crates/book-render/tests/book_pipeline.rs")
    required_tests = (
        "render_expands_sources_orders_summary_and_copies_assets",
        "include_cannot_escape_the_project_root",
    )
    for test in required_tests:
        if test not in test_source:
            fail(f"renderer integration test is missing: {test}")


def check_completed_chapter_receipts() -> None:
    count = completed_chapter_count()
    for number in range(count):
        chapter_id = f"ch{number:02d}"
        path = f"chapters/{chapter_id}/receipt.toml"
        receipt = load_toml(path)
        if receipt.get("book_render") != "pass":
            fail(f"{path} does not record book_render = pass")
        if receipt.get("pdf_content_parity") != "pass":
            fail(f"{path} does not record pdf_content_parity = pass")


def check_docs() -> None:
    publishing = read("docs/07-PUBLISHING-PIPELINE.md")
    required = (
        "canonical Markdown",
        "dist/typst/book.typ",
        "generated",
        "fail closed",
    )
    for phrase in required:
        if phrase not in publishing:
            fail(f"publishing guide is missing: {phrase}")

    receipt = read("docs/14-BOOK-RENDER-RECEIPT.md")
    for phrase in (
        "cb3d8edd70e976a2dac8c53617625e0c6b6d85ba",
        "SUMMARY.md",
        "Path containment",
        "PDF content parity",
    ):
        if phrase not in receipt:
            fail(f"book-render receipt is missing: {phrase}")

    readme = read("README.md")
    stale_claims = (
        "book-render does not exist",
        "PDF smoke-build shell",
        "PDF smoke pipeline",
    )
    for claim in stale_claims:
        if claim in readme:
            fail(f"README still contains a stale renderer claim: {claim}")


def main() -> None:
    check_workspace()
    check_pipeline()
    check_contract()
    check_completed_chapter_receipts()
    check_docs()
    print("book-render checks passed")


if __name__ == "__main__":
    main()
