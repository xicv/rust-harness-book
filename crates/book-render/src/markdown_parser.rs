impl MarkdownParser {
    fn new(source: &str) -> Self {
        Self {
            lines: source.lines().map(str::to_owned).collect(),
            index: 0,
        }
    }

    fn parse(mut self) -> Result<Vec<Block>, RenderError> {
        let blocks = self.parse_blocks(false)?;
        if self.index != self.lines.len() {
            return Err(RenderError::new("Markdown parser did not consume all input"));
        }
        Ok(blocks)
    }

    fn parse_blocks(&mut self, stop_at_card_end: bool) -> Result<Vec<Block>, RenderError> {
        let mut blocks = Vec::new();
        while self.index < self.lines.len() {
            let line = self.lines[self.index].clone();
            let trimmed = line.trim();

            if trimmed.is_empty() {
                self.index += 1;
                continue;
            }
            if trimmed == "</div>" {
                if stop_at_card_end {
                    self.index += 1;
                    return Ok(blocks);
                }
                return Err(RenderError::new("unexpected closing learning-card div"));
            }
            if trimmed == "<div class=\"learning-card\">" {
                self.index += 1;
                blocks.push(Block::Card(self.parse_blocks(true)?));
                continue;
            }
            if trimmed.starts_with("<div") || trimmed.starts_with("</div") {
                return Err(RenderError::new(format!(
                    "unsupported block HTML: {trimmed}"
                )));
            }
            if trimmed.starts_with("<p class=\"card-label\">") {
                blocks.push(self.parse_card_label()?);
                continue;
            }
            if let Some((level, heading)) = parse_heading(trimmed) {
                self.index += 1;
                blocks.push(Block::Heading {
                    level,
                    content: parse_inlines(heading)?,
                });
                continue;
            }
            if let Some((marker, info)) = parse_fence_start(trimmed) {
                blocks.push(self.parse_code_block(&marker, &info)?);
                continue;
            }
            if is_rule(trimmed) {
                self.index += 1;
                blocks.push(Block::Rule);
                continue;
            }
            if trimmed.starts_with('>') {
                blocks.push(self.parse_quote()?);
                continue;
            }
            if let Some(marker) = parse_list_marker(&line)? {
                blocks.push(self.parse_list(marker.indent, marker.kind)?);
                continue;
            }
            if self.is_table_start() {
                blocks.push(self.parse_table()?);
                continue;
            }
            if let Some(image) = parse_standalone_image(trimmed)? {
                self.index += 1;
                blocks.push(image);
                continue;
            }
            if looks_like_block_html(trimmed) {
                return Err(RenderError::new(format!(
                    "unsupported block HTML: {trimmed}"
                )));
            }
            blocks.push(self.parse_paragraph()?);
        }

        if stop_at_card_end {
            return Err(RenderError::new("learning-card div is not closed"));
        }
        Ok(blocks)
    }

    fn parse_card_label(&mut self) -> Result<Block, RenderError> {
        let start = "<p class=\"card-label\">";
        let end = "</p>";
        let mut value = self.lines[self.index].trim().to_owned();
        self.index += 1;
        while !value.contains(end) {
            if self.index >= self.lines.len() {
                return Err(RenderError::new("card-label paragraph is not closed"));
            }
            value.push(' ');
            value.push_str(self.lines[self.index].trim());
            self.index += 1;
        }
        if !value.starts_with(start) {
            return Err(RenderError::new("malformed card-label paragraph"));
        }
        let end_index = value.find(end).ok_or_else(|| {
            RenderError::new("card-label paragraph is not closed")
        })?;
        if !value[end_index + end.len()..].trim().is_empty() {
            return Err(RenderError::new(
                "content after closing card-label paragraph is unsupported",
            ));
        }
        let inner = &value[start.len()..end_index];
        Ok(Block::CardLabel(parse_inlines(inner)?))
    }

    fn parse_code_block(
        &mut self,
        marker: &str,
        info: &str,
    ) -> Result<Block, RenderError> {
        self.index += 1;
        let mut lines = Vec::new();
        while self.index < self.lines.len() {
            let line = &self.lines[self.index];
            if is_fence_end(line.trim(), marker) {
                self.index += 1;
                let language = info
                    .split(',')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_matches('{')
                    .trim_matches('}')
                    .to_owned();
                return Ok(Block::Code {
                    language,
                    source: lines.join("\n"),
                });
            }
            lines.push(line.clone());
            self.index += 1;
        }
        Err(RenderError::new("fenced code block is not closed"))
    }

    fn parse_quote(&mut self) -> Result<Block, RenderError> {
        let mut lines = Vec::new();
        while self.index < self.lines.len() {
            let trimmed = self.lines[self.index].trim_start();
            if !trimmed.starts_with('>') {
                break;
            }
            let rest = trimmed[1..].strip_prefix(' ').unwrap_or(&trimmed[1..]);
            lines.push(rest.to_owned());
            self.index += 1;
        }
        let nested = MarkdownParser::new(&lines.join("\n")).parse()?;
        Ok(Block::Quote(nested))
    }

    fn parse_list(
        &mut self,
        base_indent: usize,
        kind: ListKind,
    ) -> Result<Block, RenderError> {
        let mut items = Vec::new();
        let mut ordered_start = 1;
        let mut first = true;

        while self.index < self.lines.len() {
            let Some(marker) = parse_list_marker(&self.lines[self.index])? else {
                break;
            };
            if marker.indent != base_indent || marker.kind != kind {
                break;
            }
            if first {
                ordered_start = marker.start;
                first = false;
            }
            self.index += 1;
            let mut item_blocks = Vec::new();
            if !marker.content.trim().is_empty() {
                item_blocks.push(Block::Paragraph(parse_inlines(
                    marker.content.trim(),
                )?));
            }

            while self.index < self.lines.len() {
                let next_line = self.lines[self.index].clone();
                if next_line.trim().is_empty() {
                    self.index += 1;
                    continue;
                }
                let nested_marker = parse_list_marker(&next_line)?;
                if let Some(nested) = nested_marker {
                    if nested.indent > base_indent {
                        item_blocks.push(self.parse_list(nested.indent, nested.kind)?);
                        continue;
                    }
                    break;
                }
                let indent = leading_spaces(&next_line)?;
                if indent > base_indent {
                    self.index += 1;
                    item_blocks.push(Block::Paragraph(parse_inlines(
                        next_line.trim(),
                    )?));
                    continue;
                }
                break;
            }

            if item_blocks.is_empty() {
                item_blocks.push(Block::Paragraph(Vec::new()));
            }
            items.push(ListItem {
                blocks: item_blocks,
            });
        }

        match kind {
            ListKind::Bullet => Ok(Block::BulletList(items)),
            ListKind::Ordered => Ok(Block::OrderedList {
                start: ordered_start,
                items,
            }),
        }
    }

    fn is_table_start(&self) -> bool {
        if self.index + 1 >= self.lines.len() {
            return false;
        }
        self.lines[self.index].contains('|')
            && is_table_delimiter(self.lines[self.index + 1].trim())
    }

    fn parse_table(&mut self) -> Result<Block, RenderError> {
        let header_source = self.lines[self.index].clone();
        let delimiter_source = self.lines[self.index + 1].clone();
        let header_cells = split_table_cells(&header_source)?;
        let delimiter_cells = split_table_cells(&delimiter_source)?;
        if header_cells.is_empty() || header_cells.len() != delimiter_cells.len() {
            return Err(RenderError::new(
                "table header and delimiter row must have the same columns",
            ));
        }
        self.index += 2;
        let header = header_cells
            .iter()
            .map(|cell| parse_inlines(cell.trim()))
            .collect::<Result<Vec<_>, _>>()?;
        let mut rows = Vec::new();
        while self.index < self.lines.len() {
            let line = self.lines[self.index].clone();
            if line.trim().is_empty() || !line.contains('|') {
                break;
            }
            let cells = split_table_cells(&line)?;
            if cells.len() != header.len() {
                return Err(RenderError::new(
                    "table row has a different column count from its header",
                ));
            }
            rows.push(
                cells
                    .iter()
                    .map(|cell| parse_inlines(cell.trim()))
                    .collect::<Result<Vec<_>, _>>()?,
            );
            self.index += 1;
        }
        Ok(Block::Table { header, rows })
    }

    fn parse_paragraph(&mut self) -> Result<Block, RenderError> {
        let mut value = String::new();
        while self.index < self.lines.len() {
            let line = self.lines[self.index].clone();
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            if !value.is_empty() && self.is_block_start()? {
                break;
            }
            if !value.is_empty() {
                if value.ends_with("  ") {
                    value.truncate(value.len() - 2);
                    value.push_str("<br>");
                } else {
                    value.push(' ');
                }
            }
            value.push_str(trimmed);
            self.index += 1;
        }
        Ok(Block::Paragraph(parse_inlines(&value)?))
    }

    fn is_block_start(&self) -> Result<bool, RenderError> {
        if self.index >= self.lines.len() {
            return Ok(false);
        }
        let line = &self.lines[self.index];
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed == "<div class=\"learning-card\">"
            || trimmed == "</div>"
            || trimmed.starts_with("<p class=\"card-label\">")
            || parse_heading(trimmed).is_some()
            || parse_fence_start(trimmed).is_some()
            || is_rule(trimmed)
            || trimmed.starts_with('>')
            || parse_standalone_image(trimmed)?.is_some()
            || self.is_table_start()
        {
            return Ok(true);
        }
        Ok(parse_list_marker(line)?.is_some())
    }
}
