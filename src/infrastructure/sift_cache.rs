use directories::ProjectDirs;
use std::path::{Path, PathBuf};

const SIFT_CACHE_ROOT_DIR: &str = "sift";
const SIFT_CACHE_WORKSPACES_DIR: &str = "workspaces";

pub fn default_sift_cache_dir_for_workspace(workspace_root: &Path) -> PathBuf {
    default_sift_cache_root()
        .join(SIFT_CACHE_WORKSPACES_DIR)
        .join(workspace_cache_leaf(
            &workspace_root
                .canonicalize()
                .unwrap_or_else(|_| workspace_root.to_path_buf()),
        ))
}

fn default_sift_cache_root() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("", "", "paddles") {
        return project_dirs.cache_dir().join(SIFT_CACHE_ROOT_DIR);
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".cache")
            .join("paddles")
            .join(SIFT_CACHE_ROOT_DIR);
    }

    PathBuf::from(".paddles-cache").join(SIFT_CACHE_ROOT_DIR)
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
    use super::default_sift_cache_dir_for_workspace;

    #[test]
    fn default_sift_cache_dir_targets_machine_cache_outside_workspace() {
        let workspace = tempfile::tempdir().expect("workspace tempdir");
        let cache_dir = default_sift_cache_dir_for_workspace(workspace.path());

        assert!(
            !cache_dir.starts_with(workspace.path()),
            "sift cache should not live inside the searchable workspace"
        );
        assert!(
            cache_dir.to_string_lossy().contains("paddles")
                && cache_dir.to_string_lossy().contains("sift")
                && cache_dir.to_string_lossy().contains("workspaces"),
            "sift cache should live under machine-managed paddles cache state"
        );
    }

    #[test]
    fn default_sift_cache_dir_is_stable_for_the_same_workspace() {
        let workspace = tempfile::tempdir().expect("workspace tempdir");

        assert_eq!(
            default_sift_cache_dir_for_workspace(workspace.path()),
            default_sift_cache_dir_for_workspace(workspace.path())
        );
    }

    #[test]
    fn default_sift_cache_dir_distinguishes_different_workspaces() {
        let first = tempfile::tempdir().expect("first tempdir");
        let second = tempfile::tempdir().expect("second tempdir");

        assert_ne!(
            default_sift_cache_dir_for_workspace(first.path()),
            default_sift_cache_dir_for_workspace(second.path())
        );
    }
}
