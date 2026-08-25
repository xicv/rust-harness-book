#![forbid(unsafe_code)]
#![doc = "Canonical Markdown-to-Typst renderer for the Rust book."]

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

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

/// Converts one Markdown fragment into Typst source.
///
/// # Errors
///
/// Returns an error when the source contains malformed or unsupported structure.
pub fn render_markdown(markdown: &str) -> Result<String, RenderError> {
    Ok(markdown.to_owned())
}

/// Renders the canonical mdBook source into a generated Typst project.
///
/// # Errors
///
/// Returns an error when configuration, source, includes, assets, or output cannot
/// be read or written safely.
pub fn render(_request: &RenderRequest) -> Result<RenderReport, RenderError> {
    Err(RenderError::new("book-render is not implemented"))
}
