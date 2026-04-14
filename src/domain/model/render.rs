use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub const SUPPORTED_RENDER_TYPES: [&str; 5] = [
    "heading",
    "paragraph",
    "bullet_list",
    "code_block",
    "citations",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderDocument {
    pub blocks: Vec<RenderBlock>,
}

impl RenderDocument {
    pub fn parse_assistant_response(response: &str) -> Option<Self> {
        let trimmed = response.trim();
        extract_json_payload(trimmed)
            .and_then(Self::parse_json_envelope)
            .or_else(|| salvage_partial_render_document(trimmed))
    }

    pub fn from_assistant_plain_text(input: &str) -> Self {
        parse_plain_text_render_document(input).unwrap_or_else(|| Self {
            blocks: vec![RenderBlock::Paragraph {
                text: input.trim().to_string(),
            }],
        })
    }

    pub fn canonicalize_assistant_response(response: &str) -> Self {
        Self::parse_assistant_response(response)
            .unwrap_or_else(|| Self::from_assistant_plain_text(response))
    }

    pub fn to_plain_text(&self) -> String {
        let mut rendered = String::new();
        let mut previous = None::<&RenderBlock>;

        for block in &self.blocks {
            let block_text = block.to_plain_text();
            if block_text.trim().is_empty() {
                continue;
            }

            if let Some(previous_block) = previous {
                rendered.push_str(block_separator(previous_block, block));
            }

            rendered.push_str(&block_text);
            previous = Some(block);
        }

        rendered
    }

    fn parse_json_envelope(json: &str) -> Option<Self> {
        let envelope: RenderEnvelope = serde_json::from_str(json).ok()?;
        Self::from_envelope(envelope)
    }

    fn from_envelope(envelope: RenderEnvelope) -> Option<Self> {
        let blocks: Vec<RenderBlock> = envelope
            .blocks
            .into_iter()
            .filter_map(RenderBlockEnvelope::into_block)
            .collect();
        if blocks.is_empty() {
            return None;
        }

        let repaired = render_types_for_blocks(&blocks);
        let declared = envelope
            .render_types
            .iter()
            .map(|value| RenderType::from_str(value))
            .collect::<Option<Vec<_>>>()
            .filter(|render_types| !render_types.is_empty())
            .and_then(unique_render_types);

        let _render_types = match declared {
            Some(render_types)
                if render_types.iter().copied().collect::<BTreeSet<_>>()
                    == repaired.iter().copied().collect::<BTreeSet<_>>() =>
            {
                render_types
            }
            _ => repaired,
        };

        Some(Self { blocks })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RenderBlock {
    Heading {
        text: String,
    },
    Paragraph {
        text: String,
    },
    BulletList {
        items: Vec<String>,
    },
    CodeBlock {
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
        code: String,
    },
    Citations {
        sources: Vec<String>,
    },
}

impl RenderBlock {
    fn render_type(&self) -> RenderType {
        match self {
            Self::Heading { .. } => RenderType::Heading,
            Self::Paragraph { .. } => RenderType::Paragraph,
            Self::BulletList { .. } => RenderType::BulletList,
            Self::CodeBlock { .. } => RenderType::CodeBlock,
            Self::Citations { .. } => RenderType::Citations,
        }
    }

    fn to_plain_text(&self) -> String {
        match self {
            Self::Heading { text } => format!("**{}**", text.trim()),
            Self::Paragraph { text } => text.clone(),
            Self::BulletList { items } => items
                .iter()
                .map(|item| format!("- {item}"))
                .collect::<Vec<_>>()
                .join("\n"),
            Self::CodeBlock { language, code } => {
                match language.as_deref().filter(|value| !value.is_empty()) {
                    Some(language) => format!("```{language}\n{code}\n```"),
                    None => format!("```\n{code}\n```"),
                }
            }
            Self::Citations { sources } => format!("Sources: {}", sources.join(", ")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RenderType {
    Heading,
    Paragraph,
    BulletList,
    CodeBlock,
    Citations,
}

impl RenderType {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "heading" => Some(Self::Heading),
            "paragraph" => Some(Self::Paragraph),
            "bullet_list" => Some(Self::BulletList),
            "code_block" => Some(Self::CodeBlock),
            "citations" => Some(Self::Citations),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RenderEnvelope {
    #[serde(default)]
    render_types: Vec<String>,
    blocks: Vec<RenderBlockEnvelope>,
}

#[derive(Debug, Deserialize)]
struct RenderBlockEnvelope {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    items: Option<Vec<String>>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    sources: Option<Vec<String>>,
}

impl RenderBlockEnvelope {
    fn into_block(self) -> Option<RenderBlock> {
        match self.kind.as_str() {
            "heading" => {
                let text = self.text?.trim().to_string();
                if text.is_empty() {
                    None
                } else {
                    Some(RenderBlock::Heading { text })
                }
            }
            "paragraph" => {
                let text = self.text?.trim().to_string();
                if text.is_empty() {
                    None
                } else {
                    Some(RenderBlock::Paragraph { text })
                }
            }
            "bullet_list" => {
                let items = self
                    .items
                    .unwrap_or_default()
                    .into_iter()
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty())
                    .collect::<Vec<_>>();
                if items.is_empty() {
                    None
                } else {
                    Some(RenderBlock::BulletList { items })
                }
            }
            "code_block" => {
                let code = self.code?;
                if code.trim().is_empty() {
                    None
                } else {
                    Some(RenderBlock::CodeBlock {
                        language: self.language,
                        code,
                    })
                }
            }
            "citations" => {
                let sources = self
                    .sources
                    .unwrap_or_default()
                    .into_iter()
                    .map(|source| source.trim().to_string())
                    .filter(|source| !source.is_empty())
                    .collect::<Vec<_>>();
                if sources.is_empty() {
                    None
                } else {
                    Some(RenderBlock::Citations { sources })
                }
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct PartialRenderBlock {
    kind: Option<String>,
    text: Option<String>,
    items: Option<Vec<String>>,
    language: Option<String>,
    code: Option<String>,
    sources: Option<Vec<String>>,
}

impl PartialRenderBlock {
    fn parse(fragment: &str) -> Option<Self> {
        let repaired = repair_json_object_fragment(fragment)?;
        let value: Value = serde_json::from_str(&repaired).ok()?;
        let object = value.as_object()?;
        let partial = Self {
            kind: object
                .get("type")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string),
            text: object
                .get("text")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string),
            items: string_array_field(object, "items"),
            language: object
                .get("language")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string),
            code: object
                .get("code")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(ToString::to_string),
            sources: string_array_field(object, "sources"),
        };

        partial.has_any_field().then_some(partial)
    }

    fn merge(parts: &[Self]) -> Option<Self> {
        let mut merged = Self::default();
        for part in parts {
            merged.kind = merge_optional(merged.kind.take(), part.kind.clone())?;
            merged.text = merge_optional(merged.text.take(), part.text.clone())?;
            merged.items = merge_optional(merged.items.take(), part.items.clone())?;
            merged.language = merge_optional(merged.language.take(), part.language.clone())?;
            merged.code = merge_optional(merged.code.take(), part.code.clone())?;
            merged.sources = merge_optional(merged.sources.take(), part.sources.clone())?;
        }

        merged.resolved_render_type()?;
        Some(merged)
    }

    fn into_block(self) -> Option<RenderBlock> {
        match self.resolved_render_type()? {
            RenderType::Heading => Some(RenderBlock::Heading { text: self.text? }),
            RenderType::Paragraph => Some(RenderBlock::Paragraph { text: self.text? }),
            RenderType::BulletList => Some(RenderBlock::BulletList { items: self.items? }),
            RenderType::CodeBlock => Some(RenderBlock::CodeBlock {
                language: self.language,
                code: self.code?,
            }),
            RenderType::Citations => Some(RenderBlock::Citations {
                sources: self.sources?,
            }),
        }
    }

    fn resolved_render_type(&self) -> Option<RenderType> {
        let declared = self.kind.as_deref().and_then(RenderType::from_str);
        let inferred = self.inferred_render_type()?;

        declared.map_or(Some(inferred), |declared| {
            (declared == inferred).then_some(declared)
        })
    }

    fn inferred_render_type(&self) -> Option<RenderType> {
        let has_text = self.text.is_some();
        let has_items = self.items.as_ref().is_some_and(|items| !items.is_empty());
        let has_code = self.code.is_some();
        let has_sources = self
            .sources
            .as_ref()
            .is_some_and(|sources| !sources.is_empty());
        let has_language = self.language.is_some();

        match (has_text, has_items, has_code || has_language, has_sources) {
            (true, false, false, false) => Some(RenderType::Paragraph),
            (false, true, false, false) => Some(RenderType::BulletList),
            (false, false, true, false) if has_code => Some(RenderType::CodeBlock),
            (false, false, false, true) => Some(RenderType::Citations),
            _ => None,
        }
    }

    fn has_any_field(&self) -> bool {
        self.kind.is_some()
            || self.text.is_some()
            || self.items.is_some()
            || self.language.is_some()
            || self.code.is_some()
            || self.sources.is_some()
    }
}

fn parse_plain_text_render_document(input: &str) -> Option<RenderDocument> {
    let mut blocks = Vec::new();
    let mut paragraph_lines = Vec::<String>::new();
    let mut bullet_items = Vec::<String>::new();
    let mut in_code_block = false;
    let mut code_language = None::<String>;
    let mut code_lines = Vec::<String>::new();

    for line in input.trim().lines() {
        let trimmed = line.trim();

        if let Some(fence) = trimmed.strip_prefix("```") {
            flush_paragraph(&mut blocks, &mut paragraph_lines);
            flush_bullets(&mut blocks, &mut bullet_items);
            if in_code_block {
                blocks.push(RenderBlock::CodeBlock {
                    language: code_language.take(),
                    code: code_lines.join("\n"),
                });
                code_lines.clear();
                in_code_block = false;
            } else {
                in_code_block = true;
                code_language = (!fence.trim().is_empty()).then(|| fence.trim().to_string());
            }
            continue;
        }

        if in_code_block {
            code_lines.push(line.to_string());
            continue;
        }

        if trimmed.is_empty() {
            flush_paragraph(&mut blocks, &mut paragraph_lines);
            flush_bullets(&mut blocks, &mut bullet_items);
            continue;
        }

        if let Some(text) = parse_heading_line(trimmed) {
            flush_paragraph(&mut blocks, &mut paragraph_lines);
            flush_bullets(&mut blocks, &mut bullet_items);
            blocks.push(RenderBlock::Heading { text });
            continue;
        }

        if let Some(item) = trimmed.strip_prefix("- ") {
            flush_paragraph(&mut blocks, &mut paragraph_lines);
            bullet_items.push(strip_balanced_marker_pairs(item.trim(), "**"));
            continue;
        }

        if let Some(sources) = parse_citations_line(trimmed) {
            flush_paragraph(&mut blocks, &mut paragraph_lines);
            flush_bullets(&mut blocks, &mut bullet_items);
            blocks.push(RenderBlock::Citations { sources });
            continue;
        }

        paragraph_lines.push(strip_balanced_marker_pairs(line, "**"));
    }

    if in_code_block {
        blocks.push(RenderBlock::CodeBlock {
            language: code_language.take(),
            code: code_lines.join("\n"),
        });
    }
    flush_paragraph(&mut blocks, &mut paragraph_lines);
    flush_bullets(&mut blocks, &mut bullet_items);

    (!blocks.is_empty()).then_some(RenderDocument { blocks })
}

fn parse_heading_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if let Some(heading) = trimmed.strip_prefix('#') {
        let heading = heading.trim_start_matches('#').trim();
        if !heading.is_empty() {
            return Some(heading.to_string());
        }
    }

    if trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.len() > 4 {
        let heading = trimmed
            .trim_start_matches("**")
            .trim_end_matches("**")
            .trim();
        if !heading.is_empty() && !heading.contains("**") {
            return Some(heading.to_string());
        }
    }

    None
}

fn parse_citations_line(line: &str) -> Option<Vec<String>> {
    let sources = line.strip_prefix("Sources:")?;
    let sources = sources
        .split(',')
        .map(str::trim)
        .filter(|source| !source.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    (!sources.is_empty()).then_some(sources)
}

fn flush_paragraph(blocks: &mut Vec<RenderBlock>, lines: &mut Vec<String>) {
    let text = lines
        .iter()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();
    lines.clear();
    if !text.is_empty() {
        blocks.push(RenderBlock::Paragraph { text });
    }
}

fn flush_bullets(blocks: &mut Vec<RenderBlock>, items: &mut Vec<String>) {
    if items.is_empty() {
        return;
    }
    blocks.push(RenderBlock::BulletList {
        items: std::mem::take(items),
    });
}

fn salvage_partial_render_document(response: &str) -> Option<RenderDocument> {
    let partials = candidate_json_object_fragments(response)
        .into_iter()
        .filter_map(|fragment| PartialRenderBlock::parse(&fragment))
        .collect::<Vec<_>>();
    if partials.is_empty() {
        return None;
    }

    let mut blocks = Vec::new();
    let mut cursor = 0;
    while cursor < partials.len() {
        let max_width = usize::min(3, partials.len() - cursor);
        let mut consumed = None;
        for width in (1..=max_width).rev() {
            let candidate = PartialRenderBlock::merge(&partials[cursor..cursor + width]);
            let Some(block) = candidate.and_then(PartialRenderBlock::into_block) else {
                continue;
            };
            blocks.push(block);
            consumed = Some(width);
            break;
        }

        cursor += consumed.unwrap_or(1);
    }

    (!blocks.is_empty()).then_some(RenderDocument { blocks })
}

fn candidate_json_object_fragments(response: &str) -> Vec<String> {
    let body = unwrap_jsonish_body(response);
    let mut fragments = Vec::new();
    let mut starts = Vec::new();
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in body.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '{' if !in_string => starts.push(index),
            '}' if !in_string => {
                if let Some(start) = starts.pop() {
                    fragments.push((start, body[start..=index].to_string()));
                }
            }
            _ => {}
        }
    }

    for start in starts {
        fragments.push((start, body[start..].to_string()));
    }

    fragments.sort_by_key(|(start, _)| *start);
    fragments
        .into_iter()
        .map(|(_, fragment)| fragment)
        .collect()
}

fn unwrap_jsonish_body(response: &str) -> &str {
    let trimmed = response.trim();
    if trimmed.starts_with("```") {
        trimmed
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
    } else {
        trimmed
    }
}

fn repair_json_object_fragment(fragment: &str) -> Option<String> {
    let mut repaired = fragment.trim().trim_end_matches(',').trim_end().to_string();
    if !repaired.starts_with('{') {
        return None;
    }

    for ch in unmatched_json_closers(&repaired)? {
        repaired.push(ch);
    }

    Some(repaired)
}

fn unmatched_json_closers(fragment: &str) -> Option<Vec<char>> {
    let mut stack = Vec::new();
    let mut in_string = false;
    let mut escaped = false;

    for ch in fragment.chars() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '{' | '[' if !in_string => stack.push(ch),
            '}' if !in_string => match stack.pop() {
                Some('{') => {}
                _ => return None,
            },
            ']' if !in_string => match stack.pop() {
                Some('[') => {}
                _ => return None,
            },
            _ => {}
        }
    }

    if in_string {
        return None;
    }

    Some(
        stack
            .into_iter()
            .rev()
            .map(|ch| match ch {
                '{' => '}',
                '[' => ']',
                _ => unreachable!("only braces and brackets are pushed"),
            })
            .collect(),
    )
}

fn string_array_field(object: &serde_json::Map<String, Value>, key: &str) -> Option<Vec<String>> {
    let values = object.get(key)?.as_array()?;
    let items = values
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    (!items.is_empty()).then_some(items)
}

fn merge_optional<T: PartialEq>(current: Option<T>, incoming: Option<T>) -> Option<Option<T>> {
    match (current, incoming) {
        (None, None) => Some(None),
        (Some(current), None) => Some(Some(current)),
        (None, Some(incoming)) => Some(Some(incoming)),
        (Some(current), Some(incoming)) if current == incoming => Some(Some(current)),
        _ => None,
    }
}

fn unique_render_types(render_types: Vec<RenderType>) -> Option<Vec<RenderType>> {
    let unique = render_types.iter().copied().collect::<BTreeSet<_>>();
    if unique.len() != render_types.len() {
        None
    } else {
        Some(render_types)
    }
}

fn render_types_for_blocks(blocks: &[RenderBlock]) -> Vec<RenderType> {
    let mut seen = BTreeSet::new();
    let mut ordered = Vec::new();
    for render_type in blocks.iter().map(RenderBlock::render_type) {
        if seen.insert(render_type) {
            ordered.push(render_type);
        }
    }
    ordered
}

fn block_separator(previous: &RenderBlock, next: &RenderBlock) -> &'static str {
    if uses_compact_block_separator(previous, next) {
        "\n"
    } else {
        "\n\n"
    }
}

pub(crate) fn uses_compact_block_separator(previous: &RenderBlock, next: &RenderBlock) -> bool {
    heading_continues_into_content(previous, next)
        || paragraph_continues_into_bullets(previous, next)
}

fn heading_continues_into_content(previous: &RenderBlock, next: &RenderBlock) -> bool {
    matches!(previous, RenderBlock::Heading { .. }) && !matches!(next, RenderBlock::Heading { .. })
}

fn paragraph_continues_into_bullets(previous: &RenderBlock, next: &RenderBlock) -> bool {
    matches!(previous, RenderBlock::Paragraph { text } if is_ordered_section_label(text))
        && matches!(next, RenderBlock::BulletList { .. })
}

fn is_ordered_section_label(text: &str) -> bool {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    matches!(
        (lines.next(), lines.next()),
        (Some(line), None) if ordered_marker_end(line.trim()).is_some()
    )
}

fn ordered_marker_end(line: &str) -> Option<usize> {
    let digit_count = line
        .bytes()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digit_count == 0 {
        return None;
    }

    let marker_end = digit_count + 2;
    (line.as_bytes().get(digit_count) == Some(&b'.')
        && line.as_bytes().get(digit_count + 1) == Some(&b' '))
    .then_some(marker_end)
}

fn strip_balanced_marker_pairs(input: &str, marker: &str) -> String {
    if marker.is_empty() {
        return input.to_string();
    }

    let marker_chars = marker.chars().collect::<Vec<_>>();
    let chars = input.chars().collect::<Vec<_>>();
    let mut rendered = String::new();
    let mut cursor = 0;
    let mut in_inline_code = false;
    let mut marker_open = false;

    while cursor < chars.len() {
        if chars[cursor] == '`' {
            in_inline_code = !in_inline_code;
            rendered.push(chars[cursor]);
            cursor += 1;
            continue;
        }

        if !in_inline_code && starts_with_marker(&chars, cursor, &marker_chars) {
            if marker_open {
                marker_open = false;
                cursor += marker_chars.len();
                continue;
            }

            if has_closing_marker(&chars, cursor + marker_chars.len(), &marker_chars) {
                marker_open = true;
                cursor += marker_chars.len();
                continue;
            }
        }

        rendered.push(chars[cursor]);
        cursor += 1;
    }

    rendered
}

fn starts_with_marker(chars: &[char], cursor: usize, marker: &[char]) -> bool {
    chars
        .get(cursor..cursor + marker.len())
        .is_some_and(|window| window == marker)
}

fn has_closing_marker(chars: &[char], mut cursor: usize, marker: &[char]) -> bool {
    let mut in_inline_code = false;
    while cursor < chars.len() {
        if chars[cursor] == '`' {
            in_inline_code = !in_inline_code;
            cursor += 1;
            continue;
        }

        if !in_inline_code && starts_with_marker(chars, cursor, marker) {
            return true;
        }
        cursor += 1;
    }

    false
}

fn extract_json_payload(response: &str) -> Option<&str> {
    if response.starts_with('{') && response.ends_with('}') {
        return Some(response);
    }

    if response.starts_with("```") && response.ends_with("```") {
        let inner = response
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        if inner.starts_with('{') && inner.ends_with('}') {
            return Some(inner);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{RenderBlock, RenderDocument};

    #[test]
    fn parses_heading_blocks_from_structured_responses() {
        let response = r#"{
  "render_types": ["heading", "paragraph"],
  "blocks": [
    {"type": "heading", "text": "Summary"},
    {"type": "paragraph", "text": "Hello."}
  ]
}"#;

        let parsed = RenderDocument::parse_assistant_response(response).expect("render document");
        assert_eq!(
            parsed.blocks,
            vec![
                RenderBlock::Heading {
                    text: "Summary".to_string()
                },
                RenderBlock::Paragraph {
                    text: "Hello.".to_string()
                }
            ]
        );
        assert_eq!(parsed.to_plain_text(), "**Summary**\nHello.");
    }

    #[test]
    fn parses_markdownish_heading_lines_from_plain_text() {
        let parsed =
            RenderDocument::from_assistant_plain_text("**HTTP API Design For Paddles**\n\nBody");

        assert_eq!(
            parsed.blocks,
            vec![
                RenderBlock::Heading {
                    text: "HTTP API Design For Paddles".to_string()
                },
                RenderBlock::Paragraph {
                    text: "Body".to_string()
                }
            ]
        );
    }

    #[test]
    fn strips_inline_bold_markers_inside_paragraph_text() {
        let parsed = RenderDocument::from_assistant_plain_text(
            "The next item is **HTTP API Design For Paddles**.",
        );

        assert_eq!(
            parsed.blocks,
            vec![RenderBlock::Paragraph {
                text: "The next item is HTTP API Design For Paddles.".to_string()
            }]
        );
    }

    #[test]
    fn keeps_numbered_section_labels_tight_with_following_bullets() {
        let document = RenderDocument {
            blocks: vec![
                RenderBlock::Paragraph {
                    text: "1. Shared contract surfaces".to_string(),
                },
                RenderBlock::BulletList {
                    items: vec![
                        "Define what HQ owns vs what spoke owns.".to_string(),
                        "Promote shared schemas.".to_string(),
                    ],
                },
                RenderBlock::Paragraph {
                    text: "2. Tighter dev workflow integration".to_string(),
                },
                RenderBlock::BulletList {
                    items: vec!["Make it easy to test HQ changes locally.".to_string()],
                },
            ],
        };

        assert_eq!(
            document.to_plain_text(),
            "1. Shared contract surfaces\n- Define what HQ owns vs what spoke owns.\n- Promote shared schemas.\n\n2. Tighter dev workflow integration\n- Make it easy to test HQ changes locally."
        );
    }

    #[test]
    fn keeps_headings_tight_with_following_content() {
        let document = RenderDocument {
            blocks: vec![
                RenderBlock::Heading {
                    text: "Summary".to_string(),
                },
                RenderBlock::Paragraph {
                    text: "Body".to_string(),
                },
                RenderBlock::Heading {
                    text: "Checklist".to_string(),
                },
                RenderBlock::BulletList {
                    items: vec!["Ship it.".to_string()],
                },
            ],
        };

        assert_eq!(
            document.to_plain_text(),
            "**Summary**\nBody\n\n**Checklist**\n- Ship it."
        );
    }
}
