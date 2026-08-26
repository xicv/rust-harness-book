#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import re
import sys
import tomllib
from urllib.parse import unquote
import zipfile

ROOT = Path(__file__).resolve().parents[1]
PAGES_URL = "https://xicv.github.io/rust-harness-book/"
PACKS = {
    "ch00": ("3fed46defa0189e4e1a8f5b7dc3ab61743209b08", "rust-harness-ch00.zip"),
    "ch01": ("decef67c89afba8e4eb095b0c16454e4aca97eb5", "rust-harness-ch01.zip"),
}


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(relative: str) -> str:
    path = ROOT / relative
    if not path.is_file():
        fail(f"missing reader-delivery file: {relative}")
    return path.read_text(encoding="utf-8")


def load_toml(relative: str) -> dict:
    path = ROOT / relative
    if not path.is_file():
        fail(f"missing reader-delivery TOML: {relative}")
    try:
        with path.open("rb") as handle:
            value = tomllib.load(handle)
    except Exception as exc:  # noqa: BLE001 - validation boundary
        fail(f"invalid TOML in {relative}: {exc}")
    if not isinstance(value, dict):
        fail(f"top-level TOML value must be a table: {relative}")
    return value


def require_fragments(relative: str, fragments: tuple[str, ...]) -> None:
    source = read(relative)
    for fragment in fragments:
        if fragment not in source:
            fail(f"{relative} is missing required delivery evidence: {fragment}")


def check_licences() -> None:
    for relative in ("LICENSE", "LICENSE-BOOK"):
        source = read(relative)
        if not source.startswith("MIT License\n"):
            fail(f"{relative} is not an MIT licence")
        if "Copyright (c) 2026 xicv and contributors" not in source:
            fail(f"{relative} has no project copyright notice")
    require_fragments(
        "docs/08-SOURCE-AND-RIGHTS-POLICY.md",
        ("`LICENSE`", "`LICENSE-BOOK`", "Third-party"),
    )


def check_pack_contract() -> None:
    cargo = load_toml("Cargo.toml")
    members = cargo.get("workspace", {}).get("members", [])
    if "crates/chapter-pack" not in members:
        fail("Cargo workspace does not include crates/chapter-pack")
    require_fragments(
        "crates/chapter-pack/src/lib.rs",
        (
            "pub fn build_and_verify",
            "source_commit is not an ancestor of HEAD",
            "CompressionMethod::Stored",
            "CARGO_NET_OFFLINE",
            "enclosed_name",
        ),
    )
    for chapter, (source_commit, archive_name) in PACKS.items():
        manifest = load_toml(f"chapter-packs/{chapter}.toml")
        if manifest.get("source_commit") != source_commit:
            fail(f"{chapter} pack source commit does not match its reviewed receipt")
        if manifest.get("archive_name") != archive_name:
            fail(f"{chapter} pack archive name is unstable")
        generated = manifest.get("generated_files", [])
        destinations = {
            item.get("destination") for item in generated if isinstance(item, dict)
        }
        if "LICENSE" not in destinations:
            fail(f"{chapter} pack does not carry the code licence")
        commands = manifest.get("commands", [])
        if len(commands) != 2:
            fail(f"{chapter} pack must declare exactly two verification commands")


def check_publication_config() -> None:
    publication = load_toml("book/publication.toml")
    pages = publication.get("github_pages", {})
    if pages.get("url") != PAGES_URL or pages.get("base_path") != "/rust-harness-book/":
        fail("book/publication.toml does not pin the project Pages URL and base path")
    gitbook = publication.get("gitbook", {})
    if gitbook.get("branch") != "gitbook-publish" or gitbook.get("root") != "dist/gitbook":
        fail("book/publication.toml does not pin the generated GitBook branch and root")
    book = load_toml("book/book.toml")
    html = book.get("output", {}).get("html", {})
    if html.get("site-url") != "/rust-harness-book/":
        fail("mdBook site-url does not match the GitHub project Pages base path")
    if html.get("edit-url-template") != (
        "https://github.com/xicv/rust-harness-book/edit/main/book/{path}"
    ):
        fail("mdBook edit URL does not map source paths back into book/src")
    require_fragments(
        ".gitbook.yaml",
        ("root: ./dist/gitbook", "readme: README.md", "summary: SUMMARY.md"),
    )


def check_build_contract() -> None:
    require_fragments(
        "Makefile",
        (
            "chapter-packs:",
            "gitbook-export:",
            "reader-delivery:",
            "--manifest chapter-packs/ch00.toml",
            "--manifest chapter-packs/ch01.toml",
            "--output-dir dist/gitbook",
            "python3 scripts/check_reader_delivery.py --artifacts",
        ),
    )
    require_fragments(
        "crates/book-render/src/render_gitbook.rs",
        (
            "pub fn export_gitbook",
            "IncludeExpander",
            "MarkdownParser",
            "remove_matching_title",
            "remote image must be vendored",
        ),
    )


def check_workflows() -> None:
    require_fragments(".github/workflows/ci.yml", ("fetch-depth: 0", "make reader-delivery"))
    workflow = read(".github/workflows/publish-book.yml")
    required = (
        "pull_request:",
        "branches: [main]",
        "contents: read",
        "fetch-depth: 0",
        "run: make verify",
        "actions/configure-pages@45bfe0192ca1faeb007ade9deae92b16b8254a0d",
        "actions/upload-pages-artifact@fc324d3547104276b827a68afc52ff2a11cc49c9",
        "actions/deploy-pages@cd2ce8fcbc39b97be8ca5fce6e763baed58fa128",
        "pages: write",
        "id-token: write",
        "environment:",
        "name: github-pages",
        "github.event_name != 'pull_request'",
        "contents: write",
        "gitbook-publish",
        "git push origin HEAD:gitbook-publish",
    )
    for fragment in required:
        if fragment not in workflow:
            fail(f"publish workflow is missing: {fragment}")
    if "--force" in workflow or "push --force" in workflow:
        fail("publish workflow must not force-push the GitBook branch")


def check_reader_links() -> None:
    require_fragments(
        "book/src/SUMMARY.md",
        (
            "(_delivery/chapter-code.md)",
            "(_delivery/ch00.md)",
            "(_delivery/ch01.md)",
        ),
    )
    for chapter, (_, archive_name) in PACKS.items():
        download = f"{PAGES_URL}downloads/{chapter}/{archive_name}"
        checksum = f"{download}.sha256"
        require_fragments(
            f"book/src/_delivery/{chapter}.md",
            (download, checksum, "cargo test --workspace --locked"),
        )
        chapter_source = {
            "ch00": "book/src/00-preface/00-one-complete-turn.md",
            "ch01": "book/src/00-preface/01-toolchain-as-evidence.md",
        }[chapter]
        require_fragments(chapter_source, (f"../_delivery/{chapter}.md",))


def check_static_contract() -> None:
    check_licences()
    check_pack_contract()
    check_publication_config()
    check_build_contract()
    check_workflows()
    check_reader_links()


def check_artifacts() -> None:
    for generated_root in (ROOT / "dist/html", ROOT / "dist/gitbook"):
        for path in generated_root.rglob("*"):
            if path.is_symlink():
                fail(f"generated reader artifact must not be a symlink: {path}")
    for relative in (
        "dist/html/index.html",
        "dist/gitbook/.gitbook.yaml",
        "dist/gitbook/README.md",
        "dist/gitbook/SUMMARY.md",
    ):
        if not (ROOT / relative).is_file():
            fail(f"missing generated reader artifact: {relative}")
    gitbook_root = ROOT / "dist/gitbook"
    gitbook_readme = (gitbook_root / "README.md").read_text(encoding="utf-8")
    if PAGES_URL not in gitbook_readme or "我们先把目标说清楚" not in gitbook_readme:
        fail("GitBook README is missing the interactive-edition link or CJK introduction")
    link_pattern = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
    for markdown_path in sorted(gitbook_root.rglob("*.md")):
        source = markdown_path.read_text(encoding="utf-8")
        if "{{#include" in source or "{{#rustdoc_include" in source:
            fail(f"GitBook export contains an unexpanded include: {markdown_path}")
        if re.search(r"</?(?:div|span|p)\b", source, re.IGNORECASE):
            fail(f"GitBook export contains unsupported semantic HTML: {markdown_path}")
        for raw_target in link_pattern.findall(source):
            target = raw_target.strip().split(maxsplit=1)[0].strip("<>")
            if not target or target.startswith(("#", "mailto:")) or "://" in target:
                continue
            target = unquote(target.split("#", 1)[0].split("?", 1)[0])
            if target and not (markdown_path.parent / target).resolve().is_file():
                fail(f"broken local GitBook link in {markdown_path}: {raw_target}")
    for chapter, (_, archive_name) in PACKS.items():
        archive_path = ROOT / f"dist/html/downloads/{chapter}/{archive_name}"
        checksum_path = archive_path.with_name(f"{archive_name}.sha256")
        if not archive_path.is_file() or not checksum_path.is_file():
            fail(f"missing generated {chapter} archive or checksum")
        expected = checksum_path.read_text(encoding="utf-8").strip()
        actual = hashlib.sha256(archive_path.read_bytes()).hexdigest()
        if expected != actual:
            fail(f"generated {chapter} checksum does not match its archive")
        manifest = load_toml(f"chapter-packs/{chapter}.toml")
        root = f"rust-harness-{chapter}/"
        expected_names = sorted(
            f"{root}{path}" for path in manifest.get("source_paths", [])
        )
        expected_names.extend(
            f"{root}{item['destination']}"
            for item in manifest.get("generated_files", [])
            if isinstance(item, dict) and isinstance(item.get("destination"), str)
        )
        expected_names.sort()
        with zipfile.ZipFile(archive_path) as archive:
            names = archive.namelist()
            if names != sorted(names) or len(names) != len(set(names)):
                fail(f"generated {chapter} archive paths are not unique and sorted")
            if names != expected_names:
                fail(f"generated {chapter} archive file list differs from its manifest")
            if f"{root}LICENSE" not in names or f"{root}README.md" not in names:
                fail(f"generated {chapter} archive is missing its licence or README")
            for entry in archive.infolist():
                name = entry.filename
                path = Path(name)
                if path.is_absolute() or ".." in path.parts or not name.startswith(root):
                    fail(f"generated {chapter} archive has an unsafe path: {name}")
                mode = entry.external_attr >> 16
                if mode != 0o100644 or entry.compress_type != zipfile.ZIP_STORED:
                    fail(f"generated {chapter} archive entry metadata is unstable: {name}")
                if entry.date_time != (1980, 1, 1, 0, 0, 0):
                    fail(f"generated {chapter} archive timestamp is unstable: {name}")
        html_page = ROOT / f"dist/html/_delivery/{chapter}.html"
        html = html_page.read_text(encoding="utf-8") if html_page.is_file() else ""
        if f"{PAGES_URL}downloads/{chapter}/{archive_name}" not in html:
            fail(f"generated {chapter} HTML page does not link to its archive")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--artifacts", action="store_true")
    arguments = parser.parse_args()
    check_static_contract()
    if arguments.artifacts:
        check_artifacts()
    print("reader-delivery checks passed")


if __name__ == "__main__":
    main()
