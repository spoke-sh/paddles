use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;

const SUPPORTED_RENDER_TYPES: [&str; 4] = ["paragraph", "bullet_list", "code_block", "citations"];
pub const ANTHROPIC_RENDER_TOOL_NAME: &str = "render_final_answer";
const INVALID_STRUCTURED_RESPONSE_EXCERPT_LINES: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderCapability {
    PromptEnvelope,
    OpenAiJsonSchema,
    AnthropicToolUse,
    GeminiJsonSchema,
}

impl RenderCapability {
    pub fn resolve(provider: &str, _model_id: &str) -> Self {
        match provider {
            "openai" | "inception" => Self::OpenAiJsonSchema,
            "anthropic" => Self::AnthropicToolUse,
            "google" => Self::GeminiJsonSchema,
            _ => Self::PromptEnvelope,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::PromptEnvelope => "prompt-envelope",
            Self::OpenAiJsonSchema => "openai-json-schema",
            Self::AnthropicToolUse => "anthropic-tool-use",
            Self::GeminiJsonSchema => "gemini-json-schema",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RenderType {
    Paragraph,
    BulletList,
    CodeBlock,
    Citations,
}

impl RenderType {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "paragraph" => Some(Self::Paragraph),
            "bullet_list" => Some(Self::BulletList),
            "code_block" => Some(Self::CodeBlock),
            "citations" => Some(Self::Citations),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AssistantResponse {
    render_types: Vec<RenderType>,
    blocks: Vec<AssistantBlock>,
}

impl AssistantResponse {
    fn parse(response: &str) -> Option<Self> {
        let trimmed = response.trim();
        extract_json_payload(trimmed)
            .and_then(Self::parse_json_envelope)
            .or_else(|| salvage_partial_assistant_response(trimmed))
    }

    fn parse_json_envelope(json: &str) -> Option<Self> {
        let envelope: AssistantResponseEnvelope = serde_json::from_str(json).ok()?;
        Self::from_envelope(envelope)
    }

    fn from_envelope(envelope: AssistantResponseEnvelope) -> Option<Self> {
        let blocks: Vec<AssistantBlock> = envelope
            .blocks
            .into_iter()
            .filter_map(AssistantBlock::from_wire)
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

        let render_types = match declared {
            Some(render_types)
                if render_types.iter().copied().collect::<BTreeSet<_>>()
                    == repaired.iter().copied().collect::<BTreeSet<_>>() =>
            {
                render_types
            }
            _ => repaired,
        };

        Some(Self {
            render_types,
            blocks,
        })
    }

    fn to_plain_text(&self) -> String {
        let mut rendered = Vec::new();
        for block in &self.blocks {
            rendered.push(block.to_plain_text());
        }
        rendered
            .into_iter()
            .filter(|block| !block.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AssistantBlock {
    Paragraph(String),
    BulletList(Vec<String>),
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    Citations(Vec<String>),
}

impl AssistantBlock {
    fn from_wire(block: AssistantBlockEnvelope) -> Option<Self> {
        match block.kind.as_str() {
            "paragraph" => {
                let text = block.text?.trim().to_string();
                if text.is_empty() {
                    None
                } else {
                    Some(Self::Paragraph(text))
                }
            }
            "bullet_list" => {
                let items = block
                    .items
                    .into_iter()
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty())
                    .collect::<Vec<_>>();
                if items.is_empty() {
                    None
                } else {
                    Some(Self::BulletList(items))
                }
            }
            "code_block" => {
                let code = block.code?;
                if code.trim().is_empty() {
                    None
                } else {
                    Some(Self::CodeBlock {
                        language: block.language,
                        code,
                    })
                }
            }
            "citations" => {
                let sources = block
                    .sources
                    .into_iter()
                    .map(|source| source.trim().to_string())
                    .filter(|source| !source.is_empty())
                    .collect::<Vec<_>>();
                if sources.is_empty() {
                    None
                } else {
                    Some(Self::Citations(sources))
                }
            }
            _ => None,
        }
    }

    fn render_type(&self) -> RenderType {
        match self {
            Self::Paragraph(_) => RenderType::Paragraph,
            Self::BulletList(_) => RenderType::BulletList,
            Self::CodeBlock { .. } => RenderType::CodeBlock,
            Self::Citations(_) => RenderType::Citations,
        }
    }

    fn to_plain_text(&self) -> String {
        match self {
            Self::Paragraph(text) => text.clone(),
            Self::BulletList(items) => items
                .iter()
                .map(|item| format!("- {item}"))
                .collect::<Vec<_>>()
                .join("\n"),
            Self::CodeBlock { language, code } => {
                match language.as_deref().filter(|s| !s.is_empty()) {
                    Some(language) => format!("```{language}\n{code}\n```"),
                    None => format!("```\n{code}\n```"),
                }
            }
            Self::Citations(sources) => format!("Sources: {}", sources.join(", ")),
        }
    }
}

#[derive(Debug, Deserialize)]
struct AssistantResponseEnvelope {
    #[serde(default)]
    render_types: Vec<String>,
    blocks: Vec<AssistantBlockEnvelope>,
}

#[derive(Debug, Deserialize)]
struct AssistantBlockEnvelope {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    items: Vec<String>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    sources: Vec<String>,
}

pub fn final_answer_contract_prompt(
    render_capability: RenderCapability,
    require_citations: bool,
) -> String {
    let citation_rule = if require_citations {
        "Include exactly one `citations` block listing the repository/file sources used in the answer."
    } else {
        "Use a `citations` block only when repository/file sources are part of the answer."
    };
    let transport_rule = match render_capability {
        RenderCapability::PromptEnvelope => {
            "Respond with ONLY one complete JSON object and no prose outside it."
        }
        RenderCapability::OpenAiJsonSchema | RenderCapability::GeminiJsonSchema => {
            "The transport enforces a JSON schema. Fill the structured response envelope exactly and do not emit partial JSON."
        }
        RenderCapability::AnthropicToolUse => {
            "Use the render_final_answer tool exactly once with arguments that satisfy the schema."
        }
    };
    let grounded_rule = if require_citations {
        "- The attached evidence came from actions Paddles already executed locally.\n\
- Treat the user's stated repository failure or broken-state claim as a hypothesis unless attached evidence confirms it.\n\
- If the evidence weakens or contradicts that premise, say so explicitly instead of repeating the original claim as fact.\n\
- Report what those completed actions found.\n\
- Do not say you will run tools or inspect the workspace later.\n\
- Do not ask the user for logs, file contents, or repository state that the harness could already inspect.\n\
- Do not claim that tools produced no output when evidence is attached.\n\
- Do not ask the user to confirm the environment when attached evidence already proves the command ran.\n\
"
    } else {
        ""
    };
    format!(
        "Final answer rendering contract:\n\
{transport_rule}\n\
Supported render types: {}.\n\
Schema:\n\
{{\"render_types\":[\"paragraph\",\"citations\"],\"blocks\":[{{\"type\":\"paragraph\",\"text\":\"...\"}},{{\"type\":\"citations\",\"sources\":[\"README.md\"]}}]}}\n\
Rules:\n\
- Return exactly one complete JSON object.\n\
- Start with the `render_types` field and then provide `blocks`.\n\
- Do not wrap the JSON in markdown fences, prose, or commentary.\n\
- Do not emit partial blocks, truncated arrays, or unfinished objects.\n\
- `render_types` must list the exact block types used in `blocks`.\n\
{grounded_rule}\
- Use `paragraph` for normal prose.\n\
- A `paragraph` block must include `text`.\n\
- Use `bullet_list` for short flat lists.\n\
- A `bullet_list` block must include `items`.\n\
- Use `code_block` only for literal code or terminal output.\n\
- A `code_block` block must include `code`.\n\
- Do not use markdown headings, `**bold**`, or list markers inside `paragraph` text.\n\
- `citations` sources must be plain repository/file references.\n\
- A `citations` block must include `sources`.\n\
- {citation_rule}",
        SUPPORTED_RENDER_TYPES.join(", ")
    )
}

pub fn assistant_response_json_schema(require_citations: bool) -> Value {
    let citation_guidance = if require_citations {
        "Required when repository/file sources were used in the answer."
    } else {
        "Optional when no repository/file sources were used in the answer."
    };
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "render_types": {
                "type": "array",
                "description": "The exact unique block types used in `blocks`.",
                "items": {
                    "type": "string",
                    "enum": SUPPORTED_RENDER_TYPES,
                },
                "minItems": 1
            },
            "blocks": {
                "type": "array",
                "description": "Ordered render blocks for the final answer.",
                "minItems": 1,
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": SUPPORTED_RENDER_TYPES,
                            "description": "Render block type."
                        },
                        "text": {
                            "type": "string",
                            "description": "Paragraph text for `paragraph` blocks."
                        },
                        "items": {
                            "type": "array",
                            "description": "Flat list entries for `bullet_list` blocks.",
                            "items": { "type": "string" }
                        },
                        "language": {
                            "type": "string",
                            "description": "Optional language tag for `code_block` blocks."
                        },
                        "code": {
                            "type": "string",
                            "description": "Literal code or terminal output for `code_block` blocks."
                        },
                        "sources": {
                            "type": "array",
                            "description": format!("Repository/file citations for `citations` blocks. {citation_guidance}"),
                            "items": { "type": "string" }
                        }
                    },
                    "required": ["type"],
                    "allOf": [
                        {
                            "if": { "properties": { "type": { "const": "paragraph" } }, "required": ["type"] },
                            "then": { "required": ["text"] }
                        },
                        {
                            "if": { "properties": { "type": { "const": "bullet_list" } }, "required": ["type"] },
                            "then": { "required": ["items"] }
                        },
                        {
                            "if": { "properties": { "type": { "const": "code_block" } }, "required": ["type"] },
                            "then": { "required": ["code"] }
                        },
                        {
                            "if": { "properties": { "type": { "const": "citations" } }, "required": ["type"] },
                            "then": { "required": ["sources"] }
                        }
                    ]
                }
            }
        },
        "required": ["render_types", "blocks"]
    })
}

pub fn normalize_assistant_response(response: &str) -> String {
    match AssistantResponse::parse(response) {
        Some(parsed) => parsed.to_plain_text(),
        None => invalid_structured_response_fallback(response)
            .unwrap_or_else(|| sanitize_markdownish_fallback(response.trim())),
    }
}

pub fn ensure_citation_section(reply: &str, citations: &[String]) -> String {
    if citations.is_empty() || reply.contains("Sources:") {
        return reply.to_string();
    }

    format!("{reply}\n\nSources: {}", citations.join(", "))
}

fn sanitize_markdownish_fallback(input: &str) -> String {
    let mut sanitized = Vec::new();
    let mut in_code_block = false;

    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            sanitized.push(line.to_string());
            continue;
        }

        if in_code_block {
            sanitized.push(line.to_string());
        } else {
            sanitized.push(strip_balanced_marker_pairs(line, "**"));
        }
    }

    sanitized.join("\n")
}

fn invalid_structured_response_fallback(response: &str) -> Option<String> {
    let trimmed = unwrap_jsonish_body(response).trim();
    if !looks_like_structured_response_payload(trimmed) {
        return None;
    }

    let line_count = trimmed.lines().count().max(1);
    let char_count = trimmed.chars().count();
    let excerpt_lines = trimmed
        .lines()
        .take(INVALID_STRUCTURED_RESPONSE_EXCERPT_LINES)
        .collect::<Vec<_>>();
    let excerpt = if excerpt_lines.is_empty() {
        trimmed.to_string()
    } else {
        excerpt_lines.join("\n").trim_end().to_string()
    };

    let payload_scope_note = if line_count > INVALID_STRUCTURED_RESPONSE_EXCERPT_LINES {
        format!(
            "excerpt from a {}-line / {}-char payload",
            line_count, char_count
        )
    } else {
        format!(
            "complete payload, {} lines / {} chars",
            line_count, char_count
        )
    };
    let truncation_note = if line_count > INVALID_STRUCTURED_RESPONSE_EXCERPT_LINES {
        format!(
            "\n\nPayload excerpt truncated after {} lines.",
            INVALID_STRUCTURED_RESPONSE_EXCERPT_LINES
        )
    } else {
        format!("\n\nThe payload itself ended after {} lines.", line_count)
    };
    let escaped_view = if line_count <= 6 {
        format!(
            "\n\nEscaped payload view:\n```text\n{}\n```",
            trimmed.escape_default()
        )
    } else {
        String::new()
    };

    Some(format!(
        "The model returned an invalid structured answer.\n\nRaw payload excerpt ({payload_scope_note}):\n```json\n{}\n```{}{}",
        excerpt, truncation_note, escaped_view
    ))
}

fn looks_like_structured_response_payload(response: &str) -> bool {
    response.starts_with('{')
        || response.starts_with('[')
        || response.contains("\"blocks\"")
        || response.contains("\"render_types\"")
}

#[derive(Clone, Debug, Default)]
struct PartialAssistantBlock {
    kind: Option<String>,
    text: Option<String>,
    items: Option<Vec<String>>,
    language: Option<String>,
    code: Option<String>,
    sources: Option<Vec<String>>,
}

impl PartialAssistantBlock {
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

    fn into_block(self) -> Option<AssistantBlock> {
        match self.resolved_render_type()? {
            RenderType::Paragraph => Some(AssistantBlock::Paragraph(self.text?)),
            RenderType::BulletList => Some(AssistantBlock::BulletList(self.items?)),
            RenderType::CodeBlock => Some(AssistantBlock::CodeBlock {
                language: self.language,
                code: self.code?,
            }),
            RenderType::Citations => Some(AssistantBlock::Citations(self.sources?)),
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

fn salvage_partial_assistant_response(response: &str) -> Option<AssistantResponse> {
    let partials = candidate_json_object_fragments(response)
        .into_iter()
        .filter_map(|fragment| PartialAssistantBlock::parse(&fragment))
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
            let candidate = PartialAssistantBlock::merge(&partials[cursor..cursor + width]);
            let Some(block) = candidate.and_then(PartialAssistantBlock::into_block) else {
                continue;
            };
            blocks.push(block);
            consumed = Some(width);
            break;
        }

        cursor += consumed.unwrap_or(1);
    }

    if blocks.is_empty() {
        return None;
    }

    let render_types = render_types_for_blocks(&blocks);
    Some(AssistantResponse {
        render_types,
        blocks,
    })
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

fn render_types_for_blocks(blocks: &[AssistantBlock]) -> Vec<RenderType> {
    let mut seen = BTreeSet::new();
    let mut ordered = Vec::new();
    for render_type in blocks.iter().map(AssistantBlock::render_type) {
        if seen.insert(render_type) {
            ordered.push(render_type);
        }
    }
    ordered
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
    use super::{
        AssistantResponse, RenderCapability, assistant_response_json_schema,
        ensure_citation_section, final_answer_contract_prompt, normalize_assistant_response,
    };

    #[test]
    fn parses_and_flattens_structured_assistant_responses() {
        let response = r#"{
  "render_types": ["paragraph", "bullet_list", "code_block", "citations"],
  "blocks": [
    {"type": "paragraph", "text": "The board is ready."},
    {"type": "bullet_list", "items": ["Ship the slice", "Update the board"]},
    {"type": "code_block", "language": "sh", "code": "git status --short"},
    {"type": "citations", "sources": ["README.md", "ARCHITECTURE.md"]}
  ]
}"#;

        let parsed = AssistantResponse::parse(response).expect("structured response");
        assert_eq!(parsed.render_types.len(), 4);
        assert_eq!(
            parsed.to_plain_text(),
            "The board is ready.\n\n- Ship the slice\n- Update the board\n\n```sh\ngit status --short\n```\n\nSources: README.md, ARCHITECTURE.md"
        );
    }

    #[test]
    fn repairs_structured_responses_when_declared_types_do_not_match_blocks() {
        let response = r#"{
  "render_types": ["paragraph"],
  "blocks": [
    {"type": "paragraph", "text": "Hello."},
    {"type": "citations", "sources": ["README.md"]}
  ]
}"#;

        let parsed = AssistantResponse::parse(response).expect("repaired response");
        assert_eq!(parsed.render_types.len(), 2);
        assert_eq!(parsed.to_plain_text(), "Hello.\n\nSources: README.md");
    }

    #[test]
    fn repairs_structured_responses_when_render_types_are_missing() {
        let response = r#"{
  "blocks": [
    {"type": "paragraph", "text": "Hello."}
  ]
}"#;

        let parsed = AssistantResponse::parse(response).expect("repaired response");
        assert_eq!(parsed.render_types.len(), 1);
        assert_eq!(parsed.to_plain_text(), "Hello.");
    }

    #[test]
    fn normalizes_raw_markdownish_bold_fallbacks() {
        let response = "The next item is **HTTP API Design For Paddles**.";
        assert_eq!(
            normalize_assistant_response(response),
            "The next item is HTTP API Design For Paddles."
        );
    }

    #[test]
    fn keeps_code_blocks_intact_when_sanitizing_markdownish_fallbacks() {
        let response = "```txt\n**literal**\n```";
        assert_eq!(normalize_assistant_response(response), response);
    }

    #[test]
    fn citation_sections_are_appended_once() {
        let reply = ensure_citation_section("Summary", &["README.md".to_string()]);
        assert_eq!(reply, "Summary\n\nSources: README.md");
        assert_eq!(
            ensure_citation_section(&reply, &["README.md".to_string()]),
            reply
        );
    }

    #[test]
    fn contract_prompt_advertises_supported_render_types() {
        let prompt = final_answer_contract_prompt(RenderCapability::PromptEnvelope, true);
        assert!(prompt.contains("paragraph, bullet_list, code_block, citations"));
        assert!(prompt.contains("Include exactly one `citations` block"));
        assert!(prompt.contains("Return exactly one complete JSON object"));
        assert!(prompt.contains("Do not emit partial blocks"));
    }

    #[test]
    fn native_render_capabilities_announce_transport_constraints() {
        let prompt = final_answer_contract_prompt(RenderCapability::AnthropicToolUse, false);
        assert!(prompt.contains("Use the render_final_answer tool exactly once"));
        let prompt = final_answer_contract_prompt(RenderCapability::OpenAiJsonSchema, false);
        assert!(prompt.contains("transport enforces a JSON schema"));
        assert!(prompt.contains("do not emit partial JSON"));
    }

    #[test]
    fn grounded_contract_prompt_forbids_future_intent_language() {
        let prompt = final_answer_contract_prompt(RenderCapability::OpenAiJsonSchema, true);

        assert!(
            prompt.contains(
                "The attached evidence came from actions Paddles already executed locally."
            )
        );
        assert!(prompt.contains("Do not say you will run tools"));
        assert!(prompt.contains("Do not ask the user for logs"));
    }

    #[test]
    fn grounded_contract_prompt_forbids_denying_attached_output() {
        let prompt = final_answer_contract_prompt(RenderCapability::OpenAiJsonSchema, true);

        assert!(
            prompt
                .contains("Do not claim that tools produced no output when evidence is attached.")
        );
        assert!(prompt.contains(
            "Do not ask the user to confirm the environment when attached evidence already proves the command ran."
        ));
    }

    #[test]
    fn grounded_contract_prompt_treats_user_failure_claims_as_hypotheses() {
        let prompt = final_answer_contract_prompt(RenderCapability::OpenAiJsonSchema, true);

        assert!(prompt.contains(
            "Treat the user's stated repository failure or broken-state claim as a hypothesis unless attached evidence confirms it."
        ));
        assert!(prompt.contains(
            "If the evidence weakens or contradicts that premise, say so explicitly instead of repeating the original claim as fact."
        ));
    }

    #[test]
    fn openai_compatible_render_capabilities_include_inception() {
        assert_eq!(
            RenderCapability::resolve("openai", "gpt-4o"),
            RenderCapability::OpenAiJsonSchema
        );
        assert_eq!(
            RenderCapability::resolve("inception", "mercury-2"),
            RenderCapability::OpenAiJsonSchema
        );
    }

    #[test]
    fn empty_citation_sources_do_not_poison_parse() {
        let response = r#"{"render_types":["paragraph","citations"],"blocks":[{"type":"paragraph","text":"Hello."},{"type":"citations","sources":[]}]}"#;
        assert_eq!(normalize_assistant_response(response), "Hello.");
    }

    #[test]
    fn salvages_fragmented_structured_responses_from_partial_json() {
        let response = r#"{
  "blocks": [
    {
      "type": "paragraph"
    },
    {
      "text": "Hey! How can I help you today?"
"#;

        assert_eq!(
            normalize_assistant_response(response),
            "Hey! How can I help you today?"
        );
    }

    #[test]
    fn unrecoverable_structured_responses_render_debuggable_json_excerpt() {
        let response = r#"{
  "blocks": [
    [
      " "
"#;

        let normalized = normalize_assistant_response(response);
        assert!(normalized.contains("invalid structured answer"));
        assert!(normalized.contains("Raw payload excerpt"));
        assert!(normalized.contains("complete payload"));
        assert!(normalized.contains("The payload itself ended after"));
        assert!(normalized.contains("Escaped payload view"));
        assert!(normalized.contains("\"blocks\""));
        assert!(normalized.contains("```json"));
    }

    #[test]
    fn assistant_response_schema_requires_blocks_and_render_types() {
        let schema = assistant_response_json_schema(true);
        assert_eq!(schema["type"].as_str(), Some("object"));
        assert_eq!(schema["required"][0].as_str(), Some("render_types"));
        assert_eq!(schema["required"][1].as_str(), Some("blocks"));
        assert_eq!(
            schema["properties"]["blocks"]["items"]["allOf"][0]["then"]["required"][0].as_str(),
            Some("text")
        );
        assert_eq!(
            schema["properties"]["blocks"]["items"]["allOf"][1]["then"]["required"][0].as_str(),
            Some("items")
        );
        assert_eq!(
            schema["properties"]["blocks"]["items"]["allOf"][2]["then"]["required"][0].as_str(),
            Some("code")
        );
        assert_eq!(
            schema["properties"]["blocks"]["items"]["allOf"][3]["then"]["required"][0].as_str(),
            Some("sources")
        );
    }
}
