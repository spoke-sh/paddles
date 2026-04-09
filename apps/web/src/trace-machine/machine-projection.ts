import type {
  ConversationProjectionSnapshot,
  ConversationTraceGraphNode,
  ForensicRecordProjection,
  ForensicTurnProjection,
} from '../runtime-types';
import { machineMomentEntry, machineMomentLabel, type MachineMomentKind } from './machine-model';

export interface MachineMomentRawLinks {
  primaryForensicRecordId: string | null;
  primaryTraceNodeId: string | null;
  forensicRecordIds: string[];
  traceNodeIds: string[];
  branchIds: string[];
}

export interface MachineMomentProjection {
  momentId: string;
  turnId: string;
  lifecycle: string;
  sequence: number;
  kind: MachineMomentKind;
  label: string;
  headline: string;
  narrative: string;
  raw: MachineMomentRawLinks;
}

export interface MachineTurnProjection {
  turnId: string;
  lifecycle: string;
  moments: MachineMomentProjection[];
}

export interface ConversationMachineProjection {
  taskId: string;
  turns: MachineTurnProjection[];
}

type SequenceBucket = {
  sequence: number;
  records: ForensicRecordProjection[];
  nodes: ConversationTraceGraphNode[];
};

export function projectConversationMachine(
  snapshot: ConversationProjectionSnapshot
): ConversationMachineProjection {
  return {
    taskId: snapshot.task_id,
    turns: snapshot.forensics.turns.map((turn) => projectTurnMachine(turn, snapshot.trace_graph.nodes)),
  };
}

export function projectTurnMachine(
  turn: ForensicTurnProjection,
  graphNodes: ConversationTraceGraphNode[]
): MachineTurnProjection {
  const buckets = new Map<number, SequenceBucket>();
  const recordIds = new Set(turn.records.map((record) => record.record.record_id));

  for (const record of turn.records) {
    const bucket = ensureSequenceBucket(buckets, record.record.sequence);
    bucket.records.push(record);
  }

  for (const node of graphNodes) {
    if (!recordIds.has(node.id)) {
      continue;
    }
    const bucket = ensureSequenceBucket(buckets, node.sequence);
    bucket.nodes.push(node);
  }

  const moments = [...buckets.values()]
    .sort((left, right) => left.sequence - right.sequence)
    .map((bucket) => buildMachineMoment(turn, bucket));

  return {
    turnId: turn.turn_id,
    lifecycle: turn.lifecycle,
    moments,
  };
}

function ensureSequenceBucket(buckets: Map<number, SequenceBucket>, sequence: number) {
  let bucket = buckets.get(sequence);
  if (!bucket) {
    bucket = {
      sequence,
      records: [],
      nodes: [],
    };
    buckets.set(sequence, bucket);
  }
  return bucket;
}

function buildMachineMoment(
  turn: ForensicTurnProjection,
  bucket: SequenceBucket
): MachineMomentProjection {
  const primaryRecord = bucket.records[0] || null;
  const primaryNode = bucket.nodes[0] || null;
  const kind = resolveMachineMomentKind(primaryRecord, primaryNode);
  const lexicon = machineMomentEntry(kind);
  const branchIds = uniqueStrings([
    ...bucket.records.map((record) => record.record.lineage.branch_id || ''),
    ...bucket.nodes.map((node) => node.branch_id || ''),
  ]);

  return {
    momentId: `${turn.turn_id}.moment-${String(bucket.sequence).padStart(4, '0')}`,
    turnId: turn.turn_id,
    lifecycle: primaryRecord?.lifecycle || turn.lifecycle,
    sequence: bucket.sequence,
    kind,
    label: machineMomentLabel(kind),
    headline: machineHeadline(kind, primaryRecord, primaryNode),
    narrative: machineNarrative(primaryRecord, primaryNode, lexicon.narrative),
    raw: {
      primaryForensicRecordId: primaryRecord?.record.record_id || null,
      primaryTraceNodeId: primaryNode?.id || null,
      forensicRecordIds: bucket.records.map((record) => record.record.record_id),
      traceNodeIds: bucket.nodes.map((node) => node.id),
      branchIds,
    },
  };
}

function resolveMachineMomentKind(
  record: ForensicRecordProjection | null,
  node: ConversationTraceGraphNode | null
): MachineMomentKind {
  const recordKind = forensicKindKey(record);
  if (recordKind === 'TaskRootStarted' || recordKind === 'TurnStarted') {
    return 'input';
  }
  if (recordKind === 'SignalSnapshot') {
    return 'force';
  }
  if (recordKind === 'PlannerBranchDeclared' || node?.kind === 'branch') {
    return 'diverter';
  }
  if (recordKind === 'ThreadMerged' || node?.kind === 'merge') {
    return 'spring_return';
  }
  if (
    recordKind === 'ToolCallRequested' ||
    recordKind === 'ToolCallCompleted' ||
    node?.kind === 'tool' ||
    node?.kind === 'tool_done'
  ) {
    return 'tool_run';
  }
  if (recordKind === 'CompletionCheckpoint' || node?.kind === 'checkpoint') {
    return 'output';
  }
  if (recordKind === 'PlannerAction' && record) {
    return plannerActionKind(record);
  }
  if (
    recordKind === 'SelectionArtifact' ||
    recordKind === 'ModelExchangeArtifact' ||
    node?.kind === 'evidence'
  ) {
    return 'evidence_probe';
  }
  if (node?.kind === 'root' || node?.kind === 'turn') {
    return 'input';
  }
  if (node?.kind === 'signal') {
    return 'force';
  }
  if (node?.kind === 'action') {
    return 'planner';
  }
  return 'unknown';
}

function plannerActionKind(record: ForensicRecordProjection): MachineMomentKind {
  const value = forensicKindValue(record);
  const action = String(value.action || '').toLowerCase();
  if (
    action.startsWith('read ') ||
    action.startsWith('search ') ||
    action.startsWith('list files') ||
    action.startsWith('inspect ') ||
    action.startsWith('diff ') ||
    action.startsWith('refine ')
  ) {
    return 'evidence_probe';
  }
  if (
    action.startsWith('apply_patch') ||
    action.startsWith('write_file') ||
    action.startsWith('replace_in_file') ||
    action.startsWith('shell ')
  ) {
    return 'tool_run';
  }
  if (action.startsWith('stop ') || action.startsWith('answer ')) {
    return 'output';
  }
  return 'planner';
}

function machineHeadline(
  kind: MachineMomentKind,
  record: ForensicRecordProjection | null,
  node: ConversationTraceGraphNode | null
) {
  const value = forensicKindValue(record);
  if (kind === 'evidence_probe' && typeof value.action === 'string') {
    return String(value.action);
  }
  if (kind === 'tool_run' && typeof value.tool_name === 'string') {
    return `${String(value.tool_name)} completed`;
  }
  if (kind === 'force' && typeof value.summary === 'string') {
    return String(value.summary);
  }
  if (node?.label) {
    return node.label;
  }
  return machineMomentLabel(kind);
}

function machineNarrative(
  record: ForensicRecordProjection | null,
  node: ConversationTraceGraphNode | null,
  fallback: string
) {
  const value = forensicKindValue(record);
  if (typeof value.summary === 'string' && value.summary.trim()) {
    return value.summary;
  }
  if (typeof value.rationale === 'string' && value.rationale.trim()) {
    return value.rationale;
  }
  if (typeof value.action === 'string' && value.action.trim()) {
    return value.action;
  }
  if (node?.label) {
    return node.label;
  }
  return fallback;
}

function forensicKindKey(record: ForensicRecordProjection | null) {
  if (!record) {
    return null;
  }
  const kind = record.record.kind || {};
  const [key] = Object.keys(kind);
  return key || null;
}

function forensicKindValue(record: ForensicRecordProjection | null): Record<string, unknown> {
  const key = forensicKindKey(record);
  return (key ? (record?.record.kind[key] as Record<string, unknown>) : null) || {};
}

function uniqueStrings(values: string[]) {
  return [...new Set(values.filter(Boolean))];
}
