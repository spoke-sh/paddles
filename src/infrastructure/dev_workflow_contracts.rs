use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const PRE_COMMIT_HOOK_CONTRACT_PATH: &str = "support/hooks/pre-commit";

fn repo_file(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_file(path)).unwrap_or_else(|error| {
        panic!("failed to read repo file `{path}`: {error}");
    })
}

fn try_read_repo_file(path: &str) -> Option<String> {
    fs::read_to_string(repo_file(path)).ok()
}

fn normalize_line_endings(contents: &str) -> String {
    contents.replace("\r\n", "\n")
}

#[test]
fn just_quality_runs_frontend_workspace_lint_checks() {
    let justfile = read_repo_file("justfile");
    let frontend_install_section = justfile
        .split("\n# Run frontend workspace verification checks.")
        .next()
        .and_then(|prefix| prefix.split("\nfrontend-install:\n").nth(1))
        .expect("justfile should contain a frontend-install recipe");

    assert!(
        frontend_install_section.contains("npm ci"),
        "frontend-install should use npm ci for clean workspace installs",
    );
    assert!(
        frontend_install_section.contains("rm -rf "),
        "frontend-install should clear workspace node_modules before npm ci to keep installs idempotent",
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
    let hook = read_repo_file(PRE_COMMIT_HOOK_CONTRACT_PATH);

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

    if let Some(installed_hook) = try_read_repo_file(".git/hooks/pre-commit") {
        assert_eq!(
            normalize_line_endings(&installed_hook),
            normalize_line_endings(&hook),
            "installed pre-commit hook should match the repo hook contract; run `keel hooks install`",
        );
    }
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
    let dev_shell = flake
        .split("devShells.default = pkgs.mkShell {")
        .nth(1)
        .expect("flake.nix should define the default dev shell");
    let linux_browser_inputs = dev_shell
        .split("++ pkgs.lib.optionals isLinux [")
        .nth(1)
        .and_then(|section| section.split("];").next())
        .expect("flake.nix should guard Linux-only dev shell inputs");

    assert!(
        flake.contains("pkgs.nodejs"),
        "dev shell should include nodejs so frontend workspace checks run in nix develop",
    );
    assert!(
        linux_browser_inputs.contains("pkgs.chromium"),
        "dev shell should provide nixpkgs chromium only on Linux where the package is supported",
    );
    assert!(
        flake.contains("export PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH="),
        "linux shells should point Playwright at the nix-provided Chromium executable",
    );
    assert!(
        flake.contains("export PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1"),
        "linux shells should skip Playwright browser downloads when nix provides Chromium",
    );
    assert!(
        flake.contains("Let Playwright manage its own browser download on macOS."),
        "darwin shells should fall back to Playwright-managed browsers instead of nixpkgs chromium",
    );
}

#[test]
fn nix_cargo_lock_vendoring_normalizes_duplicate_name_version_entries() {
    let cargo_lock = read_repo_file("Cargo.lock");
    let flake = read_repo_file("flake.nix");
    let lock: toml::Value = toml::from_str(&cargo_lock).expect("Cargo.lock should parse");
    let packages = lock["package"]
        .as_array()
        .expect("Cargo.lock package list should be an array");
    let mut packages_by_identity: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();

    for package in packages {
        let package = package
            .as_table()
            .expect("Cargo.lock package entries should be tables");
        let name = package["name"]
            .as_str()
            .expect("Cargo.lock package name should be a string")
            .to_owned();
        let version = package["version"]
            .as_str()
            .expect("Cargo.lock package version should be a string")
            .to_owned();
        let source = package
            .get("source")
            .and_then(toml::Value::as_str)
            .unwrap_or("path")
            .to_owned();

        packages_by_identity
            .entry((name, version))
            .or_default()
            .push(source);
    }

    let duplicates: Vec<String> = packages_by_identity
        .into_iter()
        .filter(|(_, sources)| sources.len() > 1)
        .map(|((name, version), sources)| format!("{name} {version}: {}", sources.join(", ")))
        .collect();

    if duplicates.is_empty() {
        return;
    }

    assert!(
        flake.contains("normalizedCargoLock ="),
        "flake should define a normalized Cargo.lock for nix vendoring when duplicate name/version entries exist:\n{}",
        duplicates.join("\n"),
    );
    assert!(
        flake.contains("lockFileContents = normalizedCargoLock;"),
        "flake should feed the normalized Cargo.lock contents into buildRustPackage when duplicate name/version entries exist:\n{}",
        duplicates.join("\n"),
    );
    assert!(
        flake.contains("cp ${normalizedCargoLockFile} \"''${cargoRoot:+$cargoRoot/}Cargo.lock\""),
        "flake should rewrite Cargo.lock in the build tree so nixpkgs sees the same normalized lockfile during cargoSetupPostPatchHook:\n{}",
        duplicates.join("\n"),
    );
    assert!(
        flake.contains("substituteInPlace \"$cargoDepsCopy/sift-0.2.0/Cargo.toml\""),
        "flake should patch the vendored sift manifest so its metamorph dependency matches the normalized lockfile during offline builds:\n{}",
        duplicates.join("\n"),
    );
    assert!(
        flake.contains("metamorph 0.1.0"),
        "flake normalization should handle the duplicated metamorph entry that breaks nix vendoring:\n{}",
        duplicates.join("\n"),
    );
}

#[test]
fn nix_package_build_uses_repo_rust_toolchain() {
    let flake = read_repo_file("flake.nix");

    assert!(
        flake.contains("rustPlatform = pkgs.makeRustPlatform"),
        "flake should derive a Rust platform from the repo toolchain so package builds do not drift onto nixpkgs' default cargo/rustc:\n{}",
        flake,
    );
    assert!(
        flake.contains("cargo = rust;"),
        "flake should wire the shared cargo package from the repo toolchain into the package rustPlatform:\n{}",
        flake,
    );
    assert!(
        flake.contains("rustc = rust;"),
        "flake should wire the shared rustc package from the repo toolchain into the package rustPlatform:\n{}",
        flake,
    );
    assert!(
        flake.contains("rustPlatform.buildRustPackage"),
        "flake should build the package through the shared rustPlatform instead of pkgs.rustPlatform:\n{}",
        flake,
    );
}

#[test]
fn nix_package_tracks_locked_sift_revision_for_vendoring() {
    let cargo_lock = read_repo_file("Cargo.lock");
    let flake = read_repo_file("flake.nix");
    let needle = "git+https://github.com/rupurt/sift?rev=";
    let revision_start = cargo_lock
        .find(needle)
        .map(|index| index + needle.len())
        .expect("Cargo.lock should pin the sift git dependency by revision");
    let revision = &cargo_lock[revision_start..revision_start + 40];

    assert!(
        flake.contains(&format!("github:rupurt/sift?rev={revision}")),
        "flake should pin the sift input to the same revision Cargo.lock vendors for offline nix builds:\nCargo.lock rev: {revision}\n{flake}",
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
    assert!(
        gitignore.contains("/result"),
        "repo should ignore the root nix build result symlink so local verification builds do not create tracked churn",
    );
}

#[test]
fn siftignore_excludes_workspace_local_sift_artifacts_from_search() {
    let siftignore = read_repo_file(".siftignore");

    assert!(
        siftignore.contains(".sift/**"),
        "repo should exclude workspace-local .sift artifacts so paddles search does not index its own cache"
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

#[test]
fn runtime_web_tests_follow_domain_partitioning() {
    assert!(
        repo_file("apps/web/src/test-support/runtime-harness.tsx").exists(),
        "runtime web tests should expose a shared harness for bootstrap state and render helpers",
    );
    assert!(
        repo_file("apps/web/src/chat/runtime-shell.test.tsx").exists(),
        "runtime web shell and chat behaviors should live in a domain test file",
    );
    assert!(
        repo_file("apps/web/src/inspector/inspector-route.test.tsx").exists(),
        "inspector behaviors should live in an inspector-scoped test file",
    );
    assert!(
        repo_file("apps/web/src/manifold/manifold-route.test.tsx").exists(),
        "manifold behaviors should live in a manifold-scoped test file",
    );
    assert!(
        repo_file("apps/web/src/transit/transit-route.test.tsx").exists(),
        "transit behaviors should live in a transit-scoped test file",
    );
    assert!(
        !repo_file("apps/web/src/runtime-app.test.tsx").exists(),
        "the legacy monolithic runtime-app test file should be retired after the domain split",
    );
}

#[test]
fn embedded_fallback_shell_parity_boundary_is_documented() {
    let architecture = read_repo_file("ARCHITECTURE.md");
    let configuration = read_repo_file("CONFIGURATION.md");

    assert!(
        architecture.contains("## Embedded Fallback Shell Parity Boundary"),
        "architecture should define an explicit embedded fallback shell parity boundary section",
    );
    assert!(
        architecture.contains("must stay aligned")
            && architecture.contains("`/`, `/transit`, and `/manifold`")
            && architecture.contains("chat transcript")
            && architecture.contains("tool output")
            && architecture.contains("transcript-driven manifold turn selection"),
        "architecture should name the bounded operator-facing behaviors the embedded shell must preserve",
    );
    assert!(
        architecture.contains("does not need React component/module parity"),
        "architecture should explicitly bound the embedded shell away from React file/module decomposition",
    );
    assert!(
        configuration.contains("### Embedded Fallback Shell Parity Boundary")
            && configuration.contains("compiled-in `src/infrastructure/web/index.html` shell")
            && configuration.contains("single-file DOM/JS runtime"),
        "configuration should explain the shipped fallback shell artifact and its intentionally bounded implementation shape",
    );
}

#[test]
fn narrative_machine_planning_docs_define_shared_vocabulary_and_selection_contract() {
    let epic = read_repo_file(".keel/epics/VGGIor3dC/README.md");
    let voyage = read_repo_file(".keel/epics/VGGIor3dC/voyages/VGGIqsj2g/README.md");

    assert!(
        epic.contains("selected turn")
            && epic.contains("selected moment")
            && epic.contains("optional internals"),
        "the narrative-machine epic should declare the shared turn + moment + internals selection model",
    );
    assert!(
        voyage.contains("Input")
            && voyage.contains("Diverter")
            && voyage.contains("Jam")
            && voyage.contains("Spring return")
            && voyage.contains("Force")
            && voyage.contains("Output"),
        "the first voyage should pin the shared operator vocabulary for later UI stories",
    );
}

#[test]
fn runtime_shell_host_keeps_panels_flush_to_the_viewport() {
    let runtime_shell_css = read_repo_file("apps/web/src/runtime-shell.css");
    let shared_css = read_repo_file("apps/web/src/styles/runtime-shell-base.css");

    assert!(
        runtime_shell_css.contains("@import './styles/runtime-shell-base.css';"),
        "runtime shell aggregate stylesheet should import the shared shell base",
    );
    assert!(
        shared_css.contains(".runtime-shell-host {\n  font-family:")
            && shared_css.contains("  padding: 8px;"),
        "runtime shell host should add around 8px padding around the two-panel layout",
    );
    assert!(
        shared_css.contains("@media (max-width: 960px) {\n  .runtime-shell-host { flex-direction: column; height: 100dvh; padding: 8px; }"),
        "mobile runtime shell should add outer viewport padding as well",
    );
}

#[test]
fn runtime_shell_buttons_do_not_underline_and_transit_route_drops_legacy_toggle_chrome() {
    let shared_css = read_repo_file("apps/web/src/styles/runtime-shell-base.css");
    let transit_css = read_repo_file("apps/web/src/styles/transit.css");

    assert!(
        shared_css.contains(".trace-tab {\n  border: 0;")
            && shared_css.contains("  text-decoration: none;"),
        "runtime route tabs should explicitly suppress link underlines",
    );
    assert!(
        transit_css.contains(".transit-machine__scrubber-chip {\n  border: 0;")
            && !transit_css.contains(".trace-transit-toggle {")
            && !transit_css.contains(".trace-transit-toolbar {"),
        "transit route should keep the machine scrubber buttons and drop the legacy toggle chrome",
    );
}

#[test]
fn native_transport_substrate_is_documented_in_owning_repo_docs() {
    let readme = read_repo_file("README.md");
    let configuration = read_repo_file("CONFIGURATION.md");
    let architecture = read_repo_file("ARCHITECTURE.md");

    assert!(
        readme.contains("Native Transport Substrate"),
        "README should introduce the shared native transport substrate",
    );
    assert!(
        configuration.contains("[native_transports.http_request_response]")
            && configuration.contains("[native_transports.server_sent_events]")
            && configuration.contains("[native_transports.websocket]")
            && configuration.contains("[native_transports.transit]"),
        "CONFIGURATION should show authored native transport sections for every shared transport",
    );
    assert!(
        configuration.contains("phase")
            && configuration.contains("bind_target")
            && configuration.contains("auth_mode")
            && configuration.contains("last_error"),
        "CONFIGURATION should describe the shared native transport diagnostics surface",
    );
    assert!(
        architecture.contains("NativeTransportRegistry")
            && architecture.contains("shared native transport substrate"),
        "ARCHITECTURE should describe the shared transport registry boundary and substrate role",
    );
}

#[test]
fn http_and_sse_transport_operator_workflow_is_documented() {
    let configuration = read_repo_file("CONFIGURATION.md");
    let public_reference = read_repo_file("apps/docs/docs/reference/native-transports.mdx");
    let sidebar = read_repo_file("apps/docs/sidebars.ts");

    assert!(
        configuration.contains("GET /health")
            && configuration.contains("GET /session/shared/bootstrap")
            && configuration.contains("must share the same bind_target")
            && configuration.contains("http_request_response")
            && configuration.contains("server_sent_events"),
        "CONFIGURATION should explain how operators inspect and debug the HTTP/SSE transport diagnostics surface",
    );
    assert!(
        public_reference.contains("HTTP request/response")
            && public_reference.contains("Server-Sent Events")
            && public_reference.contains("GET /health")
            && public_reference.contains("GET /session/shared/bootstrap")
            && public_reference.contains("bind_target")
            && public_reference.contains("last_error")
            && public_reference.contains("same bind_target"),
        "public docs should describe how HTTP and SSE are enabled, inspected, and debugged through the shared native transport diagnostics model",
    );
    assert!(
        sidebar.contains("reference/native-transports"),
        "the native transport operator guide should be reachable from the docs sidebar",
    );
}

#[test]
fn websocket_and_transit_transport_operator_workflow_is_documented() {
    let readme = read_repo_file("README.md");
    let configuration = read_repo_file("CONFIGURATION.md");
    let architecture = read_repo_file("ARCHITECTURE.md");
    let public_reference = read_repo_file("apps/docs/docs/reference/native-transports.mdx");

    assert!(
        configuration.contains("GET /native-transports/websocket")
            && configuration.contains("POST /native-transports/transit")
            && configuration.contains("application/transit+json")
            && configuration.contains("session_ready")
            && configuration.contains("connection_id")
            && configuration.contains("transit_exchange"),
        "CONFIGURATION should explain how operators enable and inspect the WebSocket and Transit adapters",
    );
    assert!(
        public_reference.contains("WebSocket")
            && public_reference.contains("Transit")
            && public_reference.contains("GET /native-transports/websocket")
            && public_reference.contains("POST /native-transports/transit")
            && public_reference.contains("application/transit+json")
            && public_reference.contains("session_ready")
            && public_reference.contains("transport_error"),
        "public docs should describe the WebSocket and Transit routes, payload expectations, and shared diagnostics behavior",
    );
    assert!(
        readme.contains("websocket")
            && readme.contains("transit")
            && readme.contains("shared listener")
            && architecture.contains("WebSocket")
            && architecture.contains("Transit")
            && architecture.contains("shared listener")
            && architecture.contains("structured request/response semantics"),
        "foundational docs should describe how WebSocket and Transit extend the shared native transport substrate",
    );
}

#[test]
fn forensic_route_drops_legacy_nav_list_pane_chrome_and_docs_the_internals_escape_hatch() {
    let inspector_css = read_repo_file("apps/web/src/styles/inspector.css");
    let readme = read_repo_file("README.md");
    let architecture = read_repo_file("ARCHITECTURE.md");

    assert!(
        !repo_file("apps/web/src/inspector/inspector-nav.tsx").exists()
            && !repo_file("apps/web/src/inspector/inspector-record-list.tsx").exists()
            && !repo_file("apps/web/src/inspector/inspector-detail-pane.tsx").exists(),
        "legacy forensic nav/list/detail components should be retired once the machine route owns the default operator path",
    );
    assert!(
        !inspector_css.contains(".forensic-nav {")
            && !inspector_css.contains(".forensic-shell {")
            && !inspector_css.contains(".forensic-main {")
            && !inspector_css.contains(".forensic-detail-pane {"),
        "inspector stylesheet should not keep the retired nav/list/pane chrome",
    );
    assert!(
        readme.contains("machine-first detail surface")
            && readme.contains("Show internals")
            && readme.contains("detail drawer")
            && architecture.contains("machine-first detail surface")
            && architecture.contains("detail drawer")
            && architecture.contains("why the selected moment mattered")
            && architecture.contains("explicit drill-down"),
        "foundational docs should describe the new default forensic drawer workflow and its internals escape hatch",
    );
}
