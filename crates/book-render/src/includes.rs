impl<'a> IncludeExpander<'a> {
    fn expand_file(&mut self, path: &Path) -> Result<String, RenderError> {
        let canonical = canonical_existing(path, "Markdown/include source")?;
        ensure_inside(self.project_root, &canonical, "Markdown/include source")?;
        if self.stack.contains(&canonical) {
            let chain = self
                .stack
                .iter()
                .chain(std::iter::once(&canonical))
                .map(|value| value.display().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(RenderError::new(format!(
                "recursive include detected: {chain}"
            )));
        }
        if self.stack.len() >= MAX_INCLUDE_DEPTH {
            return Err(RenderError::new(format!(
                "include depth exceeds {MAX_INCLUDE_DEPTH}"
            )));
        }
        self.stack.push(canonical.clone());
        let source = read_utf8(&canonical, "Markdown/include source");
        let expanded = match source {
            Ok(value) => {
                let parent = canonical.parent().ok_or_else(|| {
                    RenderError::new("included source has no parent directory")
                })?;
                self.expand_text(&value, parent)
            }
            Err(error) => Err(error),
        };
        let _ = self.stack.pop();
        expanded
    }

    fn expand_text(&mut self, source: &str, base: &Path) -> Result<String, RenderError> {
        let mut output = String::new();
        let mut cursor = 0;
        while let Some(start_offset) = source[cursor..].find("{{#") {
            let start = cursor + start_offset;
            output.push_str(&source[cursor..start]);
            let directive_start = start + "{{#".len();
            let Some(end_offset) = source[directive_start..].find("}}") else {
                return Err(RenderError::new("mdBook include directive is not closed"));
            };
            let end = directive_start + end_offset;
            let directive = source[directive_start..end].trim();
            let (command, argument) = directive.split_once(char::is_whitespace).ok_or_else(|| {
                RenderError::new(format!("malformed mdBook directive: {directive}"))
            })?;
            if command != "include" && command != "rustdoc_include" {
                return Err(RenderError::new(format!(
                    "unsupported mdBook directive: {command}"
                )));
            }
            let (include_path, selector) = resolve_include_argument(base, argument.trim())?;
            let canonical = canonical_existing(&include_path, "included source")?;
            ensure_inside(self.project_root, &canonical, "included source")?;
            if self.stack.contains(&canonical) {
                let chain = self
                    .stack
                    .iter()
                    .chain(std::iter::once(&canonical))
                    .map(|value| value.display().to_string())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                return Err(RenderError::new(format!(
                    "recursive include detected: {chain}"
                )));
            }
            if self.stack.len() >= MAX_INCLUDE_DEPTH {
                return Err(RenderError::new(format!(
                    "include depth exceeds {MAX_INCLUDE_DEPTH}"
                )));
            }
            let included = read_utf8(&canonical, "included source")?;
            let selected = select_include(&included, selector.as_deref())?;
            self.stack.push(canonical.clone());
            let parent = canonical.parent().ok_or_else(|| {
                RenderError::new("included source has no parent directory")
            })?;
            let recursively_expanded = self.expand_text(&selected, parent);
            let _ = self.stack.pop();
            output.push_str(&recursively_expanded?);
            cursor = end + "}}".len();
        }
        output.push_str(&source[cursor..]);
        Ok(output)
    }
}

fn resolve_include_argument(
    base: &Path,
    argument: &str,
) -> Result<(PathBuf, Option<String>), RenderError> {
    if argument.is_empty() {
        return Err(RenderError::new("mdBook include path is empty"));
    }
    let direct = base.join(argument);
    if direct.is_file() {
        return Ok((direct, None));
    }

    let mut colon_indices = argument
        .match_indices(':')
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    colon_indices.reverse();
    for index in colon_indices {
        let path_part = &argument[..index];
        let selector = &argument[index + 1..];
        let candidate = base.join(path_part);
        if candidate.is_file() {
            return Ok((candidate, Some(selector.to_owned())));
        }
    }

    Err(RenderError::new(format!(
        "included source does not exist relative to {}: {argument}",
        base.display()
    )))
}

fn select_include(source: &str, selector: Option<&str>) -> Result<String, RenderError> {
    let Some(selector) = selector else {
        return Ok(source.to_owned());
    };
    if selector.is_empty() {
        return Ok(source.to_owned());
    }
    if selector
        .chars()
        .all(|character| character.is_ascii_digit() || character == ':')
    {
        return select_line_range(source, selector);
    }
    select_anchor(source, selector)
}

fn select_line_range(source: &str, selector: &str) -> Result<String, RenderError> {
    let parts = selector.split(':').collect::<Vec<_>>();
    let lines = source.lines().collect::<Vec<_>>();
    match parts.as_slice() {
        [line] => {
            let number = parse_positive_line_number(line)?;
            let value = lines.get(number - 1).ok_or_else(|| {
                RenderError::new(format!("include line {number} is outside the source"))
            })?;
            Ok((*value).to_owned())
        }
        [start, end] => {
            let start_number = if start.is_empty() {
                1
            } else {
                parse_positive_line_number(start)?
            };
            let end_number = if end.is_empty() {
                lines.len()
            } else {
                parse_positive_line_number(end)?
            };
            if start_number > end_number {
                return Ok(String::new());
            }
            let start_index = start_number.saturating_sub(1).min(lines.len());
            let end_index = end_number.min(lines.len());
            Ok(lines[start_index..end_index].join("\n"))
        }
        _ => Err(RenderError::new(format!(
            "invalid mdBook line-range selector: {selector}"
        ))),
    }
}

fn parse_positive_line_number(value: &str) -> Result<usize, RenderError> {
    let number = value.parse::<usize>().map_err(|error| {
        RenderError::new(format!("invalid include line number {value:?}: {error}"))
    })?;
    if number == 0 {
        return Err(RenderError::new("include line numbers are one-based"));
    }
    Ok(number)
}

fn select_anchor(source: &str, selector: &str) -> Result<String, RenderError> {
    let mut output = Vec::new();
    let mut active = false;
    let mut starts = 0;
    let mut ends = 0;

    for line in source.lines() {
        if let Some((is_end, name)) = anchor_marker(line) {
            if name == selector {
                if is_end {
                    if !active {
                        return Err(RenderError::new(format!(
                            "anchor {selector:?} ends before it starts"
                        )));
                    }
                    active = false;
                    ends += 1;
                } else {
                    if active || starts > 0 {
                        return Err(RenderError::new(format!(
                            "anchor {selector:?} is duplicated"
                        )));
                    }
                    active = true;
                    starts += 1;
                }
            }
            continue;
        }
        if active {
            output.push(line);
        }
    }

    if starts == 0 {
        return Err(RenderError::new(format!(
            "anchor {selector:?} was not found"
        )));
    }
    if active || ends != 1 {
        return Err(RenderError::new(format!(
            "anchor {selector:?} is not closed"
        )));
    }
    Ok(output.join("\n"))
}

fn anchor_marker(line: &str) -> Option<(bool, &str)> {
    if let Some(index) = line.find("ANCHOR_END:") {
        let value = line[index + "ANCHOR_END:".len()..].trim();
        return valid_anchor_name(value).then_some((true, value));
    }
    if let Some(index) = line.find("ANCHOR:") {
        let value = line[index + "ANCHOR:".len()..].trim();
        return valid_anchor_name(value).then_some((false, value));
    }
    None
}

fn valid_anchor_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

impl AssetRegistry {
    fn new(project_root: &Path, output_dir: &Path) -> Self {
        Self {
            project_root: project_root.to_owned(),
            output_dir: output_dir.to_owned(),
            by_source: HashMap::new(),
        }
    }

    fn register(&mut self, source: &Path) -> Result<String, RenderError> {
        let canonical = canonical_existing(source, "local image")?;
        ensure_inside(&self.project_root, &canonical, "local image")?;
        if let Some(existing) = self.by_source.get(&canonical) {
            return Ok(existing.clone());
        }
        let extension = canonical
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .ok_or_else(|| RenderError::new("local image has no file extension"))?;
        if !matches!(
            extension.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "pdf"
        ) {
            return Err(RenderError::new(format!(
                "unsupported local image format: {extension}"
            )));
        }
        let name = format!("asset-{:04}.{extension}", self.by_source.len());
        let destination = self.output_dir.join(&name);
        fs::copy(&canonical, &destination).map_err(|error| {
            io_context("copy local image", &destination, error)
        })?;
        let published = format!("assets/{name}");
        self.by_source.insert(canonical, published.clone());
        Ok(published)
    }

    fn len(&self) -> usize {
        self.by_source.len()
    }
}
