use crate::domain::ports::{
    InterpretationContext, InterpretationDecisionFramework, InterpretationDocument,
    OperatorMemoryDocument,
};
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

fn build_interpretation_context_from_documents(
    _user_prompt: &str,
    workspace_root: &Path,
    documents: &[OperatorMemoryDocument],
) -> InterpretationContext {
    if documents.is_empty() {
        return InterpretationContext::default();
    }

    let interpretation_documents = documents
        .iter()
        .take(MAX_INTERPRETATION_DOCS)
        .map(|document| InterpretationDocument {
            source: display_path(workspace_root, &document.path),
            excerpt: fallback_excerpt(&document.contents),
        })
        .collect::<Vec<_>>();

    InterpretationContext {
        summary: format!(
            "Fallback interpretation context assembled from {} AGENTS-rooted guidance document(s). Model-guided interpretation was unavailable for this turn.",
            interpretation_documents.len(),
        ),
        documents: interpretation_documents,
        tool_hints: Vec::new(),
        decision_framework: InterpretationDecisionFramework::default(),
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

fn fallback_excerpt(contents: &str) -> String {
    let excerpt = contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(MAX_INTERPRETATION_DOCS)
        .take(8)
        .collect::<Vec<_>>()
        .join("\n");
    trim_memory_contents(&excerpt)
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
    fn interpretation_context_fallback_uses_agents_rooted_documents_only() {
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
                .summary
                .contains("Fallback interpretation context assembled")
        );
        assert_eq!(interpretation.documents.len(), 1);
        assert_eq!(interpretation.documents[0].source, "AGENTS.md");
        assert!(
            interpretation.documents[0]
                .excerpt
                .contains("keel mission next")
        );
        assert!(interpretation.tool_hints.is_empty());
        assert!(interpretation.decision_framework.procedures.is_empty());
    }
}
