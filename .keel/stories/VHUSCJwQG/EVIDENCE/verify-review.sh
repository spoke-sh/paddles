#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../../.."

test -f src/application/turn_orchestration.rs
test -f src/application/conversation_read_model.rs
test -f src/application/synthesis_chamber.rs
test -f src/application/interpretation_chamber.rs
test -f src/application/recursive_control.rs

rg -q 'fn turn_orchestration\(&self\) -> TurnOrchestrationChamber' src/application/mod.rs
rg -q 'fn conversation_read_model\(&self\) -> ConversationReadModelChamber' src/application/mod.rs
rg -q 'self\.turn_orchestration\(\)\.process_prompt\(prompt\)\.await' src/application/mod.rs
rg -q 'self\.conversation_read_model\(\)' src/application/mod.rs
rg -q 'replay_conversation_projection\(task_id\)' src/application/mod.rs
rg -q 'pub\(super\) struct TurnOrchestrationChamber' src/application/turn_orchestration.rs
rg -q 'pub\(super\) struct ConversationReadModelChamber' src/application/conversation_read_model.rs
rg -q 'self\.service\.recursive_control\(\)' src/application/turn_orchestration.rs
rg -q 'execute_recursive_planner_loop' src/application/turn_orchestration.rs
rg -q 'self\.service\.synthesis_chamber\(\)' src/application/turn_orchestration.rs
rg -q 'finalize_turn_response' src/application/turn_orchestration.rs
