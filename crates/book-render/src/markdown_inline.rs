fn parse_inlines(source: &str) -> Result<Vec<Inline>, RenderError> {
    let mut output = Vec::new();
    let mut text = String::new();
    let mut index = 0;

    while index < source.len() {
        let rest = &source[index..];
        if rest.starts_with("<span class=\"term-en\">") {
            flush_inline_text(&mut output, &mut text);
            let start = "<span class=\"term-en\">".len();
            let close = rest.find("</span>").ok_or_else(|| {
                RenderError::new("term-en span is not closed")
            })?;
            let inner = &rest[start..close];
            output.push(Inline::Term(parse_inlines(inner)?));
            index += close + "</span>".len();
            continue;
        }
        if rest.starts_with("<br>") {
            flush_inline_text(&mut output, &mut text);
            output.push(Inline::Break);
            index += "<br>".len();
            continue;
        }
        if rest.starts_with("<br/>") {
            flush_inline_text(&mut output, &mut text);
            output.push(Inline::Break);
            index += "<br/>".len();
            continue;
        }
        if rest.starts_with("<br />") {
            flush_inline_text(&mut output, &mut text);
            output.push(Inline::Break);
            index += "<br />".len();
            continue;
        }
        if rest.starts_with("<http://") || rest.starts_with("<https://") {
            flush_inline_text(&mut output, &mut text);
            let close = rest.find('>').ok_or_else(|| {
                RenderError::new("autolink is not closed")
            })?;
            let target = rest[1..close].to_owned();
            output.push(Inline::Link {
                content: vec![Inline::Text(target.clone())],
                target,
            });
            index += close + 1;
            continue;
        }
        if let Some(stripped) = rest.strip_prefix("**") {
            flush_inline_text(&mut output, &mut text);
            let close = stripped.find("**").ok_or_else(|| {
                RenderError::new("strong emphasis is not closed")
            })?;
            output.push(Inline::Strong(parse_inlines(&stripped[..close])?));
            index += close + 4;
            continue;
        }
        if let Some(stripped) = rest.strip_prefix("~~") {
            flush_inline_text(&mut output, &mut text);
            let close = stripped.find("~~").ok_or_else(|| {
                RenderError::new("strikethrough is not closed")
            })?;
            output.push(Inline::Strike(parse_inlines(&stripped[..close])?));
            index += close + 4;
            continue;
        }
        if rest.starts_with('`') {
            flush_inline_text(&mut output, &mut text);
            let tick_count = rest.chars().take_while(|value| *value == '`').count();
            let marker = "`".repeat(tick_count);
            let close = rest[tick_count..].find(&marker).ok_or_else(|| {
                RenderError::new("inline code span is not closed")
            })? + tick_count;
            output.push(Inline::Code(rest[tick_count..close].to_owned()));
            index += close + tick_count;
            continue;
        }
        if rest.starts_with("![") {
            return Err(RenderError::new(
                "inline images are unsupported; place the image on its own line",
            ));
        }
        if let Some(stripped) = rest.strip_prefix('[')
            && let Some(label_end) = stripped.find("](")
        {
            let destination_start = label_end + 2;
            let destination_end = stripped[destination_start..]
                .find(')')
                .map(|offset| destination_start + offset)
                .ok_or_else(|| RenderError::new("Markdown link is not closed"))?;
            flush_inline_text(&mut output, &mut text);
            let label = &stripped[..label_end];
            let raw_destination = &stripped[destination_start..destination_end];
            let (target, _) = parse_destination_and_title(raw_destination)?;
            output.push(Inline::Link {
                target,
                content: parse_inlines(label)?,
            });
            index += destination_end + 2;
            continue;
        }
        if let Some(stripped) = rest.strip_prefix('*')
            && let Some(close) = stripped.find('*')
        {
            flush_inline_text(&mut output, &mut text);
            output.push(Inline::Emphasis(parse_inlines(&stripped[..close])?));
            index += close + 2;
            continue;
        }
        if rest.starts_with('\\') {
            let mut characters = rest.chars();
            let _ = characters.next();
            if let Some(character) = characters.next() {
                if character.is_ascii_punctuation() {
                    text.push(character);
                    index += 1 + character.len_utf8();
                } else {
                    text.push('\\');
                    index += 1;
                }
            } else {
                text.push('\\');
                index += 1;
            }
            continue;
        }
        if rest.starts_with('<') && looks_like_inline_html(rest) {
            return Err(RenderError::new(format!(
                "unsupported inline HTML near: {}",
                rest.chars().take(40).collect::<String>()
            )));
        }

        let Some(character) = rest.chars().next() else {
            break;
        };
        text.push(character);
        index += character.len_utf8();
    }

    flush_inline_text(&mut output, &mut text);
    Ok(output)
}

fn flush_inline_text(output: &mut Vec<Inline>, text: &mut String) {
    if text.is_empty() {
        return;
    }
    let value = std::mem::take(text);
    if let Some(Inline::Text(previous)) = output.last_mut() {
        previous.push_str(&value);
    } else {
        output.push(Inline::Text(value));
    }
}

fn inline_plain_text(content: &[Inline]) -> String {
    let mut output = String::new();
    for inline in content {
        match inline {
            Inline::Text(value) | Inline::Code(value) => output.push_str(value),
            Inline::Strong(children)
            | Inline::Emphasis(children)
            | Inline::Strike(children)
            | Inline::Term(children) => output.push_str(&inline_plain_text(children)),
            Inline::Link { content, .. } => output.push_str(&inline_plain_text(content)),
            Inline::Break => output.push(' '),
        }
    }
    output
}

fn looks_like_inline_html(value: &str) -> bool {
    let mut characters = value.chars();
    let _ = characters.next();
    matches!(characters.next(), Some('/') | Some('a'..='z') | Some('A'..='Z'))
}

fn looks_like_block_html(value: &str) -> bool {
    value.starts_with('<') && looks_like_inline_html(value)
}

fn strip_html_comments(source: &str) -> Result<String, RenderError> {
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    while let Some(start_offset) = source[cursor..].find("<!--") {
        let start = cursor + start_offset;
        output.push_str(&source[cursor..start]);
        let comment_start = start + "<!--".len();
        let Some(end_offset) = source[comment_start..].find("-->") else {
            return Err(RenderError::new("HTML comment is not closed"));
        };
        let end = comment_start + end_offset + "-->".len();
        let removed_newlines = source[start..end]
            .chars()
            .filter(|character| *character == '\n')
            .count();
        for _ in 0..removed_newlines {
            output.push('\n');
        }
        cursor = end;
    }
    output.push_str(&source[cursor..]);
    Ok(output)
}
