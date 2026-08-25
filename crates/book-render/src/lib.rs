#![forbid(unsafe_code)]
#![doc = "Canonical Markdown-to-Typst renderer for the Rust book."]

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Write as _};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SUMMARY_FILE: &str = "SUMMARY.md";
const BOOK_CONFIG_FILE: &str = "book.toml";
const MAX_INCLUDE_DEPTH: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderRequest {
    book_dir: PathBuf,
    template_path: PathBuf,
    output_dir: PathBuf,
}

impl RenderRequest {
    #[must_use]
    pub fn new(
        book_dir: impl Into<PathBuf>,
        template_path: impl Into<PathBuf>,
        output_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            book_dir: book_dir.into(),
            template_path: template_path.into(),
            output_dir: output_dir.into(),
        }
    }

    #[must_use]
    pub fn book_dir(&self) -> &Path {
        &self.book_dir
    }

    #[must_use]
    pub fn template_path(&self) -> &Path {
        &self.template_path
    }

    #[must_use]
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderReport {
    output_path: PathBuf,
    chapter_count: usize,
    part_count: usize,
    asset_count: usize,
}

impl RenderReport {
    #[must_use]
    pub fn output_path(&self) -> &Path {
        &self.output_path
    }

    #[must_use]
    pub const fn chapter_count(&self) -> usize {
        self.chapter_count
    }

    #[must_use]
    pub const fn part_count(&self) -> usize {
        self.part_count
    }

    #[must_use]
    pub const fn asset_count(&self) -> usize {
        self.asset_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderError {
    message: String,
}

impl RenderError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for RenderError {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BookConfig {
    title: String,
    authors: Vec<String>,
    description: String,
    language: String,
    source: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SummaryEntry {
    Part(String),
    Separator,
    Chapter(SummaryChapter),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SummaryChapter {
    title: String,
    source: PathBuf,
    depth: usize,
    label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Block {
    Heading {
        level: usize,
        content: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    Code {
        language: String,
        source: String,
    },
    BulletList(Vec<ListItem>),
    OrderedList {
        start: u64,
        items: Vec<ListItem>,
    },
    Quote(Vec<Block>),
    Card(Vec<Block>),
    CardLabel(Vec<Inline>),
    Table {
        header: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    Image {
        alt: String,
        path: String,
        caption: Option<String>,
    },
    Rule,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ListItem {
    blocks: Vec<Block>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Inline {
    Text(String),
    Strong(Vec<Inline>),
    Emphasis(Vec<Inline>),
    Strike(Vec<Inline>),
    Code(String),
    Term(Vec<Inline>),
    Link {
        target: String,
        content: Vec<Inline>,
    },
    Break,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ListKind {
    Bullet,
    Ordered,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ListMarker {
    indent: usize,
    kind: ListKind,
    start: u64,
    content: String,
}

struct MarkdownParser {
    lines: Vec<String>,
    index: usize,
}

struct IncludeExpander<'a> {
    project_root: &'a Path,
    stack: Vec<PathBuf>,
}

struct AssetRegistry {
    project_root: PathBuf,
    output_dir: PathBuf,
    by_source: HashMap<PathBuf, String>,
}

struct RenderEnvironment<'a> {
    chapter_path: Option<&'a Path>,
    project_root: Option<&'a Path>,
    chapter_labels: &'a HashMap<PathBuf, String>,
    assets: Option<&'a mut AssetRegistry>,
    heading_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ResolvedLink {
    External(String),
    Internal(String),
}

/// Converts one Markdown fragment into Typst source.
///
/// This is the strict, dependency-free subset used by the book renderer. It
/// preserves the current card, inline-style, list, quote, table, and code-block
/// contract. File includes and images require [`render`] because they need a
/// project root and output directory.
///
/// # Errors
///
/// Returns an error when the source contains malformed or unsupported structure.
pub fn render_markdown(markdown: &str) -> Result<String, RenderError> {
    if markdown.contains("{{#include") || markdown.contains("{{#rustdoc_include") {
        return Err(RenderError::new(
            "file includes require the complete book renderer",
        ));
    }

    let cleaned = strip_html_comments(markdown)?;
    let blocks = MarkdownParser::new(&cleaned).parse()?;
    let labels = HashMap::new();
    let mut environment = RenderEnvironment {
        chapter_path: None,
        project_root: None,
        chapter_labels: &labels,
        assets: None,
        heading_index: 0,
    };
    let mut output = String::new();
    render_blocks(&blocks, &mut environment, &mut output)?;
    Ok(output)
}

/// Renders the canonical mdBook source into a generated Typst project.
///
/// `book/src/SUMMARY.md` controls order. `{{#include ...}}` and
/// `{{#rustdoc_include ...}}` are expanded relative to each chapter. All
/// chapters, includes, images, and the template must remain inside the common
/// project root. Unsupported structures fail closed.
///
/// # Errors
///
/// Returns an error when configuration, source, includes, assets, or output cannot
/// be read or written safely.
pub fn render(request: &RenderRequest) -> Result<RenderReport, RenderError> {
    let book_dir = canonical_existing(request.book_dir(), "book directory")?;
    let template_path = canonical_existing(request.template_path(), "Typst template")?;
    let template_parent = template_path.parent().ok_or_else(|| {
        RenderError::new("Typst template has no parent directory")
    })?;
    let project_root = common_ancestor(&book_dir, template_parent).ok_or_else(|| {
        RenderError::new("book directory and Typst template have no common project root")
    })?;
    if project_root.parent().is_none() {
        return Err(RenderError::new(
            "refusing to use the filesystem root as the project root",
        ));
    }

    ensure_inside(&project_root, &book_dir, "book directory")?;
    ensure_inside(&project_root, &template_path, "Typst template")?;

    let config_path = book_dir.join(BOOK_CONFIG_FILE);
    let config_source = read_utf8(&config_path, "book configuration")?;
    let config = parse_book_config(&config_source)?;
    let source_dir = canonical_existing(&book_dir.join(&config.source), "book source directory")?;
    ensure_inside(&project_root, &source_dir, "book source directory")?;

    let summary_path = source_dir.join(SUMMARY_FILE);
    let summary_source = read_utf8(&summary_path, "SUMMARY.md")?;
    let mut entries = parse_summary(&summary_source, &source_dir, &project_root)?;
    assign_chapter_labels(&mut entries);
    let chapter_labels = chapter_label_map(&entries);

    let output_dir = safe_output_directory(request.output_dir(), &project_root)?;
    let staging_dir = sibling_work_path(&output_dir, "staging");
    let backup_dir = sibling_work_path(&output_dir, "backup");
    remove_if_exists(&staging_dir, "stale renderer staging directory")?;
    remove_if_exists(&backup_dir, "stale renderer backup directory")?;
    fs::create_dir_all(staging_dir.join("assets")).map_err(|error| {
        io_context(
            "create renderer staging directory",
            &staging_dir,
            error,
        )
    })?;

    let result = render_into_staging(
        &config,
        &entries,
        &chapter_labels,
        &project_root,
        &template_path,
        &staging_dir,
    );

    let (generated, asset_count) = match result {
        Ok(value) => value,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(error);
        }
    };

    let generated_path = staging_dir.join("book.typ");
    fs::write(&generated_path, generated).map_err(|error| {
        io_context("write generated Typst book", &generated_path, error)
    })?;
    let staged_template = staging_dir.join("template.typ");
    fs::copy(&template_path, &staged_template).map_err(|error| {
        io_context("copy Typst template", &staged_template, error)
    })?;

    replace_output_directory(&staging_dir, &output_dir, &backup_dir)?;

    let chapter_count = entries
        .iter()
        .filter(|entry| matches!(entry, SummaryEntry::Chapter(_)))
        .count();
    let part_count = entries
        .iter()
        .filter(|entry| matches!(entry, SummaryEntry::Part(_)))
        .count();

    Ok(RenderReport {
        output_path: output_dir.join("book.typ"),
        chapter_count,
        part_count,
        asset_count,
    })
}

include!("render_typst.rs");
include!("markdown.rs");
include!("includes.rs");
include!("config.rs");
include!("paths.rs");
