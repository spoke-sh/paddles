# PROTOCOL.md: The Keel Communications Protocol

The Keel CLI implements an asynchronous communication layer through the `ping` and `poke` commands. This system allows the workflow engine to field requests, route messages, and provide either synchronous automated responses or facilitate asynchronous offline interactions.

> Foundational stack position: `7/8`
> Read this after [ARCHITECTURE.md](ARCHITECTURE.md) and before [CONFIGURATION.md](CONFIGURATION.md).

This document defines the expected message structures, the routing logic, and the lifecycle of an inbox message.

## 1. The Inbox Lifecycle

All communication is routed through the `.keel/inbox/` directory.

1. **Submission:** A user or agent invokes `keel ping "<message>"`.
2. **Evaluation:** The engine evaluates the message against its **Routing Rules**.
3. **Response (Sync vs. Async):**
   - **Synchronous (`pong`):** If a routing rule matches, the engine immediately responds via `stdout`, marks the message as `ponged`, and saves the state to the inbox.
   - **Asynchronous (Pending):** If no routing rule matches, the engine returns only the generated **Ping ID** (e.g., `VDtzUxoCp`) to `stdout`. The message is saved to the inbox in a `pending` state.
4. **Resolution (`poke`):** A pending message can be resolved later using `keel poke <id> "[message]"`.
   - If a manual message is provided, it acts as the response, marking the ping as `ponged`.
   - If no manual message is provided, the engine re-evaluates the original message against the current routing rules (which may have been updated).

## 2. Message Format

Messages are persisted as JSON files within `.keel/inbox/<id>.json`.

**Schema (`PingMessage`):**
```json
{
  "id": "VDtzUxoCp",
  "message": "The original message content.",
  "timestamp": "2026-03-15T03:00:00Z",
  "status": "pending | ponged",
  "pong_message": "The response message, or null if pending."
}
```
- **`id`**: A globally unique identifier generated using Keel's standard ID generation (e.g., `VD...`).
- **`status`**: The current state of the interaction (`pending` or `ponged`).
- **`pong_message`**: The recorded response, ensuring historical traceability of the communication.

## 3. Routing Rules

When a `ping` (or parameter-less `poke`) is executed, the engine attempts to match the message content to a set of predefined rules to trigger a synchronous **auto-pong**.

Currently, the routing rules are simple substring matches (case-insensitive):

| Match Condition (contains word) | Synchronous Response (Pong) |
| :--- | :--- |
| `"ping"` | `pong` |
| `"hello"` or `"hi"` | `Hello! I am keel. How can I help?` |
| `"help"` | `I am a workflow engine. Try running `keel doctor` or `keel flow`.` |

*If a message does not match any of these conditions, it falls through to the Asynchronous track and requires a future `poke`.*

## 4. JSONIN and JSONOUT Data Contracts

As a primary interface for autonomous agents and external scripts, Keel implements strict `JSONOUT` and potential `JSONIN` data contracts. These are separate from human-readable CLI outputs and guarantee structured predictability.

### JSONOUT: Engine to Agent
Whenever a command is invoked with `--json` (e.g., `keel pulse --json`, `keel next --json`, `keel verify run --json`), the standard out is guaranteed to be a single, parseable JSON payload representing the complete return state.

- **Predictable Schemas**: Changes to `--json` output schemas must be treated as breaking changes. 
- **Example (`keel pulse --json`)**:
  ```json
  {
    "mode": "materialize",
    "evaluated": 3,
    "created": 1,
    "skipped": 0,
    "deferred": 2,
    "routines": [
      {
        "id": "routine-due",
        "outcome": "created",
        "story_id": "VDtx8IW2K"
      }
    ]
  }
  ```

### JSONIN: Agent to Engine
While most inputs to Keel are simple strings and flags (like `keel ping "hello"`), the protocol anticipates `JSONIN` payloads for complex configurations, bulk operations, or structured responses (like piping an LLM's structured JSON output directly to a state-mutating command).

- **The Inbox as JSONIN**: The `.keel/inbox/<id>.json` file itself represents our first JSONIN construct. When an agent creates or modifies a ping file directly, it must conform to the `PingMessage` schema defined above.
- **Future Support**: We plan to support piping JSON directly into commands (e.g., `cat payload.json | keel poke <id> --json-in`) to allow agents to pass rich contextual data structures instead of flat strings.

## 5. The System Pacemaker

The communication protocol also serves as the system's **Pacemaker**. The `.keel/heartbeat` file records the kinetic energy of the workflow.

### The Heartbeat
- **Activation**: Invoking `keel poke` (without a targeted ping ID) updates the heartbeat file.
- **Energization**: The engine considers itself "energized" if the heartbeat's modification time is within the configured `battery_decay_minutes` (default: 10m).
- **Idle State**: If the heartbeat decays, the engine transitions to **IDLE**, dimming the visual scenes and pausing autonomous backlog discharge.

### Pace-setting
To maintain board integrity, the pacemaker must be synchronized with all state changes. 
- **The Protocol**: Every state-mutating commit MUST be "pace-set" by executing a final `keel poke` immediately before the commit.
- **Consistency**: This ensures that the recorded "energy" of the system is precisely aligned with the resulting Git hash and board state.

## 6. Expanding the Protocol

As Keel's capabilities grow, the routing rules will be expanded to support more complex interactions:
- **Regex/Semantic Matching:** Moving beyond simple word inclusion to understand intent.
- **Action Triggers:** Allowing a `ping` to synchronously trigger engine operations (e.g., "ping: status" running `keel flow`).
- **Agent Handoffs:** Routing pending messages to specific sub-agents or workflow lanes for evaluation during `keel pulse`.
