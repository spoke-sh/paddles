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
fn just_quality_runs_website_lint_checks() {
    let justfile = read_repo_file("justfile");
    assert!(
        justfile.contains("website-install:\n  npm --prefix website ci"),
        "justfile should define a website install helper using npm ci",
    );
    let quality_section = justfile
        .split("\n# Run the paddles CLI.")
        .next()
        .and_then(|prefix| prefix.split("\nquality:\n").nth(1))
        .expect("justfile should contain a quality recipe");

    assert!(
        quality_section.contains("just website-install"),
        "quality should install website dependencies before JS checks",
    );
    assert!(
        quality_section.contains("just website-quality"),
        "quality should run website lint checks",
    );
}

#[test]
fn just_test_runs_website_test_checks() {
    let justfile = read_repo_file("justfile");
    let test_section = justfile
        .split("\n# Run quality checks.")
        .next()
        .and_then(|prefix| prefix.split("\ntest:\n").nth(1))
        .expect("justfile should contain a test recipe");

    assert!(
        test_section.contains("just website-install"),
        "test should install website dependencies before JS tests",
    );
    assert!(
        test_section.contains("just website-test"),
        "test should run website test checks",
    );
    assert!(
        test_section.contains("just website-e2e"),
        "test should run browser e2e checks",
    );
}

#[test]
fn website_package_defines_lint_and_test_scripts() {
    let package_json = read_repo_file("website/package.json");
    let package: Value = serde_json::from_str(&package_json).expect("package.json should parse");
    let scripts = package["scripts"]
        .as_object()
        .expect("package.json scripts should be an object");

    assert!(
        scripts.contains_key("lint"),
        "website package should define a lint script",
    );
    assert!(
        scripts.contains_key("test"),
        "website package should define a test script",
    );
    assert!(
        scripts.contains_key("e2e"),
        "website package should define an e2e script",
    );
}

#[test]
fn dev_shell_exposes_node_for_website_checks() {
    let flake = read_repo_file("flake.nix");
    assert!(
        flake.contains("pkgs.nodejs"),
        "dev shell should include nodejs so website checks run in nix develop",
    );
    assert!(
        flake.contains("pkgs.chromium"),
        "dev shell should include chromium so browser e2e checks run in nix develop",
    );
}

#[test]
fn website_lockfile_exists_for_clean_ci_installs() {
    assert!(
        repo_file("website/package-lock.json").exists(),
        "website should commit a package-lock.json so CI can use npm ci",
    );
}

#[test]
fn website_static_favicon_exists_for_docusaurus_builds() {
    assert!(
        repo_file("website/static/img/favicon.svg").exists(),
        "website should ship the favicon asset referenced by docusaurus.config.ts",
    );
}

#[test]
fn website_playwright_config_exists() {
    assert!(
        repo_file("website/playwright.config.mjs").exists(),
        "website should define a Playwright config for browser e2e",
    );
}
