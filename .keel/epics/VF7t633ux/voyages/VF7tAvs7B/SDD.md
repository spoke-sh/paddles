# Sift-Native Runtime Cutover - Software Design Description

> Replace wonopcode-owned core orchestration with a Sift-backed controller that supports retained context and immediate local tool execution.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage replaces the wonopcode-owned prompt loop with a Paddles-managed
runtime that combines:

- a Sift-backed local generative conversation
- Sift context assembly for retained evidence and searchable tool outputs
- a native local tool surface for coding work

The runtime remains single-process and local-first. Paddles owns the turn loop,
tool execution, and state retention; Sift owns retrieval, retained-artifact
budgeting, and synthetic local-context indexing.

## Context & Boundaries

This design covers core runtime orchestration in `application` and
`infrastructure`, session state, tool call parsing, tool execution, Sift
context assembly, local generative model usage, and the CLI/dependency cutover.

It does not cover remote tools, multi-agent planning beyond the local tool
loop, or presentation-heavy streaming/TUI work.

```
┌──────────────────────────────────────────────────────────────┐
│                     Paddles Runtime Cutover                 │
│                                                              │
│  CLI/Repl ──> MechSuitService ──> Sift Session Controller    │
│                                 ├─ generative conversation   │
│                                 ├─ retained context state    │
│                                 └─ local tool executor       │
│                                              │               │
│                                              v               │
│                                  Sift context assembly       │
└──────────────────────────────────────────────────────────────┘
             ↑                                   ↑
        Workspace FS                        Local shell/git
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `sift` | Rust crate | Generative sessions, context assembly, retained artifacts, local synthetic context | Git `main` |
| Local filesystem | Runtime substrate | File tools, diff/edit operations, workspace search corpus | POSIX/host FS |
| Local shell / `git` | Host toolchain | Shell commands and diff/apply operations | Host-provided |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Runtime owner | Paddles owns the turn loop and tool execution | Removes wonopcode from the core execution boundary |
| Retrieval plan | Use Sift context assembly with an explicit lexical plan for controller/tool context | Avoids hidden dense-model/network requirements on prompt execution |
| Tool protocol | Single JSON tool call per assistant step | Keeps the first cut simple and inspectable |
| State retention | Store tool outputs and turns as Sift local context plus bounded retained artifacts | Makes previous work searchable and bounded across turns |

## Architecture

The runtime is split into:

- `application::MechSuitService`: boot, model preparation, and prompt entrypoints
- `infrastructure::adapters::sift_agent`: session controller over Sift conversation and context
- local tool executor: search, file, shell, edit, and diff operations
- `main.rs`: CLI-only presentation loop

wonopcode crates no longer own runtime state or tool execution.

## Components

- `MechSuitService`
  Purpose: own boot, model setup, and runtime session lifetime.
  Interface: `prepare_model`, `process_prompt`.
- `SiftAgentAdapter`
  Purpose: run one conversational turn, assemble workspace context, parse tool calls, execute tools, and retain resulting state.
  Interface: `respond(prompt) -> Result<String>`.
- Local tool executor
  Purpose: expose simple local search/file/shell/edit/diff operations.
  Interface: typed tool call enum plus execution result summarization.

## Interfaces

- Assistant tool call contract:
  Respond with either plain assistant text or a single JSON object describing one tool call.
- Sift context contract:
  Use `ContextAssemblyRequest` with retained artifacts and `LocalContextSource::{ToolOutput,AgentTurn,EnvironmentFact}`.
- Local tool results:
  Return concise, searchable summaries suitable for `ToolOutputInput`.

## Data Flow

1. CLI receives a prompt.
2. `MechSuitService` delegates to the active `SiftAgentAdapter`.
3. The adapter assembles lexical workspace context from the current prompt, retained artifacts, and prior local context.
4. The adapter prompts the local generative conversation with the current user request, available tools, and assembled evidence.
5. If the model emits a tool call, the adapter executes it locally and records the result as `ToolOutputInput`.
6. The loop repeats until the model emits a final answer.
7. User/assistant turns and retained artifacts are stored for the next prompt.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Tool call JSON is malformed | JSON parse failure | Treat output as final assistant text | User can retry; verbose logs show raw output |
| Local tool execution fails | IO/process exit status error | Record a tool error summary and continue the loop | Assistant can recover with another tool or answer |
| Workspace path escapes root | Path resolution validation | Reject the tool call | Assistant can request a different path |
| Sift context assembly fails | `anyhow::Result` error | Surface an execution error to CLI | Retry after fixing corpus/config issue |
