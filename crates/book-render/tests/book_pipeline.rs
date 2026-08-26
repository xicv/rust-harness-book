use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use book_render::{RenderRequest, render};

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(1);

struct TestProject {
    root: PathBuf,
}

impl TestProject {
    fn new(name: &str) -> Self {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "rust-harness-book-{name}-{}-{sequence}",
            std::process::id()
        ));
        if root.exists() {
            let _ = fs::remove_dir_all(&root);
        }
        create_dir(&root);
        let root = match fs::canonicalize(&root) {
            Ok(path) => path,
            Err(error) => panic!("failed to canonicalize {}: {error}", root.display()),
        };
        Self { root }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write(&self, relative: &str, content: &str) {
        write_file(&self.path(relative), content);
    }

    fn request(&self) -> RenderRequest {
        RenderRequest::new(
            self.path("book"),
            self.path("typst/template.typ"),
            self.path("dist/typst"),
        )
    }

    fn write_base_book(&self, chapter: &str) {
        self.write(
            "book/book.toml",
            concat!(
                "[book]\n",
                "title = \"Fixture Book\"\n",
                "authors = [\"Test Author\"]\n",
                "description = \"A deterministic renderer fixture.\"\n",
                "language = \"zh-CN\"\n",
                "src = \"src\"\n",
            ),
        );
        self.write(
            "book/src/SUMMARY.md",
            concat!(
                "# Summary\n\n",
                "- [Introduction](index.md)\n\n",
                "# Part One / 第一部分\n",
                "- [Chapter One](chapter.md)\n",
            ),
        );
        self.write(
            "book/src/index.md",
            "# Introduction\n\nIntro before the part.\n",
        );
        self.write("book/src/chapter.md", chapter);
        self.write("typst/template.typ", "#let book(body, ..args) = body\n");
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn render_expands_sources_orders_summary_and_copies_assets() {
    let project = TestProject::new("complete");
    project.write_base_book(concat!(
        "# Chapter One\n\n",
        "<div class=\"learning-card\">\n",
        "<p class=\"card-label\">Outcome / 本章成果</p>\n\n",
        "A **tested** chapter with <span class=\"term-en\">Evidence</span>.\n\n",
        "```rust\n",
        "{{#rustdoc_include ../../snippets/example.rs:demo}}\n",
        "```\n\n",
        "![Diagram alt](../../images/diagram.svg \"Diagram caption\")\n",
        "</div>\n",
    ));
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

    let Ok(report) = render(&project.request()) else {
        panic!("the complete fixture should render");
    };

    assert_eq!(report.chapter_count(), 2);
    assert_eq!(report.part_count(), 1);
    assert_eq!(report.asset_count(), 1);
    assert_eq!(report.output_path(), project.path("dist/typst/book.typ"));

    let generated = read_file(report.output_path());
    let intro = position(&generated, "#text(\"Introduction\")");
    let part = position(&generated, "#text(\"Part One / 第一部分\")");
    let chapter = position(&generated, "#text(\"Chapter One\")");
    assert!(intro < part && part < chapter);
    assert!(generated.contains("fn shown() {}"));
    assert!(!generated.contains("hidden_before"));
    assert!(!generated.contains("hidden_after"));
    assert!(generated.contains("#card("));
    assert!(generated.contains("#book-image("));
    assert!(project.path("dist/typst/assets/asset-0000.svg").is_file());
    assert!(project.path("dist/typst/template.typ").is_file());
}

#[test]
fn include_cannot_escape_the_project_root() {
    let project = TestProject::new("escape");
    let outside = project
        .root
        .parent()
        .unwrap_or_else(|| Path::new("/"))
        .join(format!("book-render-outside-{}.rs", std::process::id()));
    write_file(&outside, "fn outside() {}\n");
    project.write_base_book(&format!(
        "# Chapter One\n\n```rust\n{{{{#include {}}}}}\n```\n",
        outside.display()
    ));

    let error = match render(&project.request()) {
        Ok(_) => panic!("an include outside the project must fail"),
        Err(error) => error,
    };
    let _ = fs::remove_file(&outside);

    assert!(error.to_string().contains("outside project root"));
}

#[test]
fn recursive_includes_fail_closed() {
    let project = TestProject::new("recursive");
    project.write_base_book("# Chapter One\n\n```text\n{{#include ../../snippets/a.txt}}\n```\n");
    project.write("snippets/a.txt", "A\n{{#include b.txt}}\n");
    project.write("snippets/b.txt", "B\n{{#include a.txt}}\n");

    let error = match render(&project.request()) {
        Ok(_) => panic!("recursive includes must fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("recursive include detected"));
}

#[test]
fn remote_images_must_be_vendored() {
    let project = TestProject::new("remote-image");
    project.write_base_book("# Chapter One\n\n![Remote](https://example.com/diagram.png)\n");

    let error = match render(&project.request()) {
        Ok(_) => panic!("remote images must fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("remote image must be vendored"));
}

fn create_dir(path: &Path) {
    if let Err(error) = fs::create_dir_all(path) {
        panic!("failed to create {}: {error}", path.display());
    }
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        create_dir(parent);
    }
    if let Err(error) = fs::write(path, content) {
        panic!("failed to write {}: {error}", path.display());
    }
}

fn read_file(path: &Path) -> String {
    match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => panic!("failed to read {}: {error}", path.display()),
    }
}

fn position(haystack: &str, needle: &str) -> usize {
    match haystack.find(needle) {
        Some(index) => index,
        None => panic!("generated source is missing {needle:?}"),
    }
}
