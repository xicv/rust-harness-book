#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitBookExportRequest {
    book_dir: PathBuf,
    output_dir: PathBuf,
}

impl GitBookExportRequest {
    #[must_use]
    pub fn new(book_dir: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            book_dir: book_dir.into(),
            output_dir: output_dir.into(),
        }
    }

    #[must_use]
    pub fn book_dir(&self) -> &Path {
        &self.book_dir
    }

    #[must_use]
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitBookExportReport {
    output_directory: PathBuf,
    chapter_count: usize,
    asset_count: usize,
}

impl GitBookExportReport {
    #[must_use]
    pub fn output_directory(&self) -> &Path {
        &self.output_directory
    }

    #[must_use]
    pub const fn chapter_count(&self) -> usize {
        self.chapter_count
    }

    #[must_use]
    pub const fn asset_count(&self) -> usize {
        self.asset_count
    }
}

struct GitBookEnvironment<'a> {
    chapter_path: &'a Path,
    output_path: &'a Path,
    project_root: &'a Path,
    chapter_outputs: &'a HashMap<PathBuf, PathBuf>,
    assets: &'a mut AssetRegistry,
}

/// Exports the canonical book into a GitBook-compatible Markdown tree.
///
/// The export uses the same strict SUMMARY parser, include expander, HTML
/// allowlist, and Markdown AST as the Typst renderer. The generated tree is a
/// disposable publication artifact; `book/src` remains canonical.
///
/// # Errors
///
/// Returns an error when any canonical source, include, link, asset, or output
/// path violates the book renderer contract.
pub fn export_gitbook(request: &GitBookExportRequest) -> Result<GitBookExportReport, RenderError> {
    let book_dir = canonical_existing(request.book_dir(), "book directory")?;
    let project_root = book_dir
        .parent()
        .ok_or_else(|| RenderError::new("book directory has no project root"))?
        .to_owned();
    if project_root.parent().is_none() {
        return Err(RenderError::new(
            "refusing to use the filesystem root as the project root",
        ));
    }
    ensure_inside(&project_root, &book_dir, "book directory")?;

    let config_source = read_utf8(&book_dir.join(BOOK_CONFIG_FILE), "book configuration")?;
    let config = parse_book_config(&config_source)?;
    let source_dir = canonical_existing(&book_dir.join(&config.source), "book source directory")?;
    ensure_inside(&project_root, &source_dir, "book source directory")?;
    let summary_source = read_utf8(&source_dir.join(SUMMARY_FILE), "SUMMARY.md")?;
    let mut entries = parse_summary(&summary_source, &source_dir, &project_root)?;
    assign_chapter_labels(&mut entries);
    let chapter_outputs = gitbook_chapter_outputs(&entries, &source_dir)?;

    let git_control_path = project_root.join(".git");
    let typst_template_path = project_root.join("typst/template.typ");
    let protected_paths = [
        (book_dir.as_path(), "book directory"),
        (source_dir.as_path(), "book source directory"),
        (typst_template_path.as_path(), "Typst template"),
        (git_control_path.as_path(), "Git control path"),
    ];
    let output_dir = safe_output_directory(request.output_dir(), &project_root, &protected_paths)?;
    let staging_dir = sibling_work_path(&output_dir, "gitbook-staging");
    let backup_dir = sibling_work_path(&output_dir, "gitbook-backup");
    remove_if_exists(&staging_dir, "stale GitBook staging directory")?;
    remove_if_exists(&backup_dir, "stale GitBook backup directory")?;
    fs::create_dir_all(staging_dir.join("assets"))
        .map_err(|error| io_context("create GitBook staging directory", &staging_dir, error))?;

    let result =
        export_gitbook_into_staging(&entries, &chapter_outputs, &project_root, &staging_dir);
    let asset_count = match result {
        Ok(count) => count,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(error);
        }
    };

    let generated_summary = render_gitbook_summary(&entries, &chapter_outputs)?;
    let summary_path = staging_dir.join(SUMMARY_FILE);
    fs::write(&summary_path, generated_summary)
        .map_err(|error| io_context("write GitBook SUMMARY.md", &summary_path, error))?;
    let gitbook_config_path = staging_dir.join(".gitbook.yaml");
    fs::write(
        &gitbook_config_path,
        "root: ./\n\nstructure:\n  readme: README.md\n  summary: SUMMARY.md\n",
    )
    .map_err(|error| io_context("write GitBook configuration", &gitbook_config_path, error))?;

    replace_output_directory(&staging_dir, &output_dir, &backup_dir)?;
    Ok(GitBookExportReport {
        output_directory: output_dir,
        chapter_count: chapter_outputs.len(),
        asset_count,
    })
}

fn gitbook_chapter_outputs(
    entries: &[SummaryEntry],
    source_dir: &Path,
) -> Result<HashMap<PathBuf, PathBuf>, RenderError> {
    let mut outputs = HashMap::new();
    let mut destinations = HashSet::new();
    for entry in entries {
        let SummaryEntry::Chapter(chapter) = entry else {
            continue;
        };
        let relative = chapter.source.strip_prefix(source_dir).map_err(|_| {
            RenderError::new(format!(
                "chapter is outside the book source directory: {}",
                chapter.source.display()
            ))
        })?;
        let destination = if relative == Path::new("index.md") {
            PathBuf::from("README.md")
        } else {
            relative.to_owned()
        };
        validate_gitbook_relative_path(&destination)?;
        if !destinations.insert(destination.clone()) {
            return Err(RenderError::new(format!(
                "two chapters map to the same GitBook path: {}",
                destination.display()
            )));
        }
        outputs.insert(chapter.source.clone(), destination);
    }
    Ok(outputs)
}

fn export_gitbook_into_staging(
    entries: &[SummaryEntry],
    chapter_outputs: &HashMap<PathBuf, PathBuf>,
    project_root: &Path,
    staging_dir: &Path,
) -> Result<usize, RenderError> {
    let mut assets = AssetRegistry::new(project_root, &staging_dir.join("assets"));
    let mut expander = IncludeExpander {
        project_root,
        stack: Vec::new(),
    };
    for entry in entries {
        let SummaryEntry::Chapter(chapter) = entry else {
            continue;
        };
        let source = expander.expand_file(&chapter.source)?;
        let cleaned = strip_html_comments(&source)?;
        let blocks = MarkdownParser::new(&cleaned).parse()?;
        let mut title_check = blocks.clone();
        remove_matching_title(&mut title_check, chapter)?;
        let output_path = chapter_outputs.get(&chapter.source).ok_or_else(|| {
            RenderError::new(format!(
                "chapter has no GitBook destination: {}",
                chapter.source.display()
            ))
        })?;
        let mut environment = GitBookEnvironment {
            chapter_path: &chapter.source,
            output_path,
            project_root,
            chapter_outputs,
            assets: &mut assets,
        };
        let markdown = render_gitbook_blocks(&blocks, &mut environment)?;
        let destination = staging_dir.join(output_path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| io_context("create GitBook chapter directory", parent, error))?;
        }
        fs::write(&destination, markdown)
            .map_err(|error| io_context("write GitBook chapter", &destination, error))?;
    }
    Ok(assets.len())
}

fn render_gitbook_summary(
    entries: &[SummaryEntry],
    chapter_outputs: &HashMap<PathBuf, PathBuf>,
) -> Result<String, RenderError> {
    let mut output = String::from("# Summary\n\n");
    for entry in entries {
        match entry {
            SummaryEntry::Part(title) => {
                writeln!(output, "# {title}").map_err(format_error)?;
            }
            SummaryEntry::Separator => output.push_str("---\n"),
            SummaryEntry::Chapter(chapter) => {
                let destination = chapter_outputs.get(&chapter.source).ok_or_else(|| {
                    RenderError::new("chapter is missing its GitBook summary destination")
                })?;
                writeln!(
                    output,
                    "{}- [{}]({})",
                    "  ".repeat(chapter.depth),
                    escape_markdown_text(&chapter.title),
                    path_to_markdown(destination)?
                )
                .map_err(format_error)?;
            }
        }
        output.push('\n');
    }
    Ok(output)
}

fn render_gitbook_blocks(
    blocks: &[Block],
    environment: &mut GitBookEnvironment<'_>,
) -> Result<String, RenderError> {
    let mut output = String::new();
    for block in blocks {
        let rendered = render_gitbook_block(block, environment)?;
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(rendered.trim_end());
        output.push('\n');
    }
    Ok(output)
}

fn render_gitbook_block(
    block: &Block,
    environment: &mut GitBookEnvironment<'_>,
) -> Result<String, RenderError> {
    match block {
        Block::Heading { level, content } => Ok(format!(
            "{} {}\n",
            "#".repeat(*level),
            render_gitbook_inlines(content, environment)?
        )),
        Block::Paragraph(content) => Ok(format!(
            "{}\n",
            render_gitbook_inlines(content, environment)?
        )),
        Block::Code { language, source } => {
            let fence = markdown_fence(source, language);
            Ok(format!("{fence}{language}\n{source}\n{fence}\n"))
        }
        Block::BulletList(items) => render_gitbook_list(items, None, environment),
        Block::OrderedList { start, items } => {
            render_gitbook_list(items, Some(*start), environment)
        }
        Block::Quote(children) => {
            let children = render_gitbook_blocks(children, environment)?;
            Ok(prefix_markdown_lines(&children, "> "))
        }
        Block::Card(children) => {
            let children = render_gitbook_blocks(children, environment)?;
            Ok(prefix_markdown_lines(&children, "> "))
        }
        Block::CardLabel(content) => Ok(format!(
            "**{}**\n",
            render_gitbook_inlines(content, environment)?
        )),
        Block::Table { header, rows } => render_gitbook_table(header, rows, environment),
        Block::Image { alt, path, caption } => {
            render_gitbook_image(alt, path, caption.as_deref(), environment)
        }
        Block::Rule => Ok("---\n".to_owned()),
    }
}

fn render_gitbook_list(
    items: &[ListItem],
    start: Option<u64>,
    environment: &mut GitBookEnvironment<'_>,
) -> Result<String, RenderError> {
    let mut output = String::new();
    for (index, item) in items.iter().enumerate() {
        let rendered = render_gitbook_blocks(&item.blocks, environment)?;
        let marker = match start {
            Some(value) => format!("{}. ", value + index as u64),
            None => "- ".to_owned(),
        };
        for (line_index, line) in rendered.trim_end().lines().enumerate() {
            if line_index == 0 {
                writeln!(output, "{marker}{line}").map_err(format_error)?;
            } else if line.is_empty() {
                output.push('\n');
            } else {
                writeln!(output, "  {line}").map_err(format_error)?;
            }
        }
    }
    Ok(output)
}

fn render_gitbook_table(
    header: &[Vec<Inline>],
    rows: &[Vec<Vec<Inline>>],
    environment: &GitBookEnvironment<'_>,
) -> Result<String, RenderError> {
    if header.is_empty() {
        return Err(RenderError::new("table must have at least one column"));
    }
    let mut output = String::new();
    output.push('|');
    for cell in header {
        write!(output, " {} |", render_gitbook_inlines(cell, environment)?)
            .map_err(format_error)?;
    }
    output.push('\n');
    output.push('|');
    for _ in header {
        output.push_str(" --- |");
    }
    output.push('\n');
    for row in rows {
        if row.len() != header.len() {
            return Err(RenderError::new(
                "table row has a different column count from its header",
            ));
        }
        output.push('|');
        for cell in row {
            write!(output, " {} |", render_gitbook_inlines(cell, environment)?)
                .map_err(format_error)?;
        }
        output.push('\n');
    }
    Ok(output)
}

fn render_gitbook_image(
    alt: &str,
    path: &str,
    caption: Option<&str>,
    environment: &mut GitBookEnvironment<'_>,
) -> Result<String, RenderError> {
    if is_remote_target(path) {
        return Err(RenderError::new(format!(
            "remote image must be vendored: {path}"
        )));
    }
    if alt.trim().is_empty() {
        return Err(RenderError::new("local image must have non-empty alt text"));
    }
    let chapter_parent = environment
        .chapter_path
        .parent()
        .ok_or_else(|| RenderError::new("chapter source has no parent directory"))?;
    let published = environment.assets.register(&chapter_parent.join(path))?;
    let output_parent = environment
        .output_path
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let linked_path = relative_markdown_path(output_parent, Path::new(&published))?;
    let title = caption
        .map(|value| format!(" \"{}\"", value.replace('"', "\\\"")))
        .unwrap_or_default();
    Ok(format!(
        "![{}]({linked_path}{title})\n",
        escape_markdown_text(alt)
    ))
}

fn render_gitbook_inlines(
    content: &[Inline],
    environment: &GitBookEnvironment<'_>,
) -> Result<String, RenderError> {
    let mut output = String::new();
    for inline in content {
        match inline {
            Inline::Text(value) => output.push_str(&escape_markdown_text(value)),
            Inline::Strong(children) => {
                write!(
                    output,
                    "**{}**",
                    render_gitbook_inlines(children, environment)?
                )
                .map_err(format_error)?;
            }
            Inline::Emphasis(children) | Inline::Term(children) => {
                write!(
                    output,
                    "*{}*",
                    render_gitbook_inlines(children, environment)?
                )
                .map_err(format_error)?;
            }
            Inline::Strike(children) => {
                write!(
                    output,
                    "~~{}~~",
                    render_gitbook_inlines(children, environment)?
                )
                .map_err(format_error)?;
            }
            Inline::Code(value) => output.push_str(&markdown_inline_code(value)),
            Inline::Link { target, content } => {
                let destination = resolve_gitbook_link(target, environment)?;
                write!(
                    output,
                    "[{}]({destination})",
                    render_gitbook_inlines(content, environment)?
                )
                .map_err(format_error)?;
            }
            Inline::Break => output.push_str("  \n"),
        }
    }
    Ok(output)
}

fn resolve_gitbook_link(
    target: &str,
    environment: &GitBookEnvironment<'_>,
) -> Result<String, RenderError> {
    if is_remote_target(target) || target.starts_with("mailto:") || target.starts_with('#') {
        return Ok(target.to_owned());
    }
    let (path_part, fragment) = split_once_optional(target, '#');
    let chapter_parent = environment
        .chapter_path
        .parent()
        .ok_or_else(|| RenderError::new("chapter source has no parent directory"))?;
    let resolved = canonical_existing(&chapter_parent.join(path_part), "local chapter link")?;
    ensure_inside(environment.project_root, &resolved, "local chapter link")?;
    let destination = environment.chapter_outputs.get(&resolved).ok_or_else(|| {
        RenderError::new(format!(
            "local Markdown link does not target a chapter in SUMMARY.md: {target}"
        ))
    })?;
    let output_parent = environment
        .output_path
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let mut linked_path = relative_markdown_path(output_parent, destination)?;
    if let Some(fragment) = fragment {
        linked_path.push('#');
        linked_path.push_str(fragment);
    }
    Ok(linked_path)
}

fn relative_markdown_path(
    from_directory: &Path,
    destination: &Path,
) -> Result<String, RenderError> {
    let from = from_directory.components().collect::<Vec<_>>();
    let to = destination.components().collect::<Vec<_>>();
    let common = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = PathBuf::new();
    for _ in common..from.len() {
        relative.push("..");
    }
    for component in &to[common..] {
        relative.push(component.as_os_str());
    }
    path_to_markdown(&relative)
}

fn path_to_markdown(path: &Path) -> Result<String, RenderError> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(value) => parts.push(
                value
                    .to_str()
                    .ok_or_else(|| RenderError::new("GitBook path is not valid UTF-8"))?
                    .to_owned(),
            ),
            std::path::Component::ParentDir => parts.push("..".to_owned()),
            _ => return Err(RenderError::new("GitBook path is not safely relative")),
        }
    }
    if parts.is_empty() {
        return Err(RenderError::new("GitBook path must not be empty"));
    }
    Ok(parts.join("/"))
}

fn validate_gitbook_relative_path(path: &Path) -> Result<(), RenderError> {
    path_to_markdown(path).map(|_| ())
}

fn prefix_markdown_lines(source: &str, prefix: &str) -> String {
    let mut output = String::new();
    for line in source.trim_end().lines() {
        if line.is_empty() {
            output.push_str(prefix.trim_end());
        } else {
            output.push_str(prefix);
            output.push_str(line);
        }
        output.push('\n');
    }
    output
}

fn markdown_fence(source: &str, info: &str) -> String {
    let marker = if info.contains('`') { '~' } else { '`' };
    let longest = source
        .split(|character| character != marker)
        .map(str::len)
        .max()
        .unwrap_or(0);
    marker.to_string().repeat(longest.saturating_add(1).max(3))
}

fn markdown_inline_code(value: &str) -> String {
    let longest = value
        .split(|character| character != '`')
        .map(str::len)
        .max()
        .unwrap_or(0);
    let marker = "`".repeat(longest + 1);
    if value.starts_with('`') || value.ends_with('`') {
        format!("{marker} {value} {marker}")
    } else {
        format!("{marker}{value}{marker}")
    }
}

fn escape_markdown_text(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if matches!(character, '\\' | '*' | '_' | '[' | ']' | '|' | '<' | '>') {
            output.push('\\');
        }
        output.push(character);
    }
    output
}
