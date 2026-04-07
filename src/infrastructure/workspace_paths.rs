use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Component, Path, PathBuf};

const ALWAYS_NON_AUTHORED_WORKSPACE_DIRECTORIES: &[&str] = &[".git"];

const FALLBACK_NON_AUTHORED_WORKSPACE_DIRECTORIES: &[&str] = &[
    ".direnv",
    ".docusaurus",
    ".sift",
    ".turbo",
    "build",
    "dist",
    "node_modules",
    "result",
    "target",
];

pub(crate) struct WorkspacePathPolicy {
    workspace_root: PathBuf,
    gitignore: Option<Gitignore>,
}

impl WorkspacePathPolicy {
    pub(crate) fn new(workspace_root: &Path) -> Self {
        Self {
            workspace_root: workspace_root.to_path_buf(),
            gitignore: load_workspace_gitignore(workspace_root),
        }
    }

    pub(crate) fn allows_relative_directory(&self, path: &str) -> bool {
        let Some(candidate) = self.normalize_relative_path(path) else {
            return false;
        };
        self.components_are_authored(&candidate) && !self.is_gitignored(&candidate, true)
    }

    pub(crate) fn allows_relative_file(&self, path: &str) -> bool {
        if path.is_empty() || path.ends_with('/') {
            return false;
        }

        let Some(candidate) = self.normalize_relative_path(path) else {
            return false;
        };
        if candidate
            .extension()
            .and_then(|extension| extension.to_str())
            .is_none()
        {
            return false;
        }

        self.components_are_authored(&candidate) && !self.is_gitignored(&candidate, false)
    }

    fn normalize_relative_path(&self, path: &str) -> Option<PathBuf> {
        let requested = Path::new(path);
        let relative = if requested.is_absolute() {
            requested
                .strip_prefix(&self.workspace_root)
                .ok()?
                .to_path_buf()
        } else {
            requested.to_path_buf()
        };

        relative
            .components()
            .all(|component| {
                !matches!(
                    component,
                    Component::ParentDir | Component::Prefix(_) | Component::RootDir
                )
            })
            .then_some(relative)
    }

    fn components_are_authored(&self, candidate: &Path) -> bool {
        candidate.components().all(|component| match component {
            Component::CurDir => true,
            Component::Normal(part) => part
                .to_str()
                .map(|name| self.directory_name_is_authored(name))
                .unwrap_or(false),
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => false,
        })
    }

    fn directory_name_is_authored(&self, name: &str) -> bool {
        if ALWAYS_NON_AUTHORED_WORKSPACE_DIRECTORIES.contains(&name) {
            return false;
        }

        if self.gitignore.is_some() {
            return true;
        }

        !FALLBACK_NON_AUTHORED_WORKSPACE_DIRECTORIES.contains(&name)
    }

    fn is_gitignored(&self, candidate: &Path, is_dir: bool) -> bool {
        self.gitignore.as_ref().is_some_and(|gitignore| {
            gitignore
                .matched_path_or_any_parents(self.workspace_root.join(candidate), is_dir)
                .is_ignore()
        })
    }
}

fn load_workspace_gitignore(workspace_root: &Path) -> Option<Gitignore> {
    let gitignore_path = workspace_root.join(".gitignore");
    if !gitignore_path.is_file() {
        return None;
    }

    let mut builder = GitignoreBuilder::new(workspace_root);
    builder.add(gitignore_path);
    Some(builder.build().unwrap_or_else(|_| Gitignore::empty()))
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn authored_workspace_files_exclude_vendor_and_generated_directories() {
        let workspace = tempfile::tempdir().expect("workspace");
        let policy = super::WorkspacePathPolicy::new(workspace.path());
        assert!(
            !policy.allows_relative_file("apps/docs/node_modules/playwright-core/lib/compare.js")
        );
        assert!(!policy.allows_relative_file("apps/web/dist/assets/index.js"));
        assert!(!policy.allows_relative_file("result/bin/paddles"));
    }

    #[test]
    fn authored_workspace_files_keep_real_repo_sources() {
        let workspace = tempfile::tempdir().expect("workspace");
        let policy = super::WorkspacePathPolicy::new(workspace.path());
        assert!(policy.allows_relative_file("src/application/mod.rs"));
        assert!(policy.allows_relative_file("apps/web/src/runtime-app.tsx"));
        assert!(policy.allows_relative_file("Cargo.lock"));
    }

    #[test]
    fn authored_workspace_files_respect_repo_gitignore_patterns() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("apps/docs/.docusaurus"))
            .expect("create generated docs dir");
        fs::write(
            workspace.path().join(".gitignore"),
            "/apps/docs/.docusaurus/\n",
        )
        .expect("write gitignore");
        let policy = super::WorkspacePathPolicy::new(workspace.path());

        assert!(!policy.allows_relative_file("apps/docs/.docusaurus/client-modules.js"));
        assert!(!policy.allows_relative_directory("apps/docs/.docusaurus"));
        assert!(policy.allows_relative_file("apps/docs/src/intro.tsx"));
    }
}
