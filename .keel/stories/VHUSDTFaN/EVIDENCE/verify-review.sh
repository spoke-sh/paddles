#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/../../../.." && pwd)
cd "$REPO_ROOT"

test -f src/infrastructure/runtime_presentation.rs
test ! -f src/domain/model/runtime_events.rs

rg -n '^pub mod runtime_presentation;$' src/infrastructure/mod.rs
if rg -n '^pub mod runtime_events;$' src/domain/model/mod.rs; then
    exit 1
fi
if rg -n 'pub use runtime_events::' src/domain/model/mod.rs; then
    exit 1
fi

rg -n 'crate::infrastructure::runtime_presentation' src/infrastructure/cli/interactive_tui.rs
rg -n 'crate::infrastructure::runtime_presentation' src/infrastructure/web/mod.rs
