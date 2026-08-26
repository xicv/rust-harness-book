#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path
import re
import sys
import tomllib
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
EXCLUDED_PARTS = {".git", "target", "dist", "__pycache__"}
TEXT_SUFFIXES = {
    ".css",
    ".json",
    ".md",
    ".py",
    ".rs",
    ".toml",
    ".typ",
    ".yaml",
    ".yml",
}


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def relative(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def is_excluded(path: Path) -> bool:
    try:
        parts = path.relative_to(ROOT).parts
    except ValueError:
        return True
    return any(part in EXCLUDED_PARTS for part in parts)


def load_toml(path: Path) -> dict:
    try:
        with path.open("rb") as handle:
            return tomllib.load(handle)
    except Exception as exc:  # noqa: BLE001 - validation boundary
        fail(f"invalid TOML in {relative(path)}: {exc}")


def load_json(path: Path) -> dict:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001 - validation boundary
        fail(f"invalid JSON in {relative(path)}: {exc}")
    if not isinstance(value, dict):
        fail(f"top-level JSON value must be an object: {relative(path)}")
    return value


def parse_data_files() -> None:
    for path in sorted(ROOT.rglob("*.toml")):
        if not is_excluded(path):
            load_toml(path)
    for path in sorted(ROOT.rglob("*.json")):
        if not is_excluded(path):
            load_json(path)


def strip_fenced_code(text: str) -> str:
    lines: list[str] = []
    fence: str | None = None
    for line in text.splitlines():
        stripped = line.lstrip()
        marker = None
        if stripped.startswith("```"):
            marker = "```"
        elif stripped.startswith("~~~"):
            marker = "~~~"
        if marker is not None:
            if fence is None:
                fence = marker
            elif fence == marker:
                fence = None
            continue
        if fence is None:
            lines.append(line)
    return "\n".join(lines)


def check_markdown_links() -> None:
    pattern = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
    for path in sorted(ROOT.rglob("*.md")):
        if is_excluded(path):
            continue
        text = strip_fenced_code(path.read_text(encoding="utf-8"))
        for raw_target in pattern.findall(text):
            target = raw_target.strip().split(maxsplit=1)[0].strip("<>")
            if not target or target.startswith(("#", "mailto:")) or "://" in target:
                continue
            target = unquote(target.split("#", 1)[0].split("?", 1)[0])
            if not target:
                continue
            resolved = (path.parent / target).resolve()
            if not resolved.exists():
                fail(f"broken local Markdown link in {relative(path)}: {raw_target}")


def summary_chapter_targets() -> list[tuple[str, Path]]:
    summary = ROOT / "book/src/SUMMARY.md"
    text = summary.read_text(encoding="utf-8")
    targets: list[tuple[str, Path]] = []
    for label, target in re.findall(r"\[([^\]]+)\]\(([^)]+)\)", text):
        if "://" in target or target.startswith("#"):
            continue
        resolved = (summary.parent / target).resolve()
        if not resolved.is_file():
            fail(f"SUMMARY target does not exist: {target}")
        relative_target = resolved.relative_to(summary.parent)
        is_reader_resource = relative_target.parts[0].startswith("_")
        if resolved.name != "index.md" and not is_reader_resource:
            targets.append((label, resolved))
    paths = [path for _, path in targets]
    if len(paths) != len(set(paths)):
        fail("SUMMARY contains duplicate chapter targets")
    return targets


def chapter_ids_from_map() -> list[str]:
    path = ROOT / "docs/04-CHAPTER-MAP.md"
    ids = [f"ch{value}" for value in re.findall(r"^\|\s*(\d{2})\s*\|", path.read_text(encoding="utf-8"), re.M)]
    expected = [f"ch{number:02d}" for number in range(len(ids))]
    if ids != expected:
        fail(f"chapter map IDs must be contiguous from ch00: {ids}")
    return ids


def check_chapters() -> tuple[set[str], set[str]]:
    chapter_ids = chapter_ids_from_map()
    entries = summary_chapter_targets()
    completed_ids: set[str] = set()
    if len(entries) != len(chapter_ids):
        fail(
            "chapter count mismatch between SUMMARY and chapter map: "
            f"{len(entries)} != {len(chapter_ids)}"
        )

    for chapter_id, (label, target) in zip(chapter_ids, entries, strict=True):
        expected_prefix = chapter_id.removeprefix("ch") + "-"
        if not target.name.startswith(expected_prefix):
            fail(
                f"chapter order/path mismatch for {chapter_id}: "
                f"{relative(target)}"
            )
        text = target.read_text(encoding="utf-8")
        heading_match = re.search(r"^#\s+(.+)$", text, re.M)
        if heading_match is None or heading_match.group(1).strip() != label.strip():
            fail(
                f"SUMMARY label and chapter heading differ for {chapter_id}: "
                f"{label!r} != {heading_match.group(1).strip() if heading_match else None!r}"
            )
        goal_match = re.search(r"chapter_goal:\s*(.+)", text)
        if goal_match is None or not goal_match.group(1).strip():
            fail(f"chapter metadata has no chapter_goal: {relative(target)}")
        goal = goal_match.group(1).strip()
        if f"Harness milestone / Harness 里程碑:** {goal}" not in text:
            fail(f"chapter goal is not visible in the chapter shell: {relative(target)}")
        status_match = re.search(r"chapter_status:\s*([a-z-]+)", text)
        if status_match is None:
            fail(f"chapter metadata has no chapter_status: {relative(target)}")
        status = status_match.group(1)
        if status == "completed":
            completed_ids.add(chapter_id)
        elif status != "planned":
            fail(f"unsupported chapter status for {chapter_id}: {status}")
        if status == "planned":
            if "Status / 状态:** Planned" not in text:
                fail(f"planned chapter is not visibly labelled: {relative(target)}")
            if "intentionally a placeholder" not in text:
                fail(f"planned chapter does not disclose placeholder status: {relative(target)}")

    scaffold = load_json(ROOT / "scaffold.json")
    planned = int(scaffold.get("planned_chapter_count", -1))
    completed = int(scaffold.get("completed_chapter_count", -1))
    if planned != len(chapter_ids):
        fail(f"scaffold planned_chapter_count is {planned}, expected {len(chapter_ids)}")
    completed_actual = sum(
        1
        for _, target in entries
        if re.search(r"chapter_status:\s*completed", target.read_text(encoding="utf-8"))
    )
    if completed != completed_actual:
        fail(
            f"scaffold completed_chapter_count is {completed}, "
            f"actual is {completed_actual}"
        )

    return set(chapter_ids), completed_ids


def check_terms(chapter_ids: set[str]) -> None:
    data = load_toml(ROOT / "book/terms.toml")
    ids: set[str] = set()
    pairs: set[tuple[str, str]] = set()
    for term in data.get("term", []):
        term_id = term.get("id")
        zh = term.get("zh")
        en = term.get("en")
        introduced = term.get("introduced_in")
        if not isinstance(term_id, str) or not term_id:
            fail("every term needs a non-empty id")
        if not isinstance(zh, str) or not isinstance(en, str):
            fail(f"term {term_id} needs Chinese and English forms")
        if term_id in ids:
            fail(f"duplicate term id: {term_id}")
        pair = (zh, en)
        if pair in pairs:
            fail(f"duplicate bilingual term: {pair}")
        if introduced not in chapter_ids:
            fail(f"term {term_id} has invalid introduced_in: {introduced}")
        ids.add(term_id)
        pairs.add(pair)


def chapter_metadata_examples(target: Path) -> set[str]:
    text = target.read_text(encoding="utf-8")
    match = re.search(r"^examples:\s*\[([^]]*)\]", text, re.M)
    if match is None:
        return set()
    return {
        item.strip()
        for item in match.group(1).split(",")
        if item.strip()
    }


def check_examples(chapter_ids: set[str], completed_ids: set[str]) -> set[str]:
    data = load_toml(ROOT / "book/examples.toml")
    ids: set[str] = set()
    example_chapters: set[str] = set()
    for example in data.get("example", []):
        example_id = example.get("id")
        if not isinstance(example_id, str) or not example_id:
            fail("every registered example needs a non-empty id")
        if example_id in ids:
            fail(f"duplicate example id: {example_id}")
        ids.add(example_id)
        chapter = example.get("chapter")
        if chapter not in chapter_ids:
            fail(f"example {example_id} has invalid chapter: {chapter}")
        example_chapters.add(chapter)
        source = example.get("source")
        source_path = ROOT / source if isinstance(source, str) else None
        if source_path is None or not source_path.is_file():
            fail(f"example {example_id} source does not exist: {source}")
        anchor = example.get("anchor")
        if anchor is not None:
            if not isinstance(anchor, str) or not anchor:
                fail(f"example {example_id} has an invalid anchor")
            source_text = source_path.read_text(encoding="utf-8")
            if f"ANCHOR: {anchor}" not in source_text:
                fail(f"example {example_id} start anchor does not exist: {anchor}")
            if f"ANCHOR_END: {anchor}" not in source_text:
                fail(f"example {example_id} end anchor does not exist: {anchor}")
        command = example.get("command")
        if not isinstance(command, list) or not command or not all(isinstance(item, str) for item in command):
            fail(f"example {example_id} needs a non-empty string command array")
        if not isinstance(example.get("offline"), bool):
            fail(f"example {example_id} needs an offline boolean")

    targets = summary_chapter_targets()
    for chapter_id, (_, target) in zip(chapter_ids_from_map(), targets, strict=True):
        if chapter_id not in completed_ids:
            continue
        metadata_ids = chapter_metadata_examples(target)
        if not metadata_ids:
            fail(f"completed chapter has no examples metadata: {chapter_id}")
        unknown = metadata_ids - ids
        if unknown:
            fail(f"completed chapter {chapter_id} has unregistered examples: {sorted(unknown)}")
        if chapter_id not in example_chapters:
            fail(f"completed chapter has no registered examples: {chapter_id}")

    return ids


def check_completed_receipts(completed_ids: set[str]) -> None:
    sources = load_toml(ROOT / "book/sources.lock.toml")
    expected_versions = {
        "rust": sources.get("rust", {}).get("toolchain"),
        "mdbook": sources.get("mdbook", {}).get("version"),
        "typst": sources.get("typst", {}).get("version"),
        "codex_commit": sources.get("codex", {}).get("commit"),
    }

    for chapter_id in sorted(completed_ids):
        receipt_path = ROOT / "chapters" / chapter_id / "receipt.toml"
        if not receipt_path.is_file():
            fail(f"completed chapter has no receipt: {chapter_id}")
        receipt = load_toml(receipt_path)
        if receipt.get("schema_version") != 1:
            fail(f"completed chapter has unsupported receipt schema: {chapter_id}")
        if receipt.get("id") != chapter_id or receipt.get("status") != "completed":
            fail(f"completed chapter receipt identity/status mismatch: {chapter_id}")
        if receipt.get("offline") is not True:
            fail(f"completed chapter receipt must confirm offline verification: {chapter_id}")
        for key, expected in expected_versions.items():
            if receipt.get(key) != expected:
                fail(f"completed chapter receipt {key} does not match source ledger: {chapter_id}")
        verification = receipt.get("verification")
        if not isinstance(verification, dict):
            fail(f"completed chapter receipt has no verification table: {chapter_id}")
        required_tests = verification.get("required_tests")
        commands = verification.get("commands")
        if not isinstance(required_tests, list) or not required_tests or not all(isinstance(item, str) for item in required_tests):
            fail(f"completed chapter receipt has invalid required_tests: {chapter_id}")
        if not isinstance(commands, list) or not commands or not all(isinstance(item, str) for item in commands):
            fail(f"completed chapter receipt has invalid commands: {chapter_id}")
        for key in ("html", "pdf_build", "pdf_content_parity"):
            if not isinstance(verification.get(key), str) or not verification[key]:
                fail(f"completed chapter receipt has invalid {key}: {chapter_id}")


def check_version_pins() -> None:
    sources = load_toml(ROOT / "book/sources.lock.toml")
    toolchain = load_toml(ROOT / "rust-toolchain.toml")
    workspace = load_toml(ROOT / "Cargo.toml")
    scaffold = load_json(ROOT / "scaffold.json")

    rust = sources.get("rust", {})
    pinned_rust = rust.get("toolchain")
    if toolchain.get("toolchain", {}).get("channel") != pinned_rust:
        fail("rust-toolchain.toml does not match book/sources.lock.toml")
    if scaffold.get("rust") != pinned_rust:
        fail("scaffold.json Rust version does not match the source ledger")

    package = workspace.get("workspace", {}).get("package", {})
    if package.get("edition") != rust.get("edition"):
        fail("workspace edition does not match the source ledger")
    if workspace.get("workspace", {}).get("resolver") != rust.get("resolver"):
        fail("workspace resolver does not match the source ledger")
    rust_version = str(package.get("rust-version", ""))
    if not str(pinned_rust).startswith(rust_version):
        fail("workspace rust-version is inconsistent with the pinned toolchain")

    if scaffold.get("edition") != rust.get("edition"):
        fail("scaffold.json edition does not match the source ledger")
    if scaffold.get("mdbook") != sources.get("mdbook", {}).get("version"):
        fail("scaffold.json mdBook version does not match the source ledger")
    if scaffold.get("typst") != sources.get("typst", {}).get("version"):
        fail("scaffold.json Typst version does not match the source ledger")

    codex_sha = sources.get("codex", {}).get("commit")
    if not isinstance(codex_sha, str) or not SHA_RE.fullmatch(codex_sha):
        fail("Codex commit must be a full 40-character lowercase SHA")
    if scaffold.get("codex_commit") != codex_sha:
        fail("scaffold.json Codex commit does not match the source ledger")


def check_action_pins() -> None:
    workflow = ROOT / ".github/workflows/ci.yml"
    if not workflow.is_file():
        fail("missing .github/workflows/ci.yml")
    text = workflow.read_text(encoding="utf-8")
    actions = load_toml(ROOT / "book/sources.lock.toml").get("github_actions", {})
    expected = {
        "actions/checkout": actions.get("checkout"),
        "dtolnay/rust-toolchain": actions.get("rust_toolchain"),
        "typst-community/setup-typst": actions.get("typst"),
    }
    for name, sha in expected.items():
        if not isinstance(sha, str) or not SHA_RE.fullmatch(sha):
            fail(f"invalid GitHub Action SHA in source ledger: {name}")
        if f"uses: {name}@{sha}" not in text:
            fail(f"CI workflow does not use the pinned action: {name}@{sha}")
    for match in re.findall(r"uses:\s*[^\s@]+@([^\s#]+)", text):
        if not SHA_RE.fullmatch(match):
            fail(f"CI action is not pinned to a full commit SHA: @{match}")


def check_text_hygiene() -> None:
    for path in sorted(ROOT.rglob("*")):
        if is_excluded(path) or not path.is_file() or path.suffix not in TEXT_SUFFIXES:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError as exc:
            fail(f"non-UTF-8 text file {relative(path)}: {exc}")
        for number, line in enumerate(text.splitlines(), start=1):
            if line.endswith((" ", "\t")):
                fail(f"trailing whitespace: {relative(path)}:{number}")
        if "\t" in text and path.name != "Makefile":
            fail(f"tab found in text file: {relative(path)}")


def check_unwanted_files() -> None:
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or is_excluded(path):
            continue
        if path.name in {".DS_Store"} or path.suffix in {".pyc", ".pyo"}:
            fail(f"generated file must not be committed: {relative(path)}")


def check_css_balance() -> None:
    path = ROOT / "book/theme/cards.css"
    text = re.sub(r"/\*.*?\*/", "", path.read_text(encoding="utf-8"), flags=re.S)
    if text.count("{") != text.count("}"):
        fail("unbalanced braces in book/theme/cards.css")


def main() -> None:
    parse_data_files()
    check_markdown_links()
    chapter_ids, completed_ids = check_chapters()
    check_terms(chapter_ids)
    check_examples(chapter_ids, completed_ids)
    check_completed_receipts(completed_ids)
    check_version_pins()
    check_action_pins()
    check_text_hygiene()
    check_unwanted_files()
    check_css_balance()
    print(
        "scaffold checks passed "
        f"({len(chapter_ids)} chapters, {len(completed_ids)} completed)"
    )


if __name__ == "__main__":
    main()
