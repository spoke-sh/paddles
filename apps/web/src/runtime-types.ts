export type RenderBlock =
  | { type: 'heading'; text: string }
  | { type: 'paragraph'; text: string }
  | { type: 'bullet_list'; items: string[] }
  | { type: 'code_block'; language?: string | null; code: string }
  | { type: 'citations'; sources: string[] };

export interface RenderDocument {
  blocks: RenderBlock[];
}

export interface SessionResponse {
  session_id: string;
}

export interface ConversationTranscriptEntry {
  record_id: string;
  turn_id: string;
  speaker: 'user' | 'assistant' | 'system';
  content: string;
  response_mode?:
    | 'direct_answer'
    | 'grounded_answer'
    | 'completed_edit'
    | 'blocked_edit'
    | 'policy_refusal'
    | null;
  render?: RenderDocument | null;
}

export interface ConversationTranscript {
  task_id: string;
  entries: ConversationTranscriptEntry[];
}

export interface TraceLineageRef {
  id: string;
  kind: string;
  label: string;
}

export interface TraceLineage {
  task_id: string;
  turn_id: string;
  branch_id?: string | null;
  parent_record_id?: string | null;
}

export interface TraceRecord {
  record_id: string;
  sequence: number;
  lineage: TraceLineage;
  kind: Record<string, unknown>;
}

export interface ForensicRecordProjection {
  lifecycle: string;
  superseded_by_record_id?: string | null;
  record: TraceRecord;
}

export interface ForensicTurnProjection {
  turn_id: string;
  lifecycle: string;
  records: ForensicRecordProjection[];
}

export interface ConversationForensicProjection {
  task_id: string;
  turns: ForensicTurnProjection[];
}

export interface TraceSignalContribution {
  source: string;
  share_percent: number;
  rationale?: string;
}

export interface ArtifactEnvelope {
  artifact_id?: string;
  summary?: string;
  inline_content?: string;
  mime_type?: string;
  truncated?: boolean;
  labels?: Record<string, string>;
  locator?: Record<string, unknown>;
}

export interface ManifoldSignalState {
  snapshot_record_id: string;
  lifecycle: string;
  kind: string;
  gate: string;
  phase: string;
  summary: string;
  level: string;
  magnitude_percent: number;
  anchor?: TraceLineageRef | null;
  contributions: TraceSignalContribution[];
  artifact: ArtifactEnvelope;
}

export interface ManifoldPrimitiveBasis {
  kind: string;
  signal_kind?: string;
  gate?: string;
  anchor?: TraceLineageRef;
}

export interface ManifoldPrimitiveState {
  primitive_id: string;
  kind: string;
  label: string;
  basis: ManifoldPrimitiveBasis;
  evidence_record_id?: string | null;
  anchor?: TraceLineageRef | null;
  level: string;
  magnitude_percent: number;
}

export interface ManifoldConduitState {
  conduit_id: string;
  from_primitive_id: string;
  to_primitive_id: string;
  label: string;
  basis: ManifoldPrimitiveBasis;
  evidence_record_id?: string | null;
}

export interface ManifoldGateState {
  gate: string;
  label: string;
  phase: string;
  level: string;
  magnitude_percent: number;
  anchor?: TraceLineageRef | null;
  dominant_signal_kind: string;
  signal_kinds: string[];
  dominant_record_id?: string | null;
}

export interface ManifoldFrame {
  record_id: string;
  sequence: number;
  lifecycle: string;
  anchor?: TraceLineageRef | null;
  active_signals: ManifoldSignalState[];
  gates: ManifoldGateState[];
  primitives: ManifoldPrimitiveState[];
  conduits: ManifoldConduitState[];
}

export interface ManifoldTurnProjection {
  turn_id: string;
  lifecycle: string;
  frames: ManifoldFrame[];
}

export interface ConversationManifoldProjection {
  task_id: string;
  turns: ManifoldTurnProjection[];
}

export interface ConversationTraceGraphNode {
  id: string;
  kind: string;
  label: string;
  branch_id?: string | null;
  sequence: number;
}

export interface ConversationTraceGraphEdge {
  from: string;
  to: string;
}

export interface ConversationTraceGraphBranch {
  id: string;
  label: string;
  status: string;
  parent_branch_id?: string | null;
}

export interface ConversationTraceGraph {
  task_id: string;
  nodes: ConversationTraceGraphNode[];
  edges: ConversationTraceGraphEdge[];
  branches: ConversationTraceGraphBranch[];
}

export interface DelegatedWorkerProjection {
  worker_id: string;
  role_label: string;
  ownership_summary: string;
  read_scopes: string[];
  write_scopes: string[];
  parent_thread: string;
  worker_thread: string;
  status:
    | 'running'
    | 'waiting'
    | 'awaiting_integration'
    | 'integrated'
    | 'closed'
    | 'conflict'
    | 'rejected'
    | 'stale'
    | 'unavailable';
  progress_summary: string;
  latest_detail: string;
  artifact_count: number;
  completion_recorded: boolean;
  integration_status?: 'integrated' | 'rejected' | 'stale' | 'unavailable' | null;
  degraded: boolean;
}

export interface ConversationDelegationProjection {
  task_id: string;
  harness_identity: string;
  active_worker_count: number;
  degraded_worker_count: number;
  workers: DelegatedWorkerProjection[];
}

export interface ConversationProjectionSnapshot {
  task_id: string;
  transcript: ConversationTranscript;
  forensics: ConversationForensicProjection;
  manifold: ConversationManifoldProjection;
  trace_graph: ConversationTraceGraph;
  delegation: ConversationDelegationProjection;
}

export interface ConversationProjectionUpdate {
  task_id: string;
  kind: 'transcript' | 'forensic';
  transcript_update?: { task_id: string } | null;
  forensic_update?: { task_id: string; turn_id: string; record_id: string } | null;
  snapshot: ConversationProjectionSnapshot;
}

export interface ConversationBootstrapResponse {
  session_id: string;
  projection: ConversationProjectionSnapshot;
  prompt_history: string[];
}

export interface RuntimeEventPresentation {
  badge: string;
  badge_class: string;
  title: string;
  detail: string;
  text: string;
}

export interface RuntimeItemPlan {
  kind: 'plan';
  payload: {
    items: Array<Record<string, unknown>>;
  };
}

export interface RuntimeItemDiff {
  kind: 'diff';
  payload: {
    files: string[];
    diff: string;
    insertions: number;
    deletions: number;
  };
}

export interface RuntimeItemCommand {
  kind: 'command';
  payload: {
    call_id: string;
    tool_name: string;
    phase: string;
    detail: string;
  };
}

export interface RuntimeItemFile {
  kind: 'file';
  payload: {
    path: string;
    operation: string;
  };
}

export interface RuntimeItemControl {
  kind: 'control';
  payload: {
    result: {
      operation: {
        scope: string;
        operation: string;
      };
      status: string;
      subject: Record<string, unknown>;
      detail: string;
    };
  };
}

export type RuntimeItem =
  | RuntimeItemPlan
  | RuntimeItemDiff
  | RuntimeItemCommand
  | RuntimeItemFile
  | RuntimeItemControl;

export type TurnEvent = Record<string, unknown> & { type: string };

export interface ProjectionTurnEvent {
  event: TurnEvent;
  presentation: RuntimeEventPresentation;
  runtime_items: RuntimeItem[];
}
