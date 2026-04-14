use crate::domain::model::{RenderBlock, RenderDocument, SUPPORTED_RENDER_TYPES};
use serde_json::{Value, json};

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
        crate::infrastructure::providers::ModelProvider::from_name(provider)
            .map(|provider| provider.capability_surface(_model_id).render_capability)
            .unwrap_or(Self::PromptEnvelope)
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
{{\"render_types\":[\"heading\",\"paragraph\",\"citations\"],\"blocks\":[{{\"type\":\"heading\",\"text\":\"Summary\"}},{{\"type\":\"paragraph\",\"text\":\"...\"}},{{\"type\":\"citations\",\"sources\":[\"README.md\"]}}]}}\n\
Rules:\n\
- Return exactly one complete JSON object.\n\
- Start with the `render_types` field and then provide `blocks`.\n\
- Do not wrap the JSON in markdown fences, prose, or commentary.\n\
- Do not emit partial blocks, truncated arrays, or unfinished objects.\n\
- `render_types` must list the exact block types used in `blocks`.\n\
{grounded_rule}\
- Use `heading` for short section titles.\n\
- A `heading` block must include `text`.\n\
- Use `paragraph` for normal prose.\n\
- A `paragraph` block must include `text`.\n\
- Use `bullet_list` for short flat lists.\n\
- A `bullet_list` block must include `items`.\n\
- Prefer a `heading` or `paragraph` intro followed by one `bullet_list` for grouped recommendations.\n\
- Each `bullet_list` item must be a complete standalone point.\n\
- Nested lists are not supported; fold sub-points into the parent item text instead of emitting an intro bullet followed by more bullets.\n\
- Use `code_block` only for literal code or terminal output.\n\
- A `code_block` block must include `code`.\n\
- Do not use markdown headings, `**bold**`, or list markers inside `paragraph` text; use `heading` or `bullet_list` blocks instead.\n\
- Do not emit external web URLs unless they are backed by attached verified evidence for this turn.\n\
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
    let mut schema = json!({
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
                            "description": "Heading or paragraph text for `heading` and `paragraph` blocks."
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
    });

    let Some(block_item) = schema
        .get_mut("properties")
        .and_then(Value::as_object_mut)
        .and_then(|properties| properties.get_mut("blocks"))
        .and_then(Value::as_object_mut)
        .and_then(|blocks| blocks.get_mut("items"))
    else {
        return schema;
    };

    for property in ["text", "items", "language", "code", "sources"] {
        make_openai_schema_property_nullable(block_item, property);
    }
    make_openai_object_schema_require_all_properties(block_item);

    schema
}

fn make_openai_schema_property_nullable(schema: &mut Value, property_name: &str) {
    let Some(property) = schema
        .get_mut("properties")
        .and_then(Value::as_object_mut)
        .and_then(|properties| properties.get_mut(property_name))
    else {
        return;
    };

    let Some(object) = property.as_object_mut() else {
        return;
    };

    if let Some(property_type) = object.get_mut("type") {
        match property_type {
            Value::String(existing) => {
                let existing = existing.clone();
                *property_type = json!([existing, "null"]);
            }
            Value::Array(existing) => {
                if !existing.iter().any(|value| value.as_str() == Some("null")) {
                    existing.push(Value::String("null".to_string()));
                }
            }
            _ => {}
        }
    }
}

fn make_openai_object_schema_require_all_properties(schema: &mut Value) {
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return;
    };

    let required = properties
        .keys()
        .cloned()
        .map(Value::String)
        .collect::<Vec<_>>();

    if let Some(object) = schema.as_object_mut() {
        object.insert("required".to_string(), Value::Array(required));
    }
}

pub fn normalize_assistant_response(response: &str) -> String {
    if let Some(parsed) = RenderDocument::parse_assistant_response(response) {
        return parsed.to_plain_text();
    }

    invalid_structured_response_fallback(response).unwrap_or_else(|| {
        RenderDocument::from_assistant_plain_text(response.trim()).to_plain_text()
    })
}

pub fn ensure_citation_section(reply: &str, citations: &[String]) -> String {
    if citations.is_empty() || reply.contains("Sources:") {
        return reply.to_string();
    }

    format!("{reply}\n\nSources: {}", citations.join(", "))
}

pub fn extract_http_urls(text: &str) -> Vec<String> {
    let mut urls = Vec::new();

    for token in text.split_whitespace() {
        let candidate = token
            .find("https://")
            .or_else(|| token.find("http://"))
            .map(|index| &token[index..]);
        let Some(candidate) = candidate else {
            continue;
        };

        let trimmed = candidate
            .trim_end_matches(|ch: char| {
                matches!(
                    ch,
                    '.' | ',' | ';' | ':' | ')' | ']' | '}' | '"' | '\'' | '!'
                )
            })
            .trim_end_matches('/');
        if trimmed.is_empty() {
            continue;
        }
        let normalized = trimmed.to_string();
        if !urls.contains(&normalized) {
            urls.push(normalized);
        }
    }

    urls
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

    Some(
        RenderDocument {
            blocks: {
                let mut blocks = vec![
                    RenderBlock::Heading {
                        text: "Invalid Structured Answer".to_string(),
                    },
                    RenderBlock::Paragraph {
                        text: "The model returned an invalid structured answer.".to_string(),
                    },
                    RenderBlock::CodeBlock {
                        language: Some("json".to_string()),
                        code: excerpt,
                    },
                    RenderBlock::Paragraph {
                        text: format!("{payload_scope_note}. {}", truncation_note.trim()),
                    },
                ];
                if !escaped_view.is_empty() {
                    let escaped = escaped_view
                        .trim()
                        .trim_start_matches("Escaped payload view:\n```text\n")
                        .trim_end_matches("\n```")
                        .to_string();
                    blocks.push(RenderBlock::CodeBlock {
                        language: Some("text".to_string()),
                        code: escaped,
                    });
                }
                blocks
            },
        }
        .to_plain_text(),
    )
}

fn looks_like_structured_response_payload(response: &str) -> bool {
    response.starts_with('{')
        || response.starts_with('[')
        || response.contains("\"blocks\"")
        || response.contains("\"render_types\"")
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

#[cfg(test)]
mod tests {
    use super::{
        RenderCapability, assistant_response_json_schema, ensure_citation_section,
        extract_http_urls, final_answer_contract_prompt, normalize_assistant_response,
    };
    use crate::domain::model::{RenderBlock, RenderDocument};

    #[test]
    fn parses_and_flattens_structured_assistant_responses() {
        let response = r#"{
  "render_types": ["heading", "paragraph", "bullet_list", "code_block", "citations"],
  "blocks": [
    {"type": "heading", "text": "Summary"},
    {"type": "paragraph", "text": "The board is ready."},
    {"type": "bullet_list", "items": ["Ship the slice", "Update the board"]},
    {"type": "code_block", "language": "sh", "code": "git status --short"},
    {"type": "citations", "sources": ["README.md", "ARCHITECTURE.md"]}
  ]
}"#;

        let parsed =
            RenderDocument::parse_assistant_response(response).expect("structured response");
        assert_eq!(
            parsed.blocks[0],
            RenderBlock::Heading {
                text: "Summary".to_string()
            }
        );
        assert_eq!(
            parsed.to_plain_text(),
            "**Summary**\nThe board is ready.\n\n- Ship the slice\n- Update the board\n\n```sh\ngit status --short\n```\n\nSources: README.md, ARCHITECTURE.md"
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

        let parsed = RenderDocument::parse_assistant_response(response).expect("repaired response");
        assert_eq!(parsed.blocks.len(), 2);
        assert_eq!(parsed.to_plain_text(), "Hello.\n\nSources: README.md");
    }

    #[test]
    fn repairs_structured_responses_when_render_types_are_missing() {
        let response = r#"{
  "blocks": [
    {"type": "paragraph", "text": "Hello."}
  ]
}"#;

        let parsed = RenderDocument::parse_assistant_response(response).expect("repaired response");
        assert_eq!(parsed.blocks.len(), 1);
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
        assert!(prompt.contains("heading, paragraph, bullet_list, code_block, citations"));
        assert!(prompt.contains("Include exactly one `citations` block"));
        assert!(prompt.contains("Return exactly one complete JSON object"));
        assert!(prompt.contains("Do not emit partial blocks"));
        assert!(prompt.contains("Use `heading` for short section titles"));
        assert!(prompt.contains("Each `bullet_list` item must be a complete standalone point"));
        assert!(prompt.contains("Nested lists are not supported"));
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
    fn extract_http_urls_normalizes_and_deduplicates_urls() {
        assert_eq!(
            extract_http_urls(
                "Read https://example.com/docs, then https://example.com/docs/ and https://api.example.com/v1?q=1."
            ),
            vec![
                "https://example.com/docs".to_string(),
                "https://api.example.com/v1?q=1".to_string(),
            ]
        );
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
        assert!(normalized.contains("```json"));
        assert!(normalized.contains("complete payload"));
        assert!(normalized.contains("The payload itself ended after"));
        assert!(normalized.contains("\"blocks\""));
        assert!(normalized.contains("```text"));
    }

    #[test]
    fn assistant_response_schema_avoids_conditional_keywords_for_openai_compatibility() {
        let schema = assistant_response_json_schema(true);
        let block_properties = schema["properties"]["blocks"]["items"]["properties"]
            .as_object()
            .expect("render block schema properties")
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        let block_required = schema["properties"]["blocks"]["items"]["required"]
            .as_array()
            .expect("render block schema required")
            .iter()
            .filter_map(|value| value.as_str().map(ToString::to_string))
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(schema["type"].as_str(), Some("object"));
        assert_eq!(schema["required"][0].as_str(), Some("render_types"));
        assert_eq!(schema["required"][1].as_str(), Some("blocks"));
        assert!(schema["properties"]["blocks"]["items"]["allOf"].is_null());
        assert_eq!(block_required, block_properties);
        assert_eq!(
            schema["properties"]["blocks"]["items"]["properties"]["text"]["type"][0].as_str(),
            Some("string")
        );
        assert_eq!(
            schema["properties"]["blocks"]["items"]["properties"]["text"]["type"][1].as_str(),
            Some("null")
        );
    }

    #[test]
    fn assistant_response_parser_accepts_nullable_irrelevant_block_fields() {
        let response = r#"{
  "render_types": ["heading", "paragraph"],
  "blocks": [
    {
      "type": "heading",
      "text": "Overview",
      "items": null,
      "language": null,
      "code": null,
      "sources": null
    },
    {
      "type": "paragraph",
      "text": "Still works.",
      "items": null,
      "language": null,
      "code": null,
      "sources": null
    }
  ]
}"#;

        let document = crate::domain::model::RenderDocument::parse_assistant_response(response)
            .expect("nullable fields should still parse");

        assert_eq!(document.to_plain_text(), "**Overview**\nStill works.");
    }
}
