use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;

const SUPPORTED_RENDER_TYPES: [&str; 4] = ["paragraph", "bullet_list", "code_block", "citations"];
pub const ANTHROPIC_RENDER_TOOL_NAME: &str = "render_final_answer";

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
            "openai" => Self::OpenAiJsonSchema,
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
        let json = extract_json_payload(response.trim())?;
        let envelope: AssistantResponseEnvelope = serde_json::from_str(json).ok()?;
        let blocks = envelope
            .blocks
            .into_iter()
            .map(AssistantBlock::from_wire)
            .collect::<Option<Vec<_>>>()?;
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
            "Respond with ONLY one JSON object and no prose outside it."
        }
        RenderCapability::OpenAiJsonSchema | RenderCapability::GeminiJsonSchema => {
            "The transport enforces a JSON schema. Fill the structured response envelope exactly."
        }
        RenderCapability::AnthropicToolUse => {
            "Use the render_final_answer tool exactly once with arguments that satisfy the schema."
        }
    };
    format!(
        "Final answer rendering contract:\n\
{transport_rule}\n\
Supported render types: {}.\n\
Schema:\n\
{{\"render_types\":[\"paragraph\",\"citations\"],\"blocks\":[{{\"type\":\"paragraph\",\"text\":\"...\"}},{{\"type\":\"citations\",\"sources\":[\"README.md\"]}}]}}\n\
Rules:\n\
- `render_types` must list the exact block types used in `blocks`.\n\
- Use `paragraph` for normal prose.\n\
- Use `bullet_list` for short flat lists.\n\
- Use `code_block` only for literal code or terminal output.\n\
- Do not use markdown headings, `**bold**`, or list markers inside `paragraph` text.\n\
- `citations` sources must be plain repository/file references.\n\
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
                    "required": ["type"]
                }
            }
        },
        "required": ["render_types", "blocks"]
    })
}

pub fn normalize_assistant_response(response: &str) -> String {
    match AssistantResponse::parse(response) {
        Some(parsed) => parsed.to_plain_text(),
        None => sanitize_markdownish_fallback(response.trim()),
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
    }

    #[test]
    fn native_render_capabilities_announce_transport_constraints() {
        let prompt = final_answer_contract_prompt(RenderCapability::AnthropicToolUse, false);
        assert!(prompt.contains("Use the render_final_answer tool exactly once"));
        let prompt = final_answer_contract_prompt(RenderCapability::OpenAiJsonSchema, false);
        assert!(prompt.contains("transport enforces a JSON schema"));
    }

    #[test]
    fn assistant_response_schema_requires_blocks_and_render_types() {
        let schema = assistant_response_json_schema(true);
        assert_eq!(schema["type"].as_str(), Some("object"));
        assert_eq!(schema["required"][0].as_str(), Some("render_types"));
        assert_eq!(schema["required"][1].as_str(), Some("blocks"));
    }
}
