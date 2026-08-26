#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: str) -> str:
    target = ROOT / path
    if not target.is_file():
        fail(f"missing interactive-lab file: {path}")
    return target.read_text(encoding="utf-8")


def load_toml(path: str) -> dict:
    target = ROOT / path
    if not target.is_file():
        fail(f"missing interactive-lab TOML file: {path}")
    try:
        with target.open("rb") as handle:
            value = tomllib.load(handle)
    except Exception as exc:  # noqa: BLE001 - validation boundary
        fail(f"invalid TOML in {path}: {exc}")
    if not isinstance(value, dict):
        fail(f"top-level TOML value must be a table: {path}")
    return value


def check_playground_config() -> None:
    config = load_toml("book/book.toml")
    try:
        playground = config["output"]["html"]["playground"]
    except (KeyError, TypeError) as exc:
        fail(f"book playground configuration is missing: {exc}")

    required = {
        "editable": True,
        "copyable": True,
        "copy-js": True,
        "line-numbers": True,
        "runnable": True,
    }
    for key, expected in required.items():
        if playground.get(key) is not expected:
            fail(f"book playground setting {key!r} must be {expected!r}")


def check_first_lab() -> None:
    index = read("book/src/index.md")
    required_index_fragments = (
        "## 随手运行 Rust / Run it as you read",
        "rust,editable,edition2024",
        "{{#rustdoc_include _labs/00-first-playground.rs:first_inline_playground}}",
        "不要把密码、API key、私人代码或公司代码贴进在线 Playground",
        "Rust `1.98.0`",
    )
    for fragment in required_index_fragments:
        if fragment not in index:
            fail(f"introduction is missing interactive-lab contract: {fragment}")

    source = read("book/src/_labs/00-first-playground.rs")
    for fragment in (
        "ANCHOR: first_inline_playground",
        "ANCHOR_END: first_inline_playground",
        "fn main()",
        "assert_eq!(events.len(), 5);",
    ):
        if fragment not in source:
            fail(f"first interactive lab is missing: {fragment}")

    examples = load_toml("book/examples.toml").get("example", [])
    matches = [
        value
        for value in examples
        if isinstance(value, dict) and value.get("id") == "ch00-inline-playground"
    ]
    if len(matches) != 1:
        fail("book/examples.toml must register ch00-inline-playground exactly once")
    entry = matches[0]
    if entry.get("kind") != "playground":
        fail("ch00-inline-playground must use kind = playground")
    if entry.get("source") != "book/src/_labs/00-first-playground.rs":
        fail("ch00-inline-playground source does not match the checked lab")
    if entry.get("anchor") != "first_inline_playground":
        fail("ch00-inline-playground anchor does not match the checked lab")
    if entry.get("offline") is not True:
        fail("ch00-inline-playground verification must remain offline")


def check_policy() -> None:
    policy = read("docs/15-INTERACTIVE-LABS.md")
    required = (
        "Level 1 — Inline Playground",
        "Level 2 — Project lab",
        "Level 3 — Notebook lab",
        "Rust 1.98.0",
        "public Rust Playground",
        "must never paste",
        "mdbook test",
        "Evcxr",
    )
    for fragment in required:
        if fragment not in policy:
            fail(f"interactive-lab policy is missing: {fragment}")

    agents = read("AGENTS.md")
    for fragment in (
        "editable browser Playground",
        "Use a project lab instead",
        "not release evidence",
    ):
        if fragment not in agents:
            fail(f"AGENTS.md is missing interactive-lab guidance: {fragment}")

    template = read("book/src/_templates/chapter.md")
    for fragment in (
        "### 浏览器即时实验 / Inline Playground",
        "rust,editable,edition2024",
        "### 完整工程实验 / Project lab",
    ):
        if fragment not in template:
            fail(f"chapter template is missing interactive-lab guidance: {fragment}")


def main() -> None:
    check_playground_config()
    check_first_lab()
    check_policy()
    print("interactive lab checks passed")


if __name__ == "__main__":
    main()
