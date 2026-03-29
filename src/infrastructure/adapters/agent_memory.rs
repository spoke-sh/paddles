use crate::domain::ports::{InterpretationContext, InterpretationDocument};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const MEMORY_FILE_NAME: &str = "AGENTS.md";
const USER_MEMORY_RELATIVE_PATH: &str = ".config/paddles/AGENTS.md";
const SYSTEM_MEMORY_PATH: &str = "/etc/paddles/AGENTS.md";
const MAX_MEMORY_FILE_CHARS: usize = 12_000;
const MAX_INTERPRETATION_DOCS: usize = 5;

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
        if self.documents.is_empty() {
            return InterpretationContext::default();
        }

        let mut documents = self
            .documents
            .iter()
            .map(|document| {
                let (excerpt, score) = select_relevant_excerpt(&document.contents, user_prompt);
                (
                    score + interpretation_kind_score(document.kind),
                    InterpretationDocument {
                        source: display_path(workspace_root, &document.path),
                        excerpt,
                    },
                )
            })
            .collect::<Vec<_>>();
        documents.sort_by(|(left_score, left_doc), (right_score, right_doc)| {
            right_score
                .cmp(left_score)
                .then_with(|| left_doc.source.cmp(&right_doc.source))
        });
        let documents = documents
            .into_iter()
            .take(MAX_INTERPRETATION_DOCS)
            .map(|(_, document)| document)
            .collect::<Vec<_>>();

        InterpretationContext {
            summary: format!(
                "Operator interpretation context assembled from {} memory and linked guidance document(s). Use it before choosing recursive workspace actions.",
                documents.len()
            ),
            documents,
        }
    }

    fn load_with_search_paths(start_dir: &Path, search_paths: MemorySearchPaths) -> Self {
        let mut memory = Self::default();
        let mut seen_paths = HashSet::new();
        let mut loaded_agent_docs = Vec::new();

        for path in discover_memory_files(start_dir, &search_paths) {
            match load_memory_document(path.clone(), MemoryDocumentKind::AgentMemory) {
                Ok(Some(document)) => {
                    seen_paths.insert(canonical_memory_path(&document.path));
                    loaded_agent_docs.push(document.clone());
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

        for agent_doc in loaded_agent_docs {
            for linked_path in discover_linked_documents(&agent_doc.path, &agent_doc.contents) {
                let canonical = canonical_memory_path(&linked_path);
                if !seen_paths.insert(canonical) {
                    continue;
                }

                match load_memory_document(linked_path.clone(), MemoryDocumentKind::LinkedGuidance)
                {
                    Ok(Some(document)) => memory.documents.push(document),
                    Ok(None) => {}
                    Err(err) => memory.warnings.push(format!(
                        "Skipped unreadable linked guidance {}: {err}",
                        linked_path.display()
                    )),
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

fn canonical_memory_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn default_user_memory_path() -> Option<PathBuf> {
    env::var_os("HOME").map(|home| PathBuf::from(home).join(USER_MEMORY_RELATIVE_PATH))
}

fn discover_linked_documents(agent_path: &Path, contents: &str) -> Vec<PathBuf> {
    let mut ordered = Vec::new();
    let mut seen = HashSet::new();
    for target in extract_markdown_link_targets(contents)
        .into_iter()
        .chain(extract_backticked_doc_targets(contents))
    {
        let Some(resolved) = resolve_link_target(agent_path, &target) else {
            continue;
        };
        let canonical = canonical_memory_path(&resolved);
        if seen.insert(canonical) {
            ordered.push(resolved);
        }
    }
    ordered
}

fn extract_markdown_link_targets(contents: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut remainder = contents;
    while let Some(start) = remainder.find("](") {
        let candidate = &remainder[start + 2..];
        let Some(end) = candidate.find(')') else {
            break;
        };
        let target = candidate[..end].trim();
        if !target.is_empty() {
            targets.push(target.to_string());
        }
        remainder = &candidate[end + 1..];
    }
    targets
}

fn extract_backticked_doc_targets(contents: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut in_tick = false;
    let mut current = String::new();

    for ch in contents.chars() {
        if ch == '`' {
            if in_tick {
                let trimmed = current.trim();
                if looks_like_local_doc_target(trimmed) {
                    targets.push(trimmed.to_string());
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

    targets
}

fn looks_like_local_doc_target(target: &str) -> bool {
    target.ends_with(".md") || target.starts_with(".keel/")
}

fn resolve_link_target(agent_path: &Path, target: &str) -> Option<PathBuf> {
    if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
    {
        return None;
    }

    let target = target.split('#').next()?.trim();
    if target.is_empty() || !looks_like_local_doc_target(target) {
        return None;
    }

    let candidate = if Path::new(target).is_absolute() {
        PathBuf::from(target)
    } else {
        agent_path.parent()?.join(target)
    };

    if candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
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

#[cfg(test)]
mod tests {
    use super::{AgentMemory, MemorySearchPaths};
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
    fn loads_linked_guidance_from_agents_documents() {
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

        let interpretation = memory.build_interpretation_context("policy", &session_root);
        assert!(interpretation.sources().contains(&"AGENTS.md".to_string()));
        assert!(interpretation.sources().contains(&"POLICY.md".to_string()));
        assert!(interpretation.sources().contains(&"README.md".to_string()));
    }
}
