use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use book_render::{GitBookExportRequest, export_gitbook};

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(1);

#[test]
fn gitbook_export_reuses_canonical_parsing_and_expands_includes() {
    let project = TestProject::new("complete");
    project.write(
        "book/book.toml",
        concat!(
            "[book]\n",
            "title = \"Fixture Book\"\n",
            "authors = [\"Test Author\"]\n",
            "description = \"A deterministic export fixture.\"\n",
            "language = \"zh-CN\"\n",
            "src = \"src\"\n",
        ),
    );
    project.write(
        "book/src/SUMMARY.md",
        concat!(
            "# Summary\n\n",
            "- [Introduction](index.md)\n\n",
            "# Part One / 第一部分\n",
            "- [Chapter One](chapter.md)\n",
        ),
    );
    project.write(
        "book/src/index.md",
        concat!(
            "# Fixture Book\n\n",
            "<div class=\"learning-card\">\n",
            "<p class=\"card-label\">Outcome / 本章成果</p>\n\n",
            "Read <span class=\"term-en\">Evidence</span>, then open ",
            "[Chapter One](chapter.md).\n",
            "</div>\n",
        ),
    );
    project.write(
        "book/src/chapter.md",
        concat!(
            "# Chapter One\n\n",
            "```rust,editable\n",
            "{{#rustdoc_include ../../snippets/example.rs:demo}}\n",
            "```\n\n",
            "![Diagram](../../images/diagram.svg \"Diagram caption\")\n\n",
            "[Back to the Introduction](index.md)\n",
        ),
    );
    project.write(
        "snippets/example.rs",
        concat!(
            "fn hidden_before() {}\n",
            "// ANCHOR: demo\n",
            "fn shown() {}\n",
            "// ANCHOR_END: demo\n",
            "fn hidden_after() {}\n",
        ),
    );
    project.write(
        "images/diagram.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"10\" height=\"10\"></svg>\n",
    );
    let request = GitBookExportRequest::new(project.path("book"), project.path("dist/gitbook"));

    let report = export_gitbook(&request).unwrap_or_else(|error| panic!("export failed: {error}"));

    assert_eq!(report.chapter_count(), 2);
    assert_eq!(report.asset_count(), 1);
    assert_eq!(report.output_directory(), project.path("dist/gitbook"));
    assert_eq!(
        read_file(&project.path("dist/gitbook/.gitbook.yaml")),
        "root: ./\n\nstructure:\n  readme: README.md\n  summary: SUMMARY.md\n"
    );
    assert!(
        read_file(&project.path("dist/gitbook/SUMMARY.md")).contains("- [Introduction](README.md)")
    );
    let readme = read_file(&project.path("dist/gitbook/README.md"));
    assert!(!readme.contains("<div"));
    assert!(!readme.contains("<span"));
    assert!(readme.contains("> **Outcome / 本章成果**"));
    assert!(readme.contains("[Chapter One](chapter.md)"));
    let chapter = read_file(&project.path("dist/gitbook/chapter.md"));
    assert!(chapter.contains("```rust\nfn shown() {}\n```"));
    assert!(!chapter.contains("rustdoc_include"));
    assert!(!chapter.contains("hidden_before"));
    assert!(chapter.contains("![Diagram](assets/asset-0000.svg \"Diagram caption\")"));
    assert!(chapter.contains("[Back to the Introduction](README.md)"));
    assert!(project.path("dist/gitbook/assets/asset-0000.svg").is_file());
}

struct TestProject {
    root: PathBuf,
}

impl TestProject {
    fn new(name: &str) -> Self {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "rust-harness-gitbook-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&root)
            .unwrap_or_else(|error| panic!("failed to create {}: {error}", root.display()));
        let root = fs::canonicalize(&root)
            .unwrap_or_else(|error| panic!("failed to resolve {}: {error}", root.display()));
        Self { root }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|error| panic!("failed to create {}: {error}", parent.display()));
        }
        fs::write(&path, content)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}
