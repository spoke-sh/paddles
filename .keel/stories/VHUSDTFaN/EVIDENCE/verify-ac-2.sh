#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/../../../.." && pwd)
cd "$REPO_ROOT"

cargo test --test runtime_presentation_boundary -- --nocapture
cargo test infrastructure::runtime_presentation::tests:: -- --nocapture
cargo test infrastructure::web::tests::broadcast_event_sink_projects_runtime_items_for_control_state_events -- --nocapture
cargo test infrastructure::cli::interactive_tui::tests::gatherer_progress_rows_use_hunting_language -- --nocapture
cargo test application::tests::plain_turn_event_rendering_uses_hunting_language_for_gatherer_progress -- --nocapture
