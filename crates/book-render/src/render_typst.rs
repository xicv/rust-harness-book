fn render_into_staging(
    config: &BookConfig,
    entries: &[SummaryEntry],
    chapter_labels: &HashMap<PathBuf, String>,
    project_root: &Path,
    _template_path: &Path,
    staging_dir: &Path,
) -> Result<(String, usize), RenderError> {
    let (language, region) = split_language_region(&config.language)?;
    let mut generated = String::new();
    generated.push_str("#import \"template.typ\": *\n\n");
    generated.push_str("#show: book.with(\n");
    writeln!(
        generated,
        "  title: [#text(\"{}\")],",
        escape_typst_string(&config.title)
    )
    .map_err(format_error)?;
    writeln!(
        generated,
        "  subtitle: [#text(\"{}\")],",
        escape_typst_string(&config.description)
    )
    .map_err(format_error)?;
    writeln!(
        generated,
        "  authors: [#text(\"{}\")],",
        escape_typst_string(&config.authors.join(", "))
    )
    .map_err(format_error)?;
    writeln!(
        generated,
        "  language: \"{}\",",
        escape_typst_string(language)
    )
    .map_err(format_error)?;
    match region {
        Some(region) => writeln!(
            generated,
            "  region: \"{}\",",
            escape_typst_string(region)
        )
        .map_err(format_error)?,
        None => generated.push_str("  region: none,\n"),
    }
    generated.push_str(")\n\n");

    let mut assets = AssetRegistry::new(project_root, &staging_dir.join("assets"));
    let mut expander = IncludeExpander {
        project_root,
        stack: Vec::new(),
    };

    for entry in entries {
        match entry {
            SummaryEntry::Part(title) => {
                writeln!(
                    generated,
                    "#part([#text(\"{}\")])\n",
                    escape_typst_string(title)
                )
                .map_err(format_error)?;
            }
            SummaryEntry::Separator => {
                generated.push_str("#thematic-break()\n\n");
            }
            SummaryEntry::Chapter(chapter) => {
                let source = expander.expand_file(&chapter.source)?;
                let cleaned = strip_html_comments(&source)?;
                let mut blocks = MarkdownParser::new(&cleaned).parse()?;
                remove_matching_title(&mut blocks, chapter)?;

                writeln!(
                    generated,
                    "#chapter([#text(\"{}\")], level: {}) <{}>",
                    escape_typst_string(&chapter.title),
                    2 + chapter.depth,
                    chapter.label
                )
                .map_err(format_error)?;

                let mut environment = RenderEnvironment {
                    chapter_path: Some(&chapter.source),
                    project_root: Some(project_root),
                    chapter_labels,
                    assets: Some(&mut assets),
                    heading_index: 0,
                };
                render_blocks(&blocks, &mut environment, &mut generated)?;
                generated.push('\n');
            }
        }
    }

    Ok((generated, assets.len()))
}

fn split_language_region(value: &str) -> Result<(&str, Option<&str>), RenderError> {
    let mut parts = value.split('-');
    let language = parts.next().filter(|part| {
        matches!(part.len(), 2 | 3) && part.chars().all(|character| character.is_ascii_alphabetic())
    });
    let Some(language) = language else {
        return Err(RenderError::new(format!(
            "book language must start with a two- or three-letter ISO code: {value:?}"
        )));
    };
    let region = parts.next();
    if parts.next().is_some() {
        return Err(RenderError::new(format!(
            "book language may contain at most one region suffix: {value:?}"
        )));
    }
    if let Some(region) = region
        && (region.len() != 2 || !region.chars().all(|character| character.is_ascii_alphabetic()))
    {
        return Err(RenderError::new(format!(
            "book region must be a two-letter ISO code: {region:?}"
        )));
    }
    Ok((language, region))
}

fn remove_matching_title(
    blocks: &mut Vec<Block>,
    chapter: &SummaryChapter,
) -> Result<(), RenderError> {
    let first_content = blocks.iter().position(|block| {
        !matches!(block, Block::Paragraph(content) if inline_plain_text(content).trim().is_empty())
    });
    let Some(index) = first_content else {
        return Err(RenderError::new(format!(
            "chapter is empty: {}",
            chapter.source.display()
        )));
    };

    let Block::Heading { level, content } = &blocks[index] else {
        return Err(RenderError::new(format!(
            "chapter must start with an H1 matching SUMMARY.md: {}",
            chapter.source.display()
        )));
    };
    if *level != 1 {
        return Err(RenderError::new(format!(
            "chapter must start with an H1 matching SUMMARY.md: {}",
            chapter.source.display()
        )));
    }
    let actual = normalize_title(&inline_plain_text(content));
    let expected = normalize_title(&chapter.title);
    let is_index = chapter
        .source
        .file_name()
        .is_some_and(|name| name == "index.md");
    if actual != expected && !is_index {
        return Err(RenderError::new(format!(
            "chapter H1 differs from SUMMARY.md for {}: {actual:?} != {expected:?}",
            chapter.source.display()
        )));
    }
    blocks.remove(index);
    Ok(())
}

fn normalize_title(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn render_blocks(
    blocks: &[Block],
    environment: &mut RenderEnvironment<'_>,
    output: &mut String,
) -> Result<(), RenderError> {
    for block in blocks {
        match block {
            Block::Heading { level, content } => {
                environment.heading_index += 1;
                let rendered = render_inlines(content, environment)?;
                writeln!(
                    output,
                    "#section([{}], level: {})",
                    rendered,
                    level + 1
                )
                .map_err(format_error)?;
            }
            Block::Paragraph(content) => {
                let rendered = render_inlines(content, environment)?;
                writeln!(output, "#prose([{}])", rendered).map_err(format_error)?;
            }
            Block::Code { language, source } => {
                writeln!(
                    output,
                    "#code-block(\"{}\", \"{}\")",
                    escape_typst_string(language),
                    escape_typst_string(source)
                )
                .map_err(format_error)?;
            }
            Block::BulletList(items) => {
                render_list("bullet-list", None, items, environment, output)?;
            }
            Block::OrderedList { start, items } => {
                render_list("ordered-list", Some(*start), items, environment, output)?;
            }
            Block::Quote(children) => {
                output.push_str("#quote-block([\n");
                render_blocks(children, environment, output)?;
                output.push_str("])\n");
            }
            Block::Card(children) => {
                output.push_str("#card([\n");
                render_blocks(children, environment, output)?;
                output.push_str("])\n");
            }
            Block::CardLabel(content) => {
                let rendered = render_inlines(content, environment)?;
                writeln!(output, "#card-label([{}])", rendered).map_err(format_error)?;
            }
            Block::Table { header, rows } => {
                let columns = header.len();
                if columns == 0 {
                    return Err(RenderError::new("table must have at least one column"));
                }
                write!(output, "#book-table({}, (", columns).map_err(format_error)?;
                for cell in header {
                    let rendered = render_inlines(cell, environment)?;
                    write!(output, "[#strong([{}])],", rendered).map_err(format_error)?;
                }
                for row in rows {
                    if row.len() != columns {
                        return Err(RenderError::new(
                            "table row has a different column count from its header",
                        ));
                    }
                    for cell in row {
                        let rendered = render_inlines(cell, environment)?;
                        write!(output, "[{}],", rendered).map_err(format_error)?;
                    }
                }
                output.push_str("))\n");
            }
            Block::Image { alt, path, caption } => {
                if is_remote_target(path) {
                    return Err(RenderError::new(format!(
                        "remote image must be vendored: {path}"
                    )));
                }
                if alt.trim().is_empty() {
                    return Err(RenderError::new("local image must have non-empty alt text"));
                }
                let chapter_path = environment.chapter_path.ok_or_else(|| {
                    RenderError::new("images require a chapter source path")
                })?;
                let chapter_parent = chapter_path.parent().ok_or_else(|| {
                    RenderError::new("chapter source has no parent directory")
                })?;
                let source_path = chapter_parent.join(path);
                let registry = environment.assets.as_mut().ok_or_else(|| {
                    RenderError::new("images require the complete book renderer")
                })?;
                let published_path = registry.register(&source_path)?;
                let caption_value = caption.as_deref().unwrap_or(alt);
                writeln!(
                    output,
                    "#book-image(\"{}\", \"{}\", caption: [#text(\"{}\")])",
                    escape_typst_string(&published_path),
                    escape_typst_string(alt),
                    escape_typst_string(caption_value)
                )
                .map_err(format_error)?;
            }
            Block::Rule => output.push_str("#thematic-break()\n"),
        }
    }
    Ok(())
}

fn render_list(
    function: &str,
    start: Option<u64>,
    items: &[ListItem],
    environment: &mut RenderEnvironment<'_>,
    output: &mut String,
) -> Result<(), RenderError> {
    match start {
        Some(value) => write!(output, "#{function}({value}, (").map_err(format_error)?,
        None => write!(output, "#{function}((").map_err(format_error)?,
    }
    for item in items {
        output.push_str("[\n");
        render_blocks(&item.blocks, environment, output)?;
        output.push_str("],");
    }
    output.push_str("))\n");
    Ok(())
}

fn render_inlines(
    content: &[Inline],
    environment: &RenderEnvironment<'_>,
) -> Result<String, RenderError> {
    let mut output = String::new();
    for inline in content {
        match inline {
            Inline::Text(value) => {
                write!(output, "#text(\"{}\")", escape_typst_string(value))
                    .map_err(format_error)?;
            }
            Inline::Strong(children) => {
                write!(
                    output,
                    "#strong([{}])",
                    render_inlines(children, environment)?
                )
                .map_err(format_error)?;
            }
            Inline::Emphasis(children) => {
                write!(
                    output,
                    "#emph([{}])",
                    render_inlines(children, environment)?
                )
                .map_err(format_error)?;
            }
            Inline::Strike(children) => {
                write!(
                    output,
                    "#strike-text([{}])",
                    render_inlines(children, environment)?
                )
                .map_err(format_error)?;
            }
            Inline::Code(value) => {
                write!(output, "#inline-code(\"{}\")", escape_typst_string(value))
                    .map_err(format_error)?;
            }
            Inline::Term(children) => {
                write!(
                    output,
                    "#term([{}])",
                    render_inlines(children, environment)?
                )
                .map_err(format_error)?;
            }
            Inline::Link { target, content } => {
                let body = render_inlines(content, environment)?;
                match resolve_link(target, environment)? {
                    ResolvedLink::External(destination) => {
                        write!(
                            output,
                            "#book-link(\"{}\", [{}])",
                            escape_typst_string(&destination),
                            body
                        )
                        .map_err(format_error)?;
                    }
                    ResolvedLink::Internal(label) => {
                        write!(output, "#book-ref(<{}>, [{}])", label, body)
                            .map_err(format_error)?;
                    }
                }
            }
            Inline::Break => output.push_str("#linebreak()"),
        }
    }
    Ok(output)
}

fn resolve_link(
    target: &str,
    environment: &RenderEnvironment<'_>,
) -> Result<ResolvedLink, RenderError> {
    if is_remote_target(target) || target.starts_with("mailto:") {
        return Ok(ResolvedLink::External(target.to_owned()));
    }

    let Some(chapter_path) = environment.chapter_path else {
        return Ok(ResolvedLink::External(target.to_owned()));
    };
    let project_root = environment
        .project_root
        .ok_or_else(|| RenderError::new("local links require a project root"))?;

    if target.starts_with('#') {
        let current = canonical_existing(chapter_path, "current chapter")?;
        let label = environment.chapter_labels.get(&current).ok_or_else(|| {
            RenderError::new("current chapter is missing from SUMMARY.md")
        })?;
        return Ok(ResolvedLink::Internal(label.clone()));
    }

    let (path_part, fragment) = split_once_optional(target, '#');
    if fragment.is_some_and(|value| !value.is_empty()) {
        return Err(RenderError::new(format!(
            "heading-fragment links are not yet supported in PDF output: {target}"
        )));
    }
    let parent = chapter_path
        .parent()
        .ok_or_else(|| RenderError::new("chapter source has no parent directory"))?;
    let resolved = canonical_existing(&parent.join(path_part), "local chapter link")?;
    ensure_inside(project_root, &resolved, "local chapter link")?;
    let label = environment.chapter_labels.get(&resolved).ok_or_else(|| {
        RenderError::new(format!(
            "local Markdown link does not target a chapter in SUMMARY.md: {target}"
        ))
    })?;
    Ok(ResolvedLink::Internal(label.clone()))
}

fn split_once_optional(value: &str, delimiter: char) -> (&str, Option<&str>) {
    match value.split_once(delimiter) {
        Some((left, right)) => (left, Some(right)),
        None => (value, None),
    }
}
