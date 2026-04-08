use crate::infrastructure::workspace_paths::WorkspacePathPolicy;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const WORKSPACE_ENTITY_CACHE_ROOT_DIR: &str = "workspace-entity-index";
const WORKSPACE_ENTITY_CACHE_WORKSPACES_DIR: &str = "workspaces";
const WORKSPACE_ENTITY_CACHE_FILE: &str = "inventory.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WorkspaceEntityInventory {
    pub entries: Vec<WorkspaceEntityEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WorkspaceEntityEntry {
    pub path: String,
    pub basename: String,
    pub stem: String,
    pub components: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct WorkspaceEntityFileStamp {
    path: String,
    modified_nanos_since_epoch: u128,
    size_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct WorkspaceEntityCacheDocument {
    file_stamps: Vec<WorkspaceEntityFileStamp>,
    inventory: WorkspaceEntityInventory,
}

pub(crate) struct WorkspaceEntityIndex {
    workspace_root: PathBuf,
    path_policy: WorkspacePathPolicy,
    cache_path: PathBuf,
}

impl WorkspaceEntityIndex {
    pub(crate) fn new(workspace_root: &Path) -> Self {
        Self {
            workspace_root: workspace_root.to_path_buf(),
            path_policy: WorkspacePathPolicy::new(workspace_root),
            cache_path: default_workspace_entity_cache_dir_for_workspace(workspace_root)
                .join(WORKSPACE_ENTITY_CACHE_FILE),
        }
    }

    pub(crate) fn load_or_build(&self) -> Result<WorkspaceEntityInventory> {
        let current_stamps = self.scan_file_stamps()?;
        if let Some(cached) = self.load_cached_document()?
            && cached.file_stamps == current_stamps
        {
            return Ok(cached.inventory);
        }

        let inventory = WorkspaceEntityInventory {
            entries: current_stamps
                .iter()
                .map(|stamp| WorkspaceEntityEntry::from_relative_path(&stamp.path))
                .collect(),
        };
        self.persist_cached_document(&WorkspaceEntityCacheDocument {
            file_stamps: current_stamps,
            inventory: inventory.clone(),
        })?;
        Ok(inventory)
    }

    #[cfg(test)]
    pub(crate) fn cache_path(&self) -> &Path {
        &self.cache_path
    }

    fn scan_file_stamps(&self) -> Result<Vec<WorkspaceEntityFileStamp>> {
        let mut builder = WalkBuilder::new(&self.workspace_root);
        builder.hidden(false);
        builder.ignore(false);
        builder.git_ignore(false);
        builder.git_global(false);
        builder.git_exclude(false);
        builder.follow_links(false);

        let mut file_stamps = Vec::new();
        for result in builder.build() {
            let entry = result.context("walk authored workspace inventory")?;
            if !entry
                .file_type()
                .is_some_and(|file_type| file_type.is_file())
            {
                continue;
            }

            let relative_path = entry
                .path()
                .strip_prefix(&self.workspace_root)
                .unwrap_or_else(|_| entry.path())
                .to_string_lossy()
                .replace('\\', "/");
            if !self.path_policy.allows_relative_file(&relative_path) {
                continue;
            }

            let metadata = fs::metadata(entry.path()).with_context(|| {
                format!("read workspace metadata for {}", entry.path().display())
            })?;
            file_stamps.push(WorkspaceEntityFileStamp {
                path: relative_path,
                modified_nanos_since_epoch: metadata
                    .modified()
                    .ok()
                    .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
                    .map(|duration| duration.as_nanos())
                    .unwrap_or(0),
                size_bytes: metadata.len(),
            });
        }
        file_stamps.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(file_stamps)
    }

    fn load_cached_document(&self) -> Result<Option<WorkspaceEntityCacheDocument>> {
        if !self.cache_path.is_file() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&self.cache_path).with_context(|| {
            format!("read workspace entity cache {}", self.cache_path.display())
        })?;
        let document = serde_json::from_str(&contents).with_context(|| {
            format!(
                "parse workspace entity cache document {}",
                self.cache_path.display()
            )
        })?;
        Ok(Some(document))
    }

    fn persist_cached_document(&self, document: &WorkspaceEntityCacheDocument) -> Result<()> {
        if let Some(parent) = self.cache_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("create workspace entity cache dir {}", parent.display())
            })?;
        }
        let payload =
            serde_json::to_vec_pretty(document).context("serialize workspace entity inventory")?;
        fs::write(&self.cache_path, payload)
            .with_context(|| format!("write workspace entity cache {}", self.cache_path.display()))
    }
}

impl WorkspaceEntityEntry {
    fn from_relative_path(path: &str) -> Self {
        let candidate = Path::new(path);
        let basename = candidate
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(path)
            .to_string();
        let stem = candidate
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or(&basename)
            .to_string();
        let components = candidate
            .components()
            .filter_map(|component| component.as_os_str().to_str().map(str::to_string))
            .collect();

        Self {
            path: path.to_string(),
            basename,
            stem,
            components,
        }
    }
}

fn default_workspace_entity_cache_dir_for_workspace(workspace_root: &Path) -> PathBuf {
    let cache_dir = default_workspace_entity_cache_root()
        .join(WORKSPACE_ENTITY_CACHE_WORKSPACES_DIR)
        .join(workspace_cache_leaf(
            &workspace_root
                .canonicalize()
                .unwrap_or_else(|_| workspace_root.to_path_buf()),
        ));
    let _ = fs::create_dir_all(&cache_dir);
    cache_dir
}

fn default_workspace_entity_cache_root() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("", "", "paddles") {
        return project_dirs
            .cache_dir()
            .join(WORKSPACE_ENTITY_CACHE_ROOT_DIR);
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".cache")
            .join("paddles")
            .join(WORKSPACE_ENTITY_CACHE_ROOT_DIR);
    }

    PathBuf::from(".paddles-cache").join(WORKSPACE_ENTITY_CACHE_ROOT_DIR)
}

fn workspace_cache_leaf(workspace_root: &Path) -> String {
    let workspace_name = workspace_root
        .file_name()
        .and_then(|segment| segment.to_str())
        .map(sanitize_component)
        .filter(|segment| !segment.is_empty())
        .unwrap_or_else(|| "workspace".to_string());
    format!(
        "{}-{:016x}",
        workspace_name,
        stable_workspace_hash(workspace_root)
    )
}

fn sanitize_component(component: &str) -> String {
    component
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn stable_workspace_hash(path: &Path) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in path.as_os_str().to_string_lossy().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::WorkspaceEntityIndex;
    use std::fs;

    #[test]
    fn resolver_inventory_respects_workspace_boundary() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("src")).expect("create src");
        fs::create_dir_all(workspace.path().join("apps/web/node_modules")).expect("create vendor");
        fs::create_dir_all(workspace.path().join("apps/docs/.docusaurus"))
            .expect("create generated");
        fs::write(
            workspace.path().join(".gitignore"),
            "/ignored-output/\n/apps/docs/.docusaurus/\n",
        )
        .expect("write gitignore");
        fs::create_dir_all(workspace.path().join("ignored-output")).expect("create ignored");
        fs::write(workspace.path().join("src/main.rs"), "fn main() {}\n").expect("write source");
        fs::write(
            workspace.path().join("apps/web/node_modules/react.js"),
            "module.exports = {};",
        )
        .expect("write vendor");
        fs::write(
            workspace.path().join("apps/docs/.docusaurus/routes.js"),
            "export default [];",
        )
        .expect("write generated");
        fs::write(workspace.path().join("ignored-output/cache.txt"), "ignored")
            .expect("write ignored");

        let index = WorkspaceEntityIndex::new(workspace.path());
        let inventory = index.load_or_build().expect("inventory");
        let paths = inventory
            .entries
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(paths, vec!["src/main.rs"]);
    }

    #[test]
    fn workspace_entity_index_cache_refreshes_after_workspace_changes() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("src")).expect("create src");
        fs::write(workspace.path().join("src/lib.rs"), "pub fn first() {}\n")
            .expect("write first file");

        let index = WorkspaceEntityIndex::new(workspace.path());
        let first_inventory = index.load_or_build().expect("first inventory");
        assert_eq!(first_inventory.entries.len(), 1);
        assert!(
            index.cache_path().is_file(),
            "loading the inventory should persist a machine-managed cache file"
        );

        fs::write(
            workspace.path().join("src/second.rs"),
            "pub fn second() {}\n",
        )
        .expect("write second file");

        let refreshed_inventory = index.load_or_build().expect("refreshed inventory");
        let refreshed_paths = refreshed_inventory
            .entries
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(refreshed_paths, vec!["src/lib.rs", "src/second.rs"]);
    }
}
