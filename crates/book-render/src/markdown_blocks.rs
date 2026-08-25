fn parse_heading(line: &str) -> Option<(usize, &str)> {
    let count = line.chars().take_while(|character| *character == '#').count();
    if count == 0 || count > 6 {
        return None;
    }
    let rest = line.get(count..)?;
    rest.strip_prefix(' ').map(|value| (count, value.trim()))
}

fn parse_fence_start(line: &str) -> Option<(String, String)> {
    for marker_char in ['`', '~'] {
        let count = line.chars().take_while(|value| *value == marker_char).count();
        if count >= 3 {
            let marker = marker_char.to_string().repeat(count);
            let info = line.get(count..)?.trim().to_owned();
            return Some((marker, info));
        }
    }
    None
}

fn is_fence_end(line: &str, marker: &str) -> bool {
    line.starts_with(marker) && line[marker.len()..].trim().is_empty()
}

fn is_rule(line: &str) -> bool {
    let compact: String = line.chars().filter(|character| !character.is_whitespace()).collect();
    (compact.len() >= 3 && compact.chars().all(|character| character == '-'))
        || (compact.len() >= 3 && compact.chars().all(|character| character == '*'))
}

fn parse_list_marker(line: &str) -> Result<Option<ListMarker>, RenderError> {
    let indent = leading_spaces(line)?;
    let trimmed = &line[indent..];
    if let Some(content) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        return Ok(Some(ListMarker {
            indent,
            kind: ListKind::Bullet,
            start: 1,
            content: content.to_owned(),
        }));
    }

    let digit_count = trimmed
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .count();
    if digit_count == 0 {
        return Ok(None);
    }
    let Some(rest) = trimmed.get(digit_count..) else {
        return Ok(None);
    };
    let Some(content) = rest.strip_prefix(". ") else {
        return Ok(None);
    };
    let start = trimmed[..digit_count].parse::<u64>().map_err(|error| {
        RenderError::new(format!("invalid ordered-list number: {error}"))
    })?;
    Ok(Some(ListMarker {
        indent,
        kind: ListKind::Ordered,
        start,
        content: content.to_owned(),
    }))
}

fn leading_spaces(line: &str) -> Result<usize, RenderError> {
    let mut count = 0;
    for character in line.chars() {
        match character {
            ' ' => count += 1,
            '\t' => {
                return Err(RenderError::new(
                    "tabs are not supported for Markdown indentation",
                ));
            }
            _ => break,
        }
    }
    Ok(count)
}

fn is_table_delimiter(line: &str) -> bool {
    let Ok(cells) = split_table_cells(line) else {
        return false;
    };
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let value = cell.trim().trim_start_matches(':').trim_end_matches(':');
            value.len() >= 3 && value.chars().all(|character| character == '-')
        })
}

fn split_table_cells(line: &str) -> Result<Vec<String>, RenderError> {
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for character in line.trim().chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character == '|' {
            cells.push(current.trim().to_owned());
            current.clear();
        } else {
            current.push(character);
        }
    }
    if escaped {
        return Err(RenderError::new("table row ends with an incomplete escape"));
    }
    cells.push(current.trim().to_owned());
    if cells.first().is_some_and(String::is_empty) {
        cells.remove(0);
    }
    if cells.last().is_some_and(String::is_empty) {
        cells.pop();
    }
    Ok(cells)
}

fn parse_standalone_image(line: &str) -> Result<Option<Block>, RenderError> {
    if !line.starts_with("![") {
        return Ok(None);
    }
    let alt_end = line.find("](").ok_or_else(|| {
        RenderError::new("malformed Markdown image")
    })?;
    let close = line.rfind(')').ok_or_else(|| {
        RenderError::new("malformed Markdown image")
    })?;
    if close + 1 != line.len() {
        return Err(RenderError::new(
            "standalone Markdown image has trailing content",
        ));
    }
    let alt = line[2..alt_end].to_owned();
    let raw_destination = line[alt_end + 2..close].trim();
    let (path, caption) = parse_destination_and_title(raw_destination)?;
    Ok(Some(Block::Image { alt, path, caption }))
}

fn parse_destination_and_title(
    raw: &str,
) -> Result<(String, Option<String>), RenderError> {
    if raw.starts_with('<') && raw.ends_with('>') {
        return Ok((raw[1..raw.len() - 1].to_owned(), None));
    }
    if let Some(index) = raw.find(" \"") {
        if !raw.ends_with('"') {
            return Err(RenderError::new("malformed Markdown link title"));
        }
        let path = raw[..index].trim().to_owned();
        let title = raw[index + 2..raw.len() - 1].to_owned();
        return Ok((path, Some(title)));
    }
    Ok((raw.to_owned(), None))
}
