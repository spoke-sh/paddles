use std::path::{Component, Path};

const NON_AUTHORED_WORKSPACE_DIRECTORIES: &[&str] = &[
    ".direnv",
    ".git",
    "build",
    "dist",
    "node_modules",
    "result",
    "target",
];

pub(crate) fn is_authored_workspace_directory(name: &str) -> bool {
    !NON_AUTHORED_WORKSPACE_DIRECTORIES.contains(&name)
}

pub(crate) fn is_authored_workspace_file(path: &str) -> bool {
    if path.is_empty() || path.ends_with('/') {
        return false;
    }

    let candidate = Path::new(path);
    if candidate
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none()
    {
        return false;
    }

    candidate.components().all(|component| match component {
        Component::CurDir => true,
        Component::Normal(part) => part
            .to_str()
            .map(is_authored_workspace_directory)
            .unwrap_or(false),
        Component::ParentDir | Component::Prefix(_) | Component::RootDir => false,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn authored_workspace_files_exclude_vendor_and_generated_directories() {
        assert!(!super::is_authored_workspace_file(
            "apps/docs/node_modules/playwright-core/lib/compare.js"
        ));
        assert!(!super::is_authored_workspace_file(
            "apps/web/dist/assets/index.js"
        ));
        assert!(!super::is_authored_workspace_file("result/bin/paddles"));
    }

    #[test]
    fn authored_workspace_files_keep_real_repo_sources() {
        assert!(super::is_authored_workspace_file("src/application/mod.rs"));
        assert!(super::is_authored_workspace_file(
            "apps/web/src/runtime-app.tsx"
        ));
        assert!(super::is_authored_workspace_file("Cargo.lock"));
    }
}
