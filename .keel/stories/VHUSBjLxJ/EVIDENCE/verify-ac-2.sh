#!/usr/bin/env zsh
set -euo pipefail

cd /home/alex/workspace/spoke-sh/paddles

cargo test planner_workspace_actions_emit_governance_decision_events -- --nocapture

if sed -n '1,4688p' src/infrastructure/adapters/sift_agent.rs | rg -n 'TurnEvent::ToolCalled|TurnEvent::ToolFinished|TurnEvent::WorkspaceEditApplied|TurnEvent::ExecutionGovernanceDecisionRecorded|build_tool_retry_prompt|build_tool_follow_up_prompt|parse_tool_call|execute_tool\('; then
  echo "forbidden adapter repository-action emission surface remains"
  exit 1
fi

echo "runtime adapter no longer emits repository-action events or tool-loop helpers"
