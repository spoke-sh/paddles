pub const PLANNER_ACTION_SCHEMA_BEGIN: &str = "<!-- BEGIN PLANNER_ACTION_SCHEMA -->";
pub const PLANNER_ACTION_SCHEMA_END: &str = "<!-- END PLANNER_ACTION_SCHEMA -->";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlannerActionSchemaVariant {
    Initial,
    Recursive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlannerActionSchemaEntry {
    pub action: &'static str,
    pub json_example: &'static str,
    pub required_fields: &'static [&'static str],
}

pub fn planner_action_schema_entries(
    variant: PlannerActionSchemaVariant,
) -> &'static [PlannerActionSchemaEntry] {
    match variant {
        PlannerActionSchemaVariant::Initial => INITIAL_ACTION_SCHEMA,
        PlannerActionSchemaVariant::Recursive => RECURSIVE_ACTION_SCHEMA,
    }
}

pub fn render_planner_action_schema(variant: PlannerActionSchemaVariant) -> String {
    let mut rendered = format!(
        "## Planner Action Schema\n\
{PLANNER_ACTION_SCHEMA_BEGIN}\n\
Variant: {}\n\
\n\
You must respond with exactly one complete JSON object selecting the next bounded action.\n\
The first key must be `action`.\n\
\n\
Available actions:\n",
        variant.label()
    );

    for entry in planner_action_schema_entries(variant) {
        rendered.push_str("- ");
        rendered.push_str(entry.json_example);
        rendered.push('\n');
        rendered.push_str("  Required fields: ");
        rendered.push_str(&entry.required_fields.join(", "));
        rendered.push('\n');
    }

    rendered.push_str("\nRules:\n");
    for rule in SHARED_ACTION_SCHEMA_RULES {
        rendered.push_str("- ");
        rendered.push_str(rule);
        rendered.push('\n');
    }
    for rule in variant.rules() {
        rendered.push_str("- ");
        rendered.push_str(rule);
        rendered.push('\n');
    }
    rendered.push_str(PLANNER_ACTION_SCHEMA_END);
    rendered
}

impl PlannerActionSchemaVariant {
    fn label(self) -> &'static str {
        match self {
            Self::Initial => "initial routing decision",
            Self::Recursive => "recursive next-action decision",
        }
    }

    fn rules(self) -> &'static [&'static str] {
        match self {
            Self::Initial => INITIAL_ACTION_SCHEMA_RULES,
            Self::Recursive => RECURSIVE_ACTION_SCHEMA_RULES,
        }
    }
}

const SHARED_ACTION_SCHEMA_RULES: &[&str] = &[
    "Return only the JSON object; do not wrap it in markdown fences, prose, or commentary.",
    "Do not invent action names outside the schema.",
    "Use the capability manifest rendered separately as the live source of truth for currently available tools, external capabilities, execution constraints, and completion requirements.",
    "Choose the most specific next workspace action when the turn requires repository work.",
    "Choose retrieval mode and strategy explicitly whenever you select search or refine.",
    "Supported retrievers are `path-fuzzy` and `segment-fuzzy`; omit retrievers when fuzzy structural lookup is not useful.",
    "Use `inspect` only for a single read-only probe; use `shell` for broader governed workspace command execution.",
    "When the user requests a code or file change, use `write_file`, `replace_in_file`, or `apply_patch` to make the edit instead of describing the edit for the user to apply manually.",
    "Semantic workspace actions are read-only evidence actions for definitions, references, symbols, hover text, and diagnostics.",
    "Use `external_capability` only when the separate capability manifest exposes a matching external capability for the current turn.",
];

const INITIAL_ACTION_SCHEMA_RULES: &[&str] = &[
    "Every initial routing reply must include top-level `edit` and `candidate_files` fields.",
    "`edit` must be `yes` when the user is clearly asking for a code or file edit; otherwise return `no`.",
    "`candidate_files` must list up to 3 plausible relative file paths to inspect or edit first; use `[]` only when `edit` is `no`.",
    "For `answer`, put the user-facing reply in `answer` and keep `rationale` as the planner-only reason for selecting it.",
    "Answer or stop as soon as you have sufficient evidence; do not spend remaining budget on redundant searches.",
];

const RECURSIVE_ACTION_SCHEMA_RULES: &[&str] = &[
    "Every recursive action reply must include top-level `edit` and `candidate_files` fields.",
    "Stop as soon as you have enough evidence to answer; if stopping with the final user-facing answer, include it in `answer` and keep `rationale` for planner-only control reasoning.",
    "Use `refine` when an earlier search needs a sharper query.",
    "Use `branch` when the investigation should split into multiple bounded subqueries.",
];

const INITIAL_ACTION_SCHEMA: &[PlannerActionSchemaEntry] = &[
    PlannerActionSchemaEntry {
        action: "answer",
        json_example: r#"{"action":"answer","answer":"...","edit":"no","candidate_files":[],"rationale":"..."}"#,
        required_fields: &["action", "answer", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "search",
        json_example: r#"{"action":"search","query":"...","mode":"linear|graph","strategy":"bm25|vector","retrievers":["path-fuzzy","segment-fuzzy"],"intent":"optional","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "query",
            "mode",
            "strategy",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "list_files",
        json_example: r#"{"action":"list_files","pattern":"optional substring","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "read",
        json_example: r#"{"action":"read","path":"relative/path","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "path", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "inspect",
        json_example: r#"{"action":"inspect","command":"read-only shell command","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "command", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "shell",
        json_example: r#"{"action":"shell","command":"workspace shell command","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "command", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "diff",
        json_example: r#"{"action":"diff","path":"optional relative/path","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "write_file",
        json_example: r#"{"action":"write_file","path":"relative/path","content":"full file contents","edit":"yes","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "path",
            "content",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "replace_in_file",
        json_example: r#"{"action":"replace_in_file","path":"relative/path","old":"exact old text","new":"replacement text","replace_all":false,"edit":"yes","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "path",
            "old",
            "new",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "apply_patch",
        json_example: r#"{"action":"apply_patch","patch":"unified diff text","edit":"yes","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "patch", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "semantic_definitions",
        json_example: r#"{"action":"semantic_definitions","path":"relative/path","position":{"line":1,"character":0},"edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "path",
            "position",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "semantic_references",
        json_example: r#"{"action":"semantic_references","path":"relative/path","position":{"line":1,"character":0},"edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "path",
            "position",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "semantic_symbols",
        json_example: r#"{"action":"semantic_symbols","path":"relative/path","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "path", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "semantic_hover",
        json_example: r#"{"action":"semantic_hover","path":"relative/path","position":{"line":1,"character":0},"edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "path",
            "position",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "semantic_diagnostics",
        json_example: r#"{"action":"semantic_diagnostics","path":"optional relative/path","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "external_capability",
        json_example: r#"{"action":"external_capability","capability_id":"web.search|mcp.tool|connector.app_action","purpose":"why this external fabric is needed","payload":null,"edit":"no","candidate_files":[],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "capability_id",
            "purpose",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "refine",
        json_example: r#"{"action":"refine","query":"...","mode":"linear|graph","strategy":"bm25|vector","retrievers":["path-fuzzy","segment-fuzzy"],"edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "query",
            "mode",
            "strategy",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "branch",
        json_example: r#"{"action":"branch","branches":["...","..."],"edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "branches", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "stop",
        json_example: r#"{"action":"stop","reason":"...","edit":"no","candidate_files":[],"rationale":"..."}"#,
        required_fields: &["action", "reason", "edit", "candidate_files", "rationale"],
    },
];

const RECURSIVE_ACTION_SCHEMA: &[PlannerActionSchemaEntry] = &[
    PlannerActionSchemaEntry {
        action: "search",
        json_example: r#"{"action":"search","query":"...","mode":"linear|graph","strategy":"bm25|vector","retrievers":["path-fuzzy","segment-fuzzy"],"intent":"optional","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "query",
            "mode",
            "strategy",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "list_files",
        json_example: r#"{"action":"list_files","pattern":"optional substring","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "read",
        json_example: r#"{"action":"read","path":"relative/path","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "path", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "inspect",
        json_example: r#"{"action":"inspect","command":"read-only shell command","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "command", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "shell",
        json_example: r#"{"action":"shell","command":"workspace shell command","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "command", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "diff",
        json_example: r#"{"action":"diff","path":"optional relative/path","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "write_file",
        json_example: r#"{"action":"write_file","path":"relative/path","content":"full file contents","edit":"yes","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "path",
            "content",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "replace_in_file",
        json_example: r#"{"action":"replace_in_file","path":"relative/path","old":"exact old text","new":"replacement text","replace_all":false,"edit":"yes","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "path",
            "old",
            "new",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "apply_patch",
        json_example: r#"{"action":"apply_patch","patch":"unified diff text","edit":"yes","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "patch", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "semantic_definitions",
        json_example: r#"{"action":"semantic_definitions","path":"relative/path","position":{"line":1,"character":0},"edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "path",
            "position",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "semantic_references",
        json_example: r#"{"action":"semantic_references","path":"relative/path","position":{"line":1,"character":0},"edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "path",
            "position",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "semantic_symbols",
        json_example: r#"{"action":"semantic_symbols","path":"relative/path","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "path", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "semantic_hover",
        json_example: r#"{"action":"semantic_hover","path":"relative/path","position":{"line":1,"character":0},"edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "path",
            "position",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "semantic_diagnostics",
        json_example: r#"{"action":"semantic_diagnostics","path":"optional relative/path","edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "external_capability",
        json_example: r#"{"action":"external_capability","capability_id":"web.search|mcp.tool|connector.app_action","purpose":"why this external fabric is needed","payload":null,"edit":"no","candidate_files":[],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "capability_id",
            "purpose",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "refine",
        json_example: r#"{"action":"refine","query":"...","mode":"linear|graph","strategy":"bm25|vector","retrievers":["path-fuzzy","segment-fuzzy"],"edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &[
            "action",
            "query",
            "mode",
            "strategy",
            "edit",
            "candidate_files",
            "rationale",
        ],
    },
    PlannerActionSchemaEntry {
        action: "branch",
        json_example: r#"{"action":"branch","branches":["...","..."],"edit":"yes|no","candidate_files":["path1","path2","path3"],"rationale":"..."}"#,
        required_fields: &["action", "branches", "edit", "candidate_files", "rationale"],
    },
    PlannerActionSchemaEntry {
        action: "stop",
        json_example: r#"{"action":"stop","reason":"...","answer":"optional direct reply when ending immediately","edit":"no","candidate_files":[],"rationale":"..."}"#,
        required_fields: &["action", "reason", "edit", "candidate_files", "rationale"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn schema_actions(variant: PlannerActionSchemaVariant) -> Vec<&'static str> {
        planner_action_schema_entries(variant)
            .iter()
            .map(|entry| entry.action)
            .collect()
    }

    #[test]
    fn renders_initial_schema_with_control_workspace_semantic_and_external_actions() {
        let actions = schema_actions(PlannerActionSchemaVariant::Initial);
        let rendered = render_planner_action_schema(PlannerActionSchemaVariant::Initial);

        for action in [
            "answer",
            "search",
            "apply_patch",
            "semantic_definitions",
            "semantic_references",
            "semantic_symbols",
            "semantic_hover",
            "semantic_diagnostics",
            "external_capability",
            "refine",
            "branch",
            "stop",
        ] {
            assert!(actions.contains(&action), "missing action {action}");
            assert!(
                rendered.contains(&format!("\"action\":\"{action}\"")),
                "rendered schema missing JSON example for {action}"
            );
        }

        assert!(rendered.contains("edit"));
        assert!(rendered.contains("candidate_files"));
        assert!(rendered.contains("rationale"));
    }

    #[test]
    fn renders_recursive_schema_without_direct_answer_action() {
        let actions = schema_actions(PlannerActionSchemaVariant::Recursive);
        let rendered = render_planner_action_schema(PlannerActionSchemaVariant::Recursive);

        assert!(!actions.contains(&"answer"));
        assert!(!rendered.contains("\"action\":\"answer\""));

        for action in [
            "search",
            "read",
            "apply_patch",
            "semantic_definitions",
            "external_capability",
            "refine",
            "branch",
            "stop",
        ] {
            assert!(actions.contains(&action), "missing action {action}");
            assert!(
                rendered.contains(&format!("\"action\":\"{action}\"")),
                "rendered schema missing JSON example for {action}"
            );
        }

        assert!(rendered.contains("optional direct reply when ending immediately"));
    }

    #[test]
    fn schema_contract_records_required_fields() {
        let initial_entries = planner_action_schema_entries(PlannerActionSchemaVariant::Initial);
        let answer = initial_entries
            .iter()
            .find(|entry| entry.action == "answer")
            .expect("answer schema entry");
        assert_eq!(
            answer.required_fields,
            &["action", "answer", "edit", "candidate_files", "rationale"]
        );

        let apply_patch = initial_entries
            .iter()
            .find(|entry| entry.action == "apply_patch")
            .expect("apply_patch schema entry");
        assert!(apply_patch.required_fields.contains(&"patch"));

        let external = initial_entries
            .iter()
            .find(|entry| entry.action == "external_capability")
            .expect("external_capability schema entry");
        assert!(external.required_fields.contains(&"capability_id"));
        assert!(external.required_fields.contains(&"purpose"));
    }

    #[test]
    fn schema_renderer_leaves_execution_contract_separate() {
        let rendered = render_planner_action_schema(PlannerActionSchemaVariant::Initial);

        assert!(rendered.contains("capability manifest"));
        assert!(!rendered.contains("Capability Manifest:"));
        assert!(!rendered.contains("Completion Contract:"));
    }
}
