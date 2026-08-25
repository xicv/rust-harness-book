fn parse_book_config(source: &str) -> Result<BookConfig, RenderError> {
    let mut section = String::new();
    let mut values = HashMap::new();
    for raw_line in source.lines() {
        let line = strip_toml_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_owned();
            continue;
        }
        if section != "book" {
            continue;
        }
        let Some((key, raw_value)) = line.split_once('=') else {
            return Err(RenderError::new(format!(
                "malformed book.toml line: {raw_line}"
            )));
        };
        values.insert(key.trim().to_owned(), raw_value.trim().to_owned());
    }

    let title = required_toml_string(&values, "title")?;
    let description = required_toml_string(&values, "description")?;
    let language = required_toml_string(&values, "language")?;
    let source_dir = required_toml_string(&values, "src")?;
    let authors_raw = values.get("authors").ok_or_else(|| {
        RenderError::new("book.toml [book] is missing authors")
    })?;
    let authors = parse_toml_string_array(authors_raw)?;
    if authors.is_empty() {
        return Err(RenderError::new("book.toml authors must not be empty"));
    }

    Ok(BookConfig {
        title,
        authors,
        description,
        language,
        source: PathBuf::from(source_dir),
    })
}

fn strip_toml_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;
    for (index, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && in_string {
            escaped = true;
            continue;
        }
        if character == '"' {
            in_string = !in_string;
            continue;
        }
        if character == '#' && !in_string {
            return &line[..index];
        }
    }
    line
}

fn required_toml_string(
    values: &HashMap<String, String>,
    key: &str,
) -> Result<String, RenderError> {
    let value = values.get(key).ok_or_else(|| {
        RenderError::new(format!("book.toml [book] is missing {key}"))
    })?;
    parse_toml_string(value)
}

fn parse_toml_string(value: &str) -> Result<String, RenderError> {
    if !value.starts_with('"') || !value.ends_with('"') || value.len() < 2 {
        return Err(RenderError::new(format!(
            "expected a basic TOML string, found {value:?}"
        )));
    }
    unescape_basic_string(&value[1..value.len() - 1])
}

fn parse_toml_string_array(value: &str) -> Result<Vec<String>, RenderError> {
    if !value.starts_with('[') || !value.ends_with(']') {
        return Err(RenderError::new(format!(
            "expected a TOML string array, found {value:?}"
        )));
    }
    let inner = &value[1..value.len() - 1];
    let mut output = Vec::new();
    let mut cursor = 0;
    while cursor < inner.len() {
        while cursor < inner.len()
            && inner[cursor..]
                .chars()
                .next()
                .is_some_and(|character| character.is_whitespace() || character == ',')
        {
            let character = inner[cursor..].chars().next().ok_or_else(|| {
                RenderError::new("invalid TOML string array")
            })?;
            cursor += character.len_utf8();
        }
        if cursor >= inner.len() {
            break;
        }
        if !inner[cursor..].starts_with('"') {
            return Err(RenderError::new(
                "book.toml authors must be an array of basic strings",
            ));
        }
        cursor += 1;
        let start = cursor;
        let mut escaped = false;
        let mut close = None;
        for (offset, character) in inner[cursor..].char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if character == '"' {
                close = Some(cursor + offset);
                break;
            }
        }
        let end = close.ok_or_else(|| {
            RenderError::new("unterminated TOML string in authors")
        })?;
        output.push(unescape_basic_string(&inner[start..end])?);
        cursor = end + 1;
    }
    Ok(output)
}

fn unescape_basic_string(value: &str) -> Result<String, RenderError> {
    let mut output = String::new();
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        let escaped = characters.next().ok_or_else(|| {
            RenderError::new("TOML string ends with an incomplete escape")
        })?;
        match escaped {
            '\\' => output.push('\\'),
            '"' => output.push('"'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            't' => output.push('\t'),
            other => {
                return Err(RenderError::new(format!(
                    "unsupported TOML string escape: \\{other}"
                )));
            }
        }
    }
    Ok(output)
}

fn parse_summary(
    source: &str,
    source_dir: &Path,
    project_root: &Path,
) -> Result<Vec<SummaryEntry>, RenderError> {
    let mut entries = Vec::new();
    let mut seen_paths = HashSet::new();
    let mut seen_part = false;

    for (line_index, raw_line) in source.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed == "# Summary" {
            continue;
        }
        if trimmed == "---" {
            entries.push(SummaryEntry::Separator);
            continue;
        }
        if let Some(title) = trimmed.strip_prefix("# ") {
            if title.trim().is_empty() {
                return Err(RenderError::new(format!(
                    "empty part title in SUMMARY.md line {}",
                    line_index + 1
                )));
            }
            seen_part = true;
            entries.push(SummaryEntry::Part(title.trim().to_owned()));
            continue;
        }
        let marker = parse_summary_link(raw_line).ok_or_else(|| {
            RenderError::new(format!(
                "unsupported SUMMARY.md structure at line {}: {raw_line}",
                line_index + 1
            ))
        })?;
        if is_remote_target(&marker.1) {
            return Err(RenderError::new(format!(
                "SUMMARY.md chapter target must be local: {}",
                marker.1
            )));
        }
        let target_without_fragment = marker.1.split('#').next().unwrap_or(&marker.1);
        let chapter_path = canonical_existing(
            &source_dir.join(target_without_fragment),
            "SUMMARY.md chapter",
        )?;
        ensure_inside(project_root, &chapter_path, "SUMMARY.md chapter")?;
        ensure_inside(source_dir, &chapter_path, "SUMMARY.md chapter")?;
        if !seen_paths.insert(chapter_path.clone()) {
            return Err(RenderError::new(format!(
                "SUMMARY.md contains a duplicate chapter target: {}",
                chapter_path.display()
            )));
        }
        let depth = if seen_part { marker.2 } else { 0 };
        entries.push(SummaryEntry::Chapter(SummaryChapter {
            title: marker.0,
            source: chapter_path,
            depth,
            label: String::new(),
        }));
    }

    if !entries
        .iter()
        .any(|entry| matches!(entry, SummaryEntry::Chapter(_)))
    {
        return Err(RenderError::new("SUMMARY.md contains no chapters"));
    }
    Ok(entries)
}

fn parse_summary_link(line: &str) -> Option<(String, String, usize)> {
    let indent = line.chars().take_while(|character| *character == ' ').count();
    let trimmed = line.trim_start();
    let body = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))?;
    if !body.starts_with('[') {
        return None;
    }
    let label_end = body.find("](")?;
    let close = body.rfind(')')?;
    if close + 1 != body.len() {
        return None;
    }
    let label = body[1..label_end].trim();
    let target = body[label_end + 2..close].trim().trim_matches('<').trim_matches('>');
    if label.is_empty() || target.is_empty() {
        return None;
    }
    Some((label.to_owned(), target.to_owned(), indent / 2))
}

fn assign_chapter_labels(entries: &mut [SummaryEntry]) {
    let mut index = 0;
    for entry in entries {
        if let SummaryEntry::Chapter(chapter) = entry {
            chapter.label = format!("chapter-{index:03}");
            index += 1;
        }
    }
}

fn chapter_label_map(entries: &[SummaryEntry]) -> HashMap<PathBuf, String> {
    entries
        .iter()
        .filter_map(|entry| match entry {
            SummaryEntry::Chapter(chapter) => {
                Some((chapter.source.clone(), chapter.label.clone()))
            }
            SummaryEntry::Part(_) | SummaryEntry::Separator => None,
        })
        .collect()
}
