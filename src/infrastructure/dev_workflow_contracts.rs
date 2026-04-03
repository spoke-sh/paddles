use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_file(path)).unwrap_or_else(|error| {
        panic!("failed to read repo file `{path}`: {error}");
    })
}

#[test]
fn just_quality_runs_frontend_workspace_lint_checks() {
    let justfile = read_repo_file("justfile");
    assert!(
        justfile.contains("frontend-install:\n  npm ci"),
        "justfile should define a root frontend install helper using npm ci",
    );
    let quality_section = justfile
        .split("\n# Run the paddles CLI.")
        .next()
        .and_then(|prefix| prefix.split("\nquality:\n").nth(1))
        .expect("justfile should contain a quality recipe");

    assert!(
        quality_section.contains("just frontend-install"),
        "quality should install frontend workspace dependencies before JS checks",
    );
    assert!(
        quality_section.contains("just frontend-quality"),
        "quality should run frontend workspace lint checks",
    );
}

#[test]
fn just_test_runs_frontend_workspace_test_checks() {
    let justfile = read_repo_file("justfile");
    let test_section = justfile
        .split("\n# Run quality checks.")
        .next()
        .and_then(|prefix| prefix.split("\ntest:\n").nth(1))
        .expect("justfile should contain a test recipe");

    assert!(
        test_section.contains("just frontend-install"),
        "test should install frontend workspace dependencies before JS tests",
    );
    assert!(
        test_section.contains("just frontend-test"),
        "test should run frontend workspace test checks",
    );
    assert!(
        test_section.contains("just frontend-build"),
        "test should build the runtime frontend before browser e2e runs against the Rust server",
    );
    assert!(
        test_section.contains("just frontend-e2e"),
        "test should run browser e2e checks",
    );
}

#[test]
fn pre_commit_governor_runs_repo_quality_and_test_entrypoints() {
    let hook = read_repo_file(".git/hooks/pre-commit");

    assert!(
        hook.contains("just quality || exit 1"),
        "pre-commit should gate commits on the repo quality entrypoint",
    );
    assert!(
        hook.contains("just test || exit 1"),
        "pre-commit should gate commits on the repo test entrypoint, including browser e2e",
    );
    assert!(
        hook.contains("keel health || exit 1"),
        "pre-commit should still run keel health after quality and test checks",
    );
}

#[test]
fn just_paddles_rebuilds_runtime_frontend_before_launching_cli() {
    let justfile = read_repo_file("justfile");
    let paddles_section = justfile
        .split("\n# Standard mission path for verification.")
        .next()
        .and_then(|prefix| {
            prefix
                .split(
                    "\n# Run the paddles CLI. Use --cuda to enable GPU support.\npaddles *args:\n",
                )
                .nth(1)
        })
        .expect("justfile should contain a paddles recipe");

    assert!(
        paddles_section.contains("just frontend-install"),
        "paddles should install frontend workspace dependencies before launching the CLI",
    );
    assert!(
        paddles_section.contains("just frontend-build"),
        "paddles should rebuild the runtime frontend workspace before launching the CLI",
    );
}

#[test]
fn root_workspace_package_defines_shared_scripts_and_workspaces() {
    let package_json = read_repo_file("package.json");
    let package: Value = serde_json::from_str(&package_json).expect("package.json should parse");
    let scripts = package["scripts"]
        .as_object()
        .expect("package.json scripts should be an object");
    let workspaces = package["workspaces"]
        .as_array()
        .expect("package.json workspaces should be an array");

    assert!(
        workspaces
            .iter()
            .any(|entry| entry.as_str() == Some("apps/*")),
        "root package.json should manage apps/* as workspaces",
    );

    assert!(
        scripts.contains_key("lint"),
        "root workspace should define a lint script",
    );
    assert!(
        scripts.contains_key("test"),
        "root workspace should define a test script",
    );
    assert!(
        scripts.contains_key("e2e"),
        "root workspace should define an e2e script",
    );
    assert!(
        scripts.contains_key("build"),
        "root workspace should define a build script",
    );
}

#[test]
fn turbo_config_exists_for_frontend_workspace() {
    assert!(
        repo_file("turbo.json").exists(),
        "repo should define turbo.json for the frontend workspace",
    );
}

#[test]
fn frontend_apps_exist_under_apps_directory() {
    assert!(
        repo_file("apps/docs/package.json").exists(),
        "docs app should live at apps/docs/package.json",
    );
    assert!(
        repo_file("apps/web/package.json").exists(),
        "runtime React app should live at apps/web/package.json",
    );
}

#[test]
fn dev_shell_exposes_node_for_frontend_workspace_checks() {
    let flake = read_repo_file("flake.nix");
    assert!(
        flake.contains("pkgs.nodejs"),
        "dev shell should include nodejs so frontend workspace checks run in nix develop",
    );
    assert!(
        flake.contains("pkgs.chromium"),
        "dev shell should include chromium so browser e2e checks run in nix develop",
    );
}

#[test]
fn root_workspace_lockfile_exists_for_clean_ci_installs() {
    assert!(
        repo_file("package-lock.json").exists(),
        "frontend workspace should commit a root package-lock.json so CI can use npm ci",
    );
}

#[test]
fn frontend_playwright_artifacts_are_gitignored() {
    let gitignore = read_repo_file(".gitignore");

    assert!(
        gitignore.contains("/apps/web/test-results/"),
        "frontend workspace should ignore runtime app Playwright test output",
    );
    assert!(
        gitignore.contains("/apps/docs/test-results/"),
        "frontend workspace should ignore docs Playwright test output",
    );
}

#[test]
fn docs_static_favicon_exists_for_docusaurus_builds() {
    assert!(
        repo_file("apps/docs/static/img/favicon.svg").exists(),
        "docs app should ship the favicon asset referenced by docusaurus.config.ts",
    );
}

#[test]
fn docs_app_defines_browser_e2e_verification() {
    let package_json = read_repo_file("apps/docs/package.json");
    let package: Value =
        serde_json::from_str(&package_json).expect("apps/docs/package.json should parse");
    let scripts = package["scripts"]
        .as_object()
        .expect("apps/docs/package.json scripts should be an object");

    assert!(
        scripts.contains_key("e2e"),
        "docs app should define a browser e2e script in the shared workspace",
    );
    assert!(
        repo_file("apps/docs/playwright.config.mjs").exists(),
        "docs app should define a Playwright config for browser verification",
    );
    assert!(
        repo_file("apps/docs/e2e/docs.spec.mjs").exists(),
        "docs app should define a browser smoke test for the docs route",
    );
}

#[test]
fn runtime_web_playwright_config_exists() {
    assert!(
        repo_file("apps/web/playwright.config.mjs").exists(),
        "runtime web app should define a Playwright config for browser e2e",
    );
}

#[test]
fn runtime_web_live_harness_launches_paddles_inside_nix_develop() {
    let harness = read_repo_file("apps/web/scripts/serve-live-web-shell-harness.mjs");

    assert!(
        harness.contains("spawn(\n    'nix'"),
        "live runtime harness should launch paddles through nix develop",
    );
    assert!(
        harness.contains("'develop'"),
        "live runtime harness should enter nix develop before cargo run",
    );
    assert!(
        harness.contains("'cargo'"),
        "live runtime harness should still run the Rust paddles binary under the nix shell",
    );
}

#[test]
fn runtime_web_e2e_script_runs_the_product_route_playwright_suite() {
    let package_json = read_repo_file("apps/web/package.json");
    let package: Value =
        serde_json::from_str(&package_json).expect("apps/web/package.json should parse");
    let scripts = package["scripts"]
        .as_object()
        .expect("apps/web/package.json scripts should be an object");
    let e2e = scripts
        .get("e2e")
        .and_then(|value| value.as_str())
        .expect("apps/web/package.json should define an e2e script");

    assert_eq!(
        e2e, "playwright test -c playwright.config.mjs",
        "runtime web e2e should execute the single product-route Playwright contract",
    );
    assert!(
        repo_file("apps/web/e2e/product/runtime.spec.mjs").exists(),
        "runtime web should keep the product-route browser suite under apps/web/e2e/product",
    );
}

#[test]
fn runtime_web_app_defines_a_build_script() {
    let package_json = read_repo_file("apps/web/package.json");
    let package: Value =
        serde_json::from_str(&package_json).expect("apps/web/package.json should parse");
    let scripts = package["scripts"]
        .as_object()
        .expect("apps/web/package.json scripts should be an object");

    assert!(
        scripts.contains_key("build"),
        "runtime web app should define a build script for the paddles launch path",
    );
}

#[test]
fn runtime_web_app_uses_tanstack_router_instead_of_react_router() {
    let package_json = read_repo_file("apps/web/package.json");
    let package: Value =
        serde_json::from_str(&package_json).expect("apps/web/package.json should parse");
    let dependencies = package["dependencies"]
        .as_object()
        .expect("apps/web/package.json dependencies should be an object");

    assert!(
        dependencies.contains_key("@tanstack/react-router"),
        "runtime web app should depend on TanStack Router",
    );
    assert!(
        !dependencies.contains_key("react-router-dom"),
        "runtime web app should not keep the old react-router-dom dependency",
    );
}
