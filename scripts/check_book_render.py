#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]

RENDERER_SOURCE_FILES = (
    "crates/book-render/src/lib.rs",
    "crates/book-render/src/main.rs",
    "crates/book-render/src/config.rs",
    "crates/book-render/src/includes.rs",
    "crates/book-render/src/markdown.rs",
    "crates/book-render/src/markdown_blocks.rs",
    "crates/book-render/src/markdown_inline.rs",
    "crates/book-render/src/markdown_parser.rs",
    "crates/book-render/src/paths.rs",
    "crates/book-render/src/render_gitbook.rs",
    "crates/book-render/src/render_typst.rs",
)


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


def renderer_source() -> str:
    return "\n".join(read(path) for path in RENDERER_SOURCE_FILES)


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

    for path in RENDERER_SOURCE_FILES + (
        "crates/book-render/tests/render_contract.rs",
        "crates/book-render/tests/book_pipeline.rs",
        "crates/book-render/tests/gitbook_export.rs",
    ):
        read(path)


def check_pipeline() -> None:
    makefile = read("Makefile")
    if not re.search(r"(?m)^book-render:\s*$", makefile):
        fail("Makefile has no book-render target")
    if not re.search(r"(?m)^pdf:\s+book-render\s*$", makefile):
        fail("PDF target does not depend on book-render")
    required_make_fragments = (
        "python3 scripts/check_scaffold.py",
        "python3 scripts/check_book_render.py",
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
    if (ROOT / "docs/.book-render-local-check").exists():
        fail("temporary local book-render marker must not be committed")

    template = read("typst/template.typ")
    for function in (
        "#let book(",
        "#let card(",
        "#let code-block(",
        "#let book-table(",
        "#let book-image(",
    ):
        if function not in template:
            fail(f"Typst template is missing {function}")


def check_contract() -> None:
    renderer = renderer_source()
    required_capabilities = (
        "pub fn render_markdown",
        "pub fn render(request: &RenderRequest)",
        "pub fn export_gitbook",
        "parse_summary",
        "rustdoc_include",
        "recursive include detected",
        "outside project root",
        "remote image must be vendored",
        "replace_output_directory",
    )
    for capability in required_capabilities:
        if capability not in renderer:
            fail(f"renderer contract is missing evidence for: {capability}")

    test_source = read("crates/book-render/tests/book_pipeline.rs")
    required_tests = (
        "render_expands_sources_orders_summary_and_copies_assets",
        "include_cannot_escape_the_project_root",
        "recursive_includes_fail_closed",
        "remote_images_must_be_vendored",
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
        verification = receipt.get("verification")
        if not isinstance(verification, dict):
            fail(f"{path} has no verification table")
        if verification.get("book_render") != "pass":
            fail(f"{path} does not record book_render = pass")
        if verification.get("pdf_content_parity") != "pass":
            fail(f"{path} does not record pdf_content_parity = pass")
        commands = verification.get("commands")
        if not isinstance(commands, list):
            fail(f"{path} does not contain verification commands")
        if not any("dist/typst/book.typ" in command for command in commands):
            fail(f"{path} does not verify the generated Typst source")


def check_docs() -> None:
    publishing = read("docs/07-PUBLISHING-PIPELINE.md")
    publishing_folded = publishing.casefold()
    required = (
        "canonical markdown",
        "dist/typst/book.typ",
        "generated",
        "fail closed",
        "summary.md",
        "local assets",
    )
    for phrase in required:
        if phrase.casefold() not in publishing_folded:
            fail(f"publishing guide is missing: {phrase}")

    receipt = read("docs/14-BOOK-RENDER-RECEIPT.md")
    receipt_folded = receipt.casefold()
    for phrase in (
        "cb3d8edd70e976a2dac8c53617625e0c6b6d85ba",
        "2e31c4f3ae3d4fcef7f515b8492e952a9f908f2f",
        "summary.md",
        "path containment",
        "pdf content parity",
    ):
        if phrase.casefold() not in receipt_folded:
            fail(f"book-render receipt is missing: {phrase}")

    stale_claims = (
        "book-render does not exist",
        "book-render` does not exist",
        "PDF smoke-build shell",
        "PDF smoke shell",
        "PDF smoke pipeline",
        "pending-book-render",
    )
    for path in (
        "README.md",
        "docs/00-RESEARCH-STATUS.md",
        "docs/05-CHAPTER-CONTRACT.md",
        "docs/07-PUBLISHING-PIPELINE.md",
        "scaffold.json",
    ):
        text = read(path)
        for claim in stale_claims:
            if claim in text:
                fail(f"{path} still contains a stale renderer claim: {claim}")


def main() -> None:
    check_workspace()
    check_pipeline()
    check_contract()
    check_completed_chapter_receipts()
    check_docs()
    print("book-render checks passed")


if __name__ == "__main__":
    main()
