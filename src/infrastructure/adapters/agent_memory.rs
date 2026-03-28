use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const MEMORY_FILE_NAME: &str = "AGENTS.md";
const USER_MEMORY_RELATIVE_PATH: &str = ".config/paddles/AGENTS.md";
const SYSTEM_MEMORY_PATH: &str = "/etc/paddles/AGENTS.md";
const MAX_MEMORY_FILE_CHARS: usize = 12_000;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AgentMemory {
    documents: Vec<MemoryDocument>,
    warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MemoryDocument {
    path: PathBuf,
    contents: String,
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
        if self.documents.is_empty() {
            return "No AGENTS.md memory files were loaded.".to_string();
        }

        let mut sections = vec![
            "Loaded AGENTS.md memory files. Apply them from top to bottom; later files are more specific and override earlier guidance.".to_string(),
        ];

        for document in &self.documents {
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

    fn load_with_search_paths(start_dir: &Path, search_paths: MemorySearchPaths) -> Self {
        let mut memory = Self::default();

        for path in discover_memory_files(start_dir, &search_paths) {
            match fs::read_to_string(&path) {
                Ok(contents) => {
                    let trimmed = contents.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    memory.documents.push(MemoryDocument {
                        path,
                        contents: trimmed.to_string(),
                    });
                }
                Err(err) => memory.warnings.push(format!(
                    "Skipped unreadable memory file {}: {err}",
                    path.display()
                )),
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
}
