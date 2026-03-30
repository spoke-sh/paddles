use crate::domain::ports::{
    InterpretationContext, InterpretationDecisionFramework, InterpretationDocument,
    InterpretationProcedure, InterpretationProcedureStep, InterpretationToolHint,
    OperatorMemoryDocument, WorkspaceAction,
};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const MEMORY_FILE_NAME: &str = "AGENTS.md";
const USER_MEMORY_RELATIVE_PATH: &str = ".config/paddles/AGENTS.md";
const SYSTEM_MEMORY_PATH: &str = "/etc/paddles/AGENTS.md";
const MAX_MEMORY_FILE_CHARS: usize = 12_000;
const MAX_INTERPRETATION_DOCS: usize = 5;
const MAX_INTERPRETATION_TOOL_HINTS: usize = 6;
const MAX_INTERPRETATION_PROCEDURES: usize = 4;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AgentMemory {
    documents: Vec<MemoryDocument>,
    warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MemoryDocument {
    path: PathBuf,
    contents: String,
    kind: MemoryDocumentKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemoryDocumentKind {
    AgentMemory,
    LinkedGuidance,
}

#[derive(Clone, Debug, Default)]
struct MemorySearchPaths {
    system: Option<PathBuf>,
    user: Option<PathBuf>,
}

impl MemorySearchPaths {
    fn defaults() -> Self {
        Self {
            system: Some(PathBuf::from(SYSTEM_MEMORY_PATH)),
            user: default_user_memory_path(),
        }
    }
}

impl AgentMemory {
    pub(crate) fn load(start_dir: &Path) -> Self {
        Self::load_with_search_paths(start_dir, MemorySearchPaths::defaults())
    }

    pub(crate) fn render_for_prompt(&self) -> String {
        let prompt_docs = self
            .documents
            .iter()
            .filter(|document| document.kind == MemoryDocumentKind::AgentMemory)
            .collect::<Vec<_>>();
        if prompt_docs.is_empty() {
            return "No AGENTS.md memory files were loaded.".to_string();
        }

        let mut sections = vec![
            "Loaded AGENTS.md memory files. Apply them from top to bottom; later files are more specific and override earlier guidance.".to_string(),
        ];

        for document in prompt_docs {
            sections.push(format!(
                "--- {} ---\n{}",
                document.path.display(),
                trim_memory_contents(&document.contents)
            ));
        }

        sections.join("\n\n")
    }

    pub(crate) fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub(crate) fn build_interpretation_context(
        &self,
        user_prompt: &str,
        workspace_root: &Path,
    ) -> InterpretationContext {
        self.build_interpretation_context_from_documents(
            user_prompt,
            workspace_root,
            &self.operator_memory_documents(workspace_root),
        )
    }

    pub(crate) fn operator_memory_documents(
        &self,
        workspace_root: &Path,
    ) -> Vec<OperatorMemoryDocument> {
        self.documents
            .iter()
            .filter(|document| document.kind == MemoryDocumentKind::AgentMemory)
            .map(|document| OperatorMemoryDocument {
                path: document.path.clone(),
                source: display_path(workspace_root, &document.path),
                contents: document.contents.clone(),
            })
            .collect()
    }

    pub(crate) fn build_interpretation_context_from_documents(
        &self,
        user_prompt: &str,
        workspace_root: &Path,
        documents: &[OperatorMemoryDocument],
    ) -> InterpretationContext {
        build_interpretation_context_from_documents(user_prompt, workspace_root, documents)
    }

    fn load_with_search_paths(start_dir: &Path, search_paths: MemorySearchPaths) -> Self {
        let mut memory = Self::default();

        for path in discover_memory_files(start_dir, &search_paths) {
            match load_memory_document(path.clone(), MemoryDocumentKind::AgentMemory) {
                Ok(Some(document)) => {
                    memory.documents.push(document);
                }
                Ok(None) => {}
                Err(err) => {
                    memory.warnings.push(format!(
                        "Skipped unreadable memory file {}: {err}",
                        path.display()
                    ));
                }
            }
        }

        memory
    }

    #[cfg(test)]
    fn document_paths(&self) -> Vec<PathBuf> {
        self.documents
            .iter()
            .map(|document| document.path.clone())
            .collect()
    }
}

fn interpretation_kind_score(kind: MemoryDocumentKind) -> usize {
    match kind {
        MemoryDocumentKind::AgentMemory => 10,
        MemoryDocumentKind::LinkedGuidance => 0,
    }
}

fn build_interpretation_context_from_documents(
    user_prompt: &str,
    workspace_root: &Path,
    documents: &[OperatorMemoryDocument],
) -> InterpretationContext {
    if documents.is_empty() {
        return InterpretationContext::default();
    }

    let mut scored_documents = documents
        .iter()
        .map(|document| {
            let (excerpt, score) = select_relevant_excerpt(&document.contents, user_prompt);
            let kind_score = if Path::new(&document.source)
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name == MEMORY_FILE_NAME)
            {
                interpretation_kind_score(MemoryDocumentKind::AgentMemory)
            } else {
                interpretation_kind_score(MemoryDocumentKind::LinkedGuidance)
            };
            (
                score + kind_score,
                document,
                InterpretationDocument {
                    source: display_path(workspace_root, &document.path),
                    excerpt,
                },
            )
        })
        .collect::<Vec<_>>();
    scored_documents.sort_by(|(left_score, _, left_doc), (right_score, _, right_doc)| {
        right_score
            .cmp(left_score)
            .then_with(|| left_doc.source.cmp(&right_doc.source))
    });
    let selected_documents = scored_documents
        .into_iter()
        .take(MAX_INTERPRETATION_DOCS)
        .collect::<Vec<_>>();
    let interpretation_documents = selected_documents
        .iter()
        .map(|(_, _, document)| document.clone())
        .collect::<Vec<_>>();
    let tool_hints = select_relevant_tool_hints(user_prompt, workspace_root, &selected_documents);
    let decision_framework =
        select_decision_framework(user_prompt, workspace_root, &selected_documents);

    InterpretationContext {
        summary: format!(
            "Operator interpretation context assembled from {} memory and model-derived guidance document(s), {} tool hint(s), and {} decision procedure(s). Use it before choosing recursive workspace actions.",
            interpretation_documents.len(),
            tool_hints.len(),
            decision_framework.procedures.len()
        ),
        documents: interpretation_documents,
        tool_hints,
        decision_framework,
    }
}

fn load_memory_document(
    path: PathBuf,
    kind: MemoryDocumentKind,
) -> std::io::Result<Option<MemoryDocument>> {
    let contents = fs::read_to_string(&path)?;
    let trimmed = contents.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    Ok(Some(MemoryDocument {
        path,
        contents: trimmed.to_string(),
        kind,
    }))
}

fn discover_memory_files(start_dir: &Path, search_paths: &MemorySearchPaths) -> Vec<PathBuf> {
    let start_dir = start_dir
        .canonicalize()
        .unwrap_or_else(|_| start_dir.to_path_buf());
    let mut ordered_paths = Vec::new();
    let mut seen_paths = HashSet::new();

    push_if_present(
        &mut ordered_paths,
        &mut seen_paths,
        search_paths.system.clone(),
    );
    push_if_present(
        &mut ordered_paths,
        &mut seen_paths,
        search_paths.user.clone(),
    );

    let mut ancestors = start_dir
        .ancestors()
        .map(Path::to_path_buf)
        .collect::<Vec<_>>();
    ancestors.reverse();
    for directory in ancestors {
        push_if_present(
            &mut ordered_paths,
            &mut seen_paths,
            Some(directory.join(MEMORY_FILE_NAME)),
        );
    }

    ordered_paths
}

fn push_if_present(
    ordered_paths: &mut Vec<PathBuf>,
    seen_paths: &mut HashSet<PathBuf>,
    candidate: Option<PathBuf>,
) {
    let Some(candidate) = candidate else {
        return;
    };
    if !candidate.is_file() {
        return;
    }

    let canonical = candidate
        .canonicalize()
        .unwrap_or_else(|_| candidate.clone());
    if seen_paths.insert(canonical) {
        ordered_paths.push(candidate);
    }
}

fn default_user_memory_path() -> Option<PathBuf> {
    env::var_os("HOME").map(|home| PathBuf::from(home).join(USER_MEMORY_RELATIVE_PATH))
}

pub(crate) fn resolve_guidance_target(base_path: &Path, target: &str) -> Option<PathBuf> {
    if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
    {
        return None;
    }

    let target = target.split('#').next()?.trim();
    if target.is_empty() {
        return None;
    }

    let candidate = if Path::new(target).is_absolute() {
        PathBuf::from(target)
    } else {
        base_path.parent()?.join(target)
    };

    if candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

pub(crate) fn load_guidance_document(
    base_path: &Path,
    target: &str,
    workspace_root: &Path,
) -> std::io::Result<Option<OperatorMemoryDocument>> {
    let Some(resolved) = resolve_guidance_target(base_path, target) else {
        return Ok(None);
    };
    let Some(document) = load_memory_document(resolved, MemoryDocumentKind::LinkedGuidance)? else {
        return Ok(None);
    };

    Ok(Some(OperatorMemoryDocument {
        source: display_path(workspace_root, &document.path),
        path: document.path,
        contents: document.contents,
    }))
}

fn trim_memory_contents(contents: &str) -> String {
    if contents.chars().count() <= MAX_MEMORY_FILE_CHARS {
        return contents.to_string();
    }

    let mut trimmed = contents
        .chars()
        .take(MAX_MEMORY_FILE_CHARS)
        .collect::<String>();
    trimmed.push_str("\n...[truncated]");
    trimmed
}

fn display_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn select_relevant_excerpt(contents: &str, user_prompt: &str) -> (String, usize) {
    let query_terms = prompt_terms(user_prompt);
    let lines = contents.lines().collect::<Vec<_>>();

    let mut excerpt = Vec::new();
    let mut score = 0;
    if !query_terms.is_empty() {
        for (index, line) in lines.iter().enumerate() {
            let normalized = line.to_ascii_lowercase();
            let matched_terms = query_terms
                .iter()
                .filter(|term| normalized.contains(term.as_str()))
                .count();
            if matched_terms > 0 {
                score += matched_terms;
                let start = index.saturating_sub(1);
                let end = usize::min(index + 3, lines.len());
                excerpt.extend(lines[start..end].iter().copied());
                if excerpt.len() >= 8 {
                    break;
                }
            }
        }
    }

    if excerpt.is_empty() {
        excerpt.extend(
            lines
                .into_iter()
                .filter(|line| !line.trim().is_empty())
                .take(8),
        );
    }

    (trim_memory_contents(&excerpt.join("\n")), score)
}

fn prompt_terms(user_prompt: &str) -> Vec<String> {
    let mut terms = user_prompt
        .to_ascii_lowercase()
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '.')
        .filter(|term| term.len() >= 3)
        .filter(|term| {
            !matches!(
                *term,
                "the"
                    | "and"
                    | "with"
                    | "from"
                    | "into"
                    | "that"
                    | "this"
                    | "what"
                    | "when"
                    | "where"
                    | "which"
                    | "would"
                    | "could"
                    | "should"
                    | "about"
                    | "your"
            )
        })
        .map(str::to_string)
        .collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    terms
}

fn select_relevant_tool_hints(
    user_prompt: &str,
    workspace_root: &Path,
    selected_documents: &[(usize, &OperatorMemoryDocument, InterpretationDocument)],
) -> Vec<InterpretationToolHint> {
    let query_terms = prompt_terms(user_prompt);
    let mut deduped = HashMap::<String, (usize, InterpretationToolHint)>::new();

    for (document_score, memory_document, interpretation_document) in selected_documents {
        for (command, note) in extract_command_hints(&memory_document.contents) {
            let Some(action) = tool_hint_action_for_command(&command) else {
                continue;
            };
            let normalized = format!(
                "{}\n{}",
                command.to_ascii_lowercase(),
                note.to_ascii_lowercase()
            );
            let overlap = query_terms
                .iter()
                .filter(|term| normalized.contains(term.as_str()))
                .count();
            let score = document_score.saturating_mul(4)
                + overlap.saturating_mul(6)
                + usize::from(matches!(action, WorkspaceAction::Inspect { .. })) * 2;
            if score == 0 {
                continue;
            }

            let hint = InterpretationToolHint {
                source: display_path(workspace_root, &memory_document.path),
                action,
                note,
            };
            let key = format!("{}::{}", hint.action.label(), hint.action.summary());
            let replace = deduped
                .get(&key)
                .map(|(existing_score, _)| score > *existing_score)
                .unwrap_or(true);
            if replace {
                deduped.insert(key, (score, hint));
            }
        }

        if query_terms.is_empty() {
            for (command, note) in extract_command_hints(&interpretation_document.excerpt) {
                let Some(action) = tool_hint_action_for_command(&command) else {
                    continue;
                };
                let hint = InterpretationToolHint {
                    source: interpretation_document.source.clone(),
                    action,
                    note,
                };
                let key = format!("{}::{}", hint.action.label(), hint.action.summary());
                deduped
                    .entry(key)
                    .or_insert((document_score.saturating_mul(2), hint));
            }
        }
    }

    let mut hints = deduped.into_values().collect::<Vec<_>>();
    hints.sort_by(|(left_score, left_hint), (right_score, right_hint)| {
        right_score
            .cmp(left_score)
            .then_with(|| left_hint.source.cmp(&right_hint.source))
            .then_with(|| left_hint.action.summary().cmp(&right_hint.action.summary()))
    });
    hints
        .into_iter()
        .take(MAX_INTERPRETATION_TOOL_HINTS)
        .map(|(_, hint)| hint)
        .collect()
}

fn select_decision_framework(
    user_prompt: &str,
    workspace_root: &Path,
    selected_documents: &[(usize, &OperatorMemoryDocument, InterpretationDocument)],
) -> InterpretationDecisionFramework {
    let query_terms = prompt_terms(user_prompt);
    let mut procedures = Vec::<(usize, InterpretationProcedure)>::new();

    for (document_score, memory_document, _) in selected_documents {
        for procedure in extract_command_procedures(&memory_document.contents) {
            let normalized = format!(
                "{}\n{}\n{}",
                procedure.label.to_ascii_lowercase(),
                procedure.purpose.to_ascii_lowercase(),
                procedure
                    .steps
                    .iter()
                    .map(|step| format!(
                        "{}\n{}",
                        step.action.summary(),
                        step.note.to_ascii_lowercase()
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            let overlap = query_terms
                .iter()
                .filter(|term| normalized.contains(term.as_str()))
                .count();
            let score = document_score.saturating_mul(4) + overlap.saturating_mul(8);
            if score == 0 {
                continue;
            }

            procedures.push((
                score,
                InterpretationProcedure {
                    source: display_path(workspace_root, &memory_document.path),
                    label: procedure.label,
                    purpose: procedure.purpose,
                    steps: procedure.steps,
                },
            ));
        }
    }

    procedures.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .cmp(left_score)
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.label.cmp(&right.label))
    });

    InterpretationDecisionFramework {
        procedures: procedures
            .into_iter()
            .take(MAX_INTERPRETATION_PROCEDURES)
            .map(|(_, procedure)| procedure)
            .collect(),
    }
}

#[derive(Clone, Debug)]
struct ExtractedProcedure {
    label: String,
    purpose: String,
    steps: Vec<InterpretationProcedureStep>,
}

fn extract_command_procedures(contents: &str) -> Vec<ExtractedProcedure> {
    let mut procedures = Vec::new();
    let mut current_heading = String::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(heading) = extract_heading_label(trimmed) {
            current_heading = heading;
            continue;
        }

        let steps = extract_command_hints(trimmed)
            .into_iter()
            .enumerate()
            .filter_map(|(index, (command, note))| {
                tool_hint_action_for_command(&command).map(|action| InterpretationProcedureStep {
                    index,
                    action,
                    note,
                })
            })
            .collect::<Vec<_>>();

        if steps.len() < 2 {
            continue;
        }

        let label = line_label(trimmed)
            .or_else(|| (!current_heading.is_empty()).then(|| current_heading.clone()))
            .unwrap_or_else(|| "Derived procedure".to_string());
        procedures.push(ExtractedProcedure {
            label,
            purpose: trimmed.to_string(),
            steps,
        });
    }

    procedures
}

fn extract_heading_label(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let trimmed = trimmed.trim_start_matches('#').trim();
    if trimmed.is_empty() || trimmed == line {
        return None;
    }
    Some(trimmed.to_string())
}

fn line_label(line: &str) -> Option<String> {
    let trimmed = line
        .trim_start_matches(|ch: char| matches!(ch, '-' | '*' | '+' | '0'..='9' | '.' | ' '))
        .trim();
    let Some((label, _)) = trimmed.split_once(':') else {
        let fallback = trimmed
            .split(" with ")
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())?;
        return Some(
            fallback
                .trim_matches('*')
                .trim_matches('_')
                .trim()
                .to_string(),
        );
    };
    let label = label.trim().trim_matches('*').trim_matches('_').trim();
    if label.is_empty() {
        None
    } else {
        Some(label.to_string())
    }
}

fn extract_command_hints(contents: &str) -> Vec<(String, String)> {
    let mut hints = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        for snippet in extract_inline_code_spans(trimmed) {
            if looks_like_shell_command(&snippet) {
                hints.push((snippet, trimmed.to_string()));
            }
        }
    }
    hints
}

fn extract_inline_code_spans(line: &str) -> Vec<String> {
    let mut spans = Vec::new();
    let mut in_tick = false;
    let mut current = String::new();

    for ch in line.chars() {
        if ch == '`' {
            if in_tick {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    spans.push(trimmed.to_string());
                }
                current.clear();
            }
            in_tick = !in_tick;
            continue;
        }

        if in_tick {
            current.push(ch);
        }
    }

    spans
}

fn looks_like_shell_command(candidate: &str) -> bool {
    let normalized = candidate.trim();
    [
        "keel ", "git ", "rg ", "find ", "cat ", "sed -n", "head ", "tail ", "pwd", "ls",
    ]
    .iter()
    .any(|prefix| normalized.starts_with(prefix))
}

fn tool_hint_action_for_command(command: &str) -> Option<WorkspaceAction> {
    let normalized = command.trim();
    if normalized.is_empty() || !is_read_only_command(normalized) {
        return None;
    }

    Some(WorkspaceAction::Inspect {
        command: normalized.to_string(),
    })
}

fn is_read_only_command(command: &str) -> bool {
    [
        "keel health",
        "keel flow",
        "keel doctor",
        "keel mission ",
        "keel pulse",
        "keel workshop",
        "keel screen ",
        "keel topology ",
        "keel story show",
        "keel voyage show",
        "keel epic show",
        "keel bearing list",
        "git status",
        "git diff",
        "git log",
        "rg ",
        "ls",
        "find ",
        "cat ",
        "sed -n",
        "head ",
        "tail ",
        "pwd",
    ]
    .iter()
    .any(|prefix| command.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::{AgentMemory, MemorySearchPaths, load_guidance_document};
    use std::fs;

    #[test]
    fn loads_system_user_and_ancestor_memory_in_specificity_order() {
        let sandbox = tempfile::tempdir().expect("sandbox");
        let system_memory = sandbox.path().join("etc/paddles/AGENTS.md");
        let user_memory = sandbox.path().join("home/alex/.config/paddles/AGENTS.md");
        let workspace_root = sandbox.path().join("workspace");
        let project_root = workspace_root.join("project");
        let session_root = project_root.join("app");

        fs::create_dir_all(system_memory.parent().expect("system dir")).expect("system dir");
        fs::create_dir_all(user_memory.parent().expect("user dir")).expect("user dir");
        fs::create_dir_all(&session_root).expect("session root");

        fs::write(&system_memory, "system memory").expect("system memory");
        fs::write(&user_memory, "user memory").expect("user memory");
        fs::write(workspace_root.join("AGENTS.md"), "workspace memory").expect("workspace memory");
        fs::write(project_root.join("AGENTS.md"), "project memory").expect("project memory");
        fs::write(session_root.join("AGENTS.md"), "session memory").expect("session memory");

        let memory = AgentMemory::load_with_search_paths(
            &session_root,
            MemorySearchPaths {
                system: Some(system_memory.clone()),
                user: Some(user_memory.clone()),
            },
        );

        assert_eq!(
            memory.document_paths(),
            vec![
                system_memory,
                user_memory,
                workspace_root.join("AGENTS.md"),
                project_root.join("AGENTS.md"),
                session_root.join("AGENTS.md"),
            ]
        );

        let rendered = memory.render_for_prompt();
        assert!(rendered.contains("system memory"));
        assert!(rendered.contains("session memory"));
    }

    #[test]
    fn agent_memory_roots_only_load_agents_documents() {
        let sandbox = tempfile::tempdir().expect("sandbox");
        let session_root = sandbox.path().join("workspace/project");
        fs::create_dir_all(&session_root).expect("session root");
        fs::write(
            session_root.join("AGENTS.md"),
            "See [Policy](POLICY.md) and `README.md`.",
        )
        .expect("agents");
        fs::write(session_root.join("POLICY.md"), "policy guidance").expect("policy");
        fs::write(session_root.join("README.md"), "readme guidance").expect("readme");

        let memory = AgentMemory::load_with_search_paths(
            &session_root,
            MemorySearchPaths {
                system: None,
                user: None,
            },
        );

        assert_eq!(
            memory.document_paths(),
            vec![session_root.join("AGENTS.md")]
        );
        let operator_memory = memory.operator_memory_documents(&session_root);
        assert_eq!(operator_memory.len(), 1);
        assert_eq!(operator_memory[0].source, "AGENTS.md");
    }

    #[test]
    fn load_guidance_document_resolves_targets_relative_to_source_documents() {
        let sandbox = tempfile::tempdir().expect("sandbox");
        let session_root = sandbox.path().join("workspace/project");
        fs::create_dir_all(&session_root).expect("session root");
        fs::write(session_root.join("AGENTS.md"), "See [Policy](POLICY.md).").expect("agents");
        fs::write(session_root.join("POLICY.md"), "policy guidance").expect("policy");

        let loaded =
            load_guidance_document(&session_root.join("AGENTS.md"), "POLICY.md", &session_root)
                .expect("guidance load")
                .expect("guidance document");

        assert_eq!(loaded.source, "POLICY.md");
        assert_eq!(loaded.contents, "policy guidance");
    }

    #[test]
    fn interpretation_context_extracts_relevant_read_only_tool_hints() {
        let sandbox = tempfile::tempdir().expect("sandbox");
        let session_root = sandbox.path().join("workspace/project");
        fs::create_dir_all(&session_root).expect("session root");
        fs::write(
            session_root.join("AGENTS.md"),
            "Inspect current demand with `keel mission next`, `keel pulse`, and `keel workshop`.\nDo not use `keel story submit <id>` as a read-only probe.",
        )
        .expect("agents");

        let memory = AgentMemory::load_with_search_paths(
            &session_root,
            MemorySearchPaths {
                system: None,
                user: None,
            },
        );

        let interpretation =
            memory.build_interpretation_context("What's next on the keel board?", &session_root);

        assert!(
            interpretation
                .tool_hints
                .iter()
                .any(|hint| hint.action.summary().contains("keel mission next"))
        );
        assert!(
            interpretation
                .tool_hints
                .iter()
                .all(|hint| !hint.action.summary().contains("keel story submit"))
        );
        assert!(
            interpretation
                .decision_framework
                .procedures
                .iter()
                .any(|procedure| {
                    procedure.label.contains("Inspect")
                        && procedure
                            .steps
                            .iter()
                            .any(|step| step.action.summary().contains("keel mission next"))
                })
        );
    }
}
