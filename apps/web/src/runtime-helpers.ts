import type {
  ArtifactEnvelope,
  ConversationTraceGraphNode,
  ForensicRecordProjection,
  ForensicTurnProjection,
  ManifoldFrame,
  ManifoldGateState,
  ManifoldPrimitiveBasis,
  ManifoldPrimitiveState,
  ManifoldSignalState,
  ProjectionTurnEvent,
  TraceLineageRef,
  TraceSignalContribution,
  TurnEvent,
} from './runtime-types';

export const TRACE_TILE_MIN = 128;
export const TRACE_TILE_GAP = 14;
export const TRACE_ZOOM_MIN = 0.4;
export const TRACE_ZOOM_MAX = 1.85;
export const MANIFOLD_PLAYBACK_STEP_MS = 720;

export const KIND_COLORS: Record<string, string> = {
  root: '#2d90c8',
  turn: '#2d90c8',
  action: '#f08a24',
  branch: '#dfb23f',
  tool: '#f08a24',
  tool_done: '#d27820',
  evidence: '#17956c',
  checkpoint: '#17956c',
  merge: '#d65f4a',
  thread: '#70808d',
  forensic: '#2d90c8',
  lineage: '#5376a3',
  signal: '#f08a24',
};

export const TRACE_SIGNIFICANT_KINDS = new Set([
  'root',
  'turn',
  'action',
  'branch',
  'tool',
  'evidence',
  'checkpoint',
  'merge',
]);

export const TRACE_KIND_FAMILY: Record<string, string> = {
  forensic: 'model_io',
  lineage: 'lineage',
  signal: 'signals',
  thread: 'threads',
  tool_done: 'tool_results',
};

export const TRACE_DETAIL_LEVEL_LABELS: Record<string, string> = {
  overview: 'overview detail',
  balanced: 'balanced detail',
  focus: 'focus detail',
};

export const FORCE_KIND_COLORS: Record<string, string> = {
  context_strain: '#f08a24',
  compaction_cue: '#5376a3',
  action_bias: '#d65f4a',
  fallback: '#9b5de5',
  budget_boundary: '#dfb23f',
};

export const FORCE_LEVEL_COLORS: Record<string, string> = {
  low: '#17956c',
  medium: '#2d90c8',
  high: '#f08a24',
  critical: '#d65f4a',
};

export const FORCE_SOURCE_LABELS: Record<string, string> = {
  operator_memory: 'Operator memory',
  retained_artifacts: 'Retained artifacts',
  thread_summaries: 'Thread summaries',
  evidence_budget: 'Evidence budget',
  controller_policy: 'Controller policy',
  prompt_edit_signal: 'Edit signal',
  candidate_file_evidence: 'Candidate file evidence',
  provider_or_parser: 'Provider or parser',
  controller_safety: 'Controller safety',
  workspace_editor_boundary: 'Workspace editor boundary',
  search_budget: 'Search budget',
  inspect_budget: 'Inspect budget',
  read_budget: 'Read budget',
  premise_challenge: 'Premise challenge',
  planner_budget: 'Planner budget',
  context: 'Context',
};

export const FORCE_SOURCE_PALETTE = [
  '#2d90c8',
  '#17956c',
  '#f08a24',
  '#d65f4a',
  '#5376a3',
  '#dfb23f',
];

export const TRACE_VIEW_LABELS: Record<string, string> = {
  inspector: 'Forensic Inspector',
  manifold: 'Steering Gate Manifold',
  transit: 'Turn Steps',
};

export const STEERING_GATE_ORDER = ['evidence', 'convergence', 'containment'];

export const STEERING_GATE_COLORS: Record<string, string> = {
  evidence: '#2d90c8',
  convergence: '#f08a24',
  containment: '#17956c',
};

export const FORENSIC_KIND_LABELS: Record<string, string> = {
  TaskRootStarted: 'task root',
  TurnStarted: 'turn start',
  PlannerAction: 'planner step',
  PlannerBranchDeclared: 'branch',
  SelectionArtifact: 'selection artifact',
  ModelExchangeArtifact: 'model exchange',
  LineageEdge: 'lineage edge',
  SignalSnapshot: 'influence snapshot',
  ToolCallRequested: 'tool call',
  ToolCallCompleted: 'tool result',
  CompletionCheckpoint: 'completion',
  ThreadMerged: 'thread merge',
  ThreadCandidateCaptured: 'thread candidate',
  ThreadDecisionSelected: 'thread decision',
};

export function humanizeToken(token: string | null | undefined) {
  return String(token ?? '')
    .replace(/_/g, ' ')
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .toLowerCase();
}

export function titleCase(text: string | null | undefined) {
  return humanizeToken(text)
    .split(' ')
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}

export function sourceLabel(source: string) {
  return FORCE_SOURCE_LABELS[source] || titleCase(source || 'unknown');
}

export function sourceColor(source: string) {
  const key = String(source || 'source');
  let hash = 0;
  for (let index = 0; index < key.length; index += 1) {
    hash = (hash * 31 + key.charCodeAt(index)) >>> 0;
  }
  return FORCE_SOURCE_PALETTE[hash % FORCE_SOURCE_PALETTE.length];
}

export function signalKindLabel(kind: string | null | undefined) {
  return titleCase(kind || 'signal');
}

export function steeringGateLabel(gate: string | null | undefined) {
  return `${titleCase(gate || 'containment')} gate`;
}

export function steeringGateClass(gate: string | null | undefined) {
  return String(gate || 'containment')
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9_-]/g, '-');
}

export function steeringPhaseLabel(phase: string | null | undefined) {
  return titleCase(phase || 'steady');
}

export function manifoldGateLabel(gate: ManifoldGateState) {
  return gate.label || steeringGateLabel(gate.gate);
}

export function truncate(value: string | null | undefined, length: number) {
  const text = String(value ?? '');
  return text.length > length ? `${text.slice(0, length)}...` : text;
}

export function kindEntry(recordProjection: ForensicRecordProjection) {
  const kind = recordProjection?.record?.kind || {};
  const [key] = Object.keys(kind);
  return {
    key: key || 'Unknown',
    value: ((key && kind[key]) as Record<string, unknown>) || {},
  };
}

export function kindLabel(key: string) {
  return (
    FORENSIC_KIND_LABELS[key] ||
    key.replace(/([a-z0-9])([A-Z])/g, '$1 $2').replace(/_/g, ' ').toLowerCase()
  );
}

export function lifecycleLabel(lifecycle: string | null | undefined) {
  return lifecycle ? String(lifecycle).replace(/_/g, ' ') : 'unknown';
}

export function traceNodeFamily(kind: string) {
  return TRACE_KIND_FAMILY[kind] || 'significant';
}

export function traceNodeVisible(
  node: ConversationTraceGraphNode,
  scope: 'significant' | 'full',
  visibility: Record<string, boolean>
) {
  if (scope === 'full') {
    return true;
  }
  if (TRACE_SIGNIFICANT_KINDS.has(node.kind)) {
    return true;
  }
  return !!visibility[traceNodeFamily(node.kind)];
}

export function traceDetailLevelForZoom(zoom: number) {
  if (zoom <= 0.68) {
    return 'overview';
  }
  if (zoom <= 1.14) {
    return 'balanced';
  }
  return 'focus';
}

export function traceLayoutForZoom(zoom: number) {
  const detailLevel = traceDetailLevelForZoom(zoom);
  const zoomCurve = Math.pow(zoom, 0.82);
  const tileMultiplier =
    detailLevel === 'overview' ? 0.84 : detailLevel === 'balanced' ? 0.96 : 1.14;
  const columnGap =
    TRACE_TILE_GAP *
    (detailLevel === 'overview' ? 0.84 : detailLevel === 'balanced' ? 1 : 1.2) *
    Math.max(0.78, zoom * 0.9);
  const rowGap =
    TRACE_TILE_GAP *
    (detailLevel === 'overview' ? 0.94 : detailLevel === 'balanced' ? 1.08 : 1.28) *
    Math.max(0.8, zoom * 0.92);

  return {
    detailLevel,
    tileMin: TRACE_TILE_MIN * zoomCurve * tileMultiplier,
    columnGap,
    rowGap,
  };
}

export function traceNodeDirection(
  rowIndex: number,
  nodeIndex: number,
  rowLength: number,
  totalRows: number
) {
  const isReverseRow = rowIndex % 2 === 1;
  const isRowTail = nodeIndex === rowLength - 1;
  const hasNextRow = rowIndex < totalRows - 1;
  if (isRowTail && hasNextRow) {
    return 'down';
  }
  return isReverseRow ? 'rtl' : 'ltr';
}

export function formatTraceKind(kind: string) {
  return kind.replace(/_/g, ' ');
}

export function artifactText(artifact: ArtifactEnvelope | null | undefined) {
  return artifact?.inline_content || '';
}

export function primaryArtifact(recordProjection: ForensicRecordProjection | null | undefined) {
  if (!recordProjection) {
    return null;
  }
  const entry = kindEntry(recordProjection);
  if (entry.key === 'ModelExchangeArtifact') {
    return (entry.value.artifact as ArtifactEnvelope) || null;
  }
  if (entry.key === 'CompletionCheckpoint') {
    return (entry.value.response as ArtifactEnvelope) || null;
  }
  if (entry.key === 'SelectionArtifact') {
    return (entry.value.artifact as ArtifactEnvelope) || null;
  }
  if (entry.key === 'ToolCallRequested' || entry.key === 'ToolCallCompleted') {
    return (entry.value.payload as ArtifactEnvelope) || null;
  }
  if (entry.key === 'SignalSnapshot') {
    return (entry.value.artifact as ArtifactEnvelope) || null;
  }
  if (entry.key === 'TurnStarted' || entry.key === 'TaskRootStarted') {
    return (entry.value.prompt as ArtifactEnvelope) || null;
  }
  return null;
}

export function maybePrettyJson(text: string) {
  if (!text) {
    return '';
  }
  try {
    return JSON.stringify(JSON.parse(text), null, 2);
  } catch {
    return text;
  }
}

export function renderedRecordBody(recordProjection: ForensicRecordProjection) {
  const entry = kindEntry(recordProjection);
  const artifact = primaryArtifact(recordProjection);
  if (artifact) {
    if (entry.key === 'SignalSnapshot') {
      const resolver = resolverSignalDetailsFromArtifact(artifact);
      if (resolver) {
        const detailLines = [
          resolver.path || resolverOutcomeMeta(resolver),
          resolverOutcomeNarrative(resolver),
        ].filter(Boolean);
        return [resolverOutcomeTitle(resolver), '', ...detailLines].join('\n');
      }
    }
    const text = artifactText(artifact);
    if (!text) {
      return artifact.locator
        ? `Payload stored in context at ${JSON.stringify(artifact.locator, null, 2)}`
        : 'No inline payload was recorded for this artifact.';
    }
    return artifact.mime_type?.includes('json') ? maybePrettyJson(text) : text;
  }
  if (entry.key === 'LineageEdge') {
    return [
      String(entry.value.summary || 'lineage edge'),
      '',
      `${String((entry.value.source as TraceLineageRef)?.label || 'source')} -> ${String(
        (entry.value.target as TraceLineageRef)?.label || 'target'
      )}`,
      `relation: ${String(entry.value.relation || 'related')}`,
    ].join('\n');
  }
  if (entry.key === 'PlannerAction') {
    return `action: ${String(entry.value.action || '')}\n\nrationale: ${String(
      entry.value.rationale || ''
    )}`;
  }
  if (entry.key === 'SignalSnapshot') {
    const contributions = (((entry.value.contributions as TraceSignalContribution[]) || []).map(
      (contribution) =>
        `- ${contribution.source}: ${contribution.share_percent}% (${contribution.rationale || ''})`
    )).join('\n');
    return [
      String(entry.value.summary || 'influence snapshot'),
      '',
      `level: ${String(entry.value.level || 'unknown')}`,
      `magnitude: ${String(entry.value.magnitude_percent || 0)}%`,
      '',
      contributions,
    ].join('\n');
  }
  return maybePrettyJson(JSON.stringify(entry.value, null, 2));
}

export function rawRecordBody(recordProjection: ForensicRecordProjection) {
  const artifact = primaryArtifact(recordProjection);
  if (artifact?.inline_content) {
    return artifact.inline_content;
  }
  return JSON.stringify(recordProjection.record, null, 2);
}

export function recordSummary(recordProjection: ForensicRecordProjection) {
  const entry = kindEntry(recordProjection);
  if (entry.key === 'ModelExchangeArtifact') {
    const artifact = (entry.value.artifact as ArtifactEnvelope) || {};
    return String(artifact.summary || `${entry.value.category} ${entry.value.phase}`);
  }
  if (entry.key === 'PlannerAction') {
    return String(entry.value.action || 'planner action');
  }
  if (entry.key === 'LineageEdge') {
    return String(entry.value.summary || 'lineage edge');
  }
  if (entry.key === 'SignalSnapshot') {
    return String(entry.value.summary || 'influence snapshot');
  }
  if (entry.key === 'CompletionCheckpoint') {
    return String(entry.value.summary || 'completion');
  }
  if (entry.key === 'SelectionArtifact') {
    return String(entry.value.summary || 'selection');
  }
  if (entry.key === 'ToolCallRequested' || entry.key === 'ToolCallCompleted') {
    return String(entry.value.tool_name || 'tool');
  }
  if ((entry.key === 'TurnStarted' || entry.key === 'TaskRootStarted') && entry.value.prompt) {
    return String(((entry.value.prompt as ArtifactEnvelope).summary || entry.key));
  }
  return kindLabel(entry.key);
}

export function recordMeta(recordProjection: ForensicRecordProjection) {
  const entry = kindEntry(recordProjection);
  const base = [
    `step ${recordProjection.record.sequence}`,
    kindLabel(entry.key),
    lifecycleLabel(recordProjection.lifecycle),
  ];
  if (entry.key === 'ModelExchangeArtifact') {
    base.push(
      String(entry.value.lane || ''),
      String(entry.value.phase || ''),
      `${String(entry.value.provider || 'provider')}:${String(entry.value.model || 'model')}`
    );
  }
  if (entry.key === 'SignalSnapshot') {
    base.push(String(entry.value.level || ''), `${String(entry.value.magnitude_percent || 0)}%`);
  }
  return base.filter(Boolean).join(' · ');
}

export function recordsForTurn(
  turn: ForensicTurnProjection | null,
  focus: { kind: 'all' | 'model_call' | 'planner_step'; id: string | null }
) {
  if (!turn) {
    return [];
  }
  return turn.records.filter((recordProjection) => recordMatchesFocus(recordProjection, focus));
}

export function plannerStepsForTurn(turn: ForensicTurnProjection | null) {
  if (!turn) {
    return [];
  }
  return turn.records
    .filter((recordProjection) => kindEntry(recordProjection).key === 'PlannerAction')
    .map((recordProjection) => {
      const entry = kindEntry(recordProjection);
      return {
        id: `planner-step:${recordProjection.record.record_id}`,
        recordId: recordProjection.record.record_id,
        label: truncate(String(entry.value.action || 'planner step'), 44),
      };
    });
}

export function modelCallsForTurn(turn: ForensicTurnProjection | null) {
  if (!turn) {
    return [];
  }
  const grouped = new Map<string, Record<string, string[] | string>>();
  for (const recordProjection of turn.records) {
    const entry = kindEntry(recordProjection);
    if (entry.key !== 'ModelExchangeArtifact') {
      continue;
    }
    const exchangeId = String(entry.value.exchange_id || '');
    if (!grouped.has(exchangeId)) {
      grouped.set(exchangeId, {
        id: exchangeId,
        lane: String(entry.value.lane || ''),
        category: String(entry.value.category || ''),
        provider: String(entry.value.provider || ''),
        model: String(entry.value.model || ''),
        summary: String(
          ((entry.value.artifact as ArtifactEnvelope) || {}).summary || 'model call'
        ),
        phases: [],
      });
    }
    const current = grouped.get(exchangeId)!;
    (current.phases as string[]).push(String(entry.value.phase || ''));
  }
  return Array.from(grouped.values()).map((value) => ({
    id: String(value.id || ''),
    lane: String(value.lane || ''),
    category: String(value.category || ''),
    provider: String(value.provider || ''),
    model: String(value.model || ''),
    summary: String(value.summary || ''),
  }));
}

export function latestRecordForTurn(turn: ForensicTurnProjection | null, focus: { kind: 'all' | 'model_call' | 'planner_step'; id: string | null }) {
  const records = recordsForTurn(turn, focus);
  return records.length ? records[records.length - 1] : null;
}

function nodeIdsForRecord(recordProjection: ForensicRecordProjection) {
  const ids = new Set<string>();
  const entry = kindEntry(recordProjection);
  if (entry.key === 'ModelExchangeArtifact') {
    ids.add(`model-call:${String(entry.value.exchange_id || '')}`);
    const artifact = entry.value.artifact as ArtifactEnvelope;
    if (artifact?.artifact_id) {
      ids.add(`artifact:${artifact.artifact_id}`);
    }
  } else if (entry.key === 'PlannerAction') {
    ids.add(`planner-step:${recordProjection.record.record_id}`);
  } else if (entry.key === 'CompletionCheckpoint') {
    const response = entry.value.response as ArtifactEnvelope;
    if (response?.artifact_id) {
      ids.add(`output:${response.artifact_id}`);
    }
  } else if (entry.key === 'SignalSnapshot') {
    ids.add(`signal:${recordProjection.record.record_id}`);
    const appliesTo = entry.value.applies_to as TraceLineageRef | undefined;
    if (appliesTo?.id) {
      ids.add(appliesTo.id);
    }
  } else if (entry.key === 'LineageEdge') {
    const source = entry.value.source as TraceLineageRef | undefined;
    const target = entry.value.target as TraceLineageRef | undefined;
    if (source?.id) {
      ids.add(source.id);
    }
    if (target?.id) {
      ids.add(target.id);
    }
  }
  return ids;
}

function recordMatchesFocus(
  recordProjection: ForensicRecordProjection,
  focus: { kind: 'all' | 'model_call' | 'planner_step'; id: string | null }
) {
  if (!focus || focus.kind === 'all') {
    return true;
  }
  const entry = kindEntry(recordProjection);
  if (focus.kind === 'model_call') {
    if (entry.key === 'ModelExchangeArtifact') {
      return String(entry.value.exchange_id || '') === focus.id;
    }
    if (entry.key === 'LineageEdge') {
      const source = entry.value.source as TraceLineageRef | undefined;
      const target = entry.value.target as TraceLineageRef | undefined;
      return source?.id === `model-call:${focus.id}` || target?.id === `model-call:${focus.id}`;
    }
    if (entry.key === 'SignalSnapshot') {
      const appliesTo = entry.value.applies_to as TraceLineageRef | undefined;
      return appliesTo?.id === `model-call:${focus.id}`;
    }
    return false;
  }
  if (focus.kind === 'planner_step') {
    if (entry.key === 'PlannerAction') {
      return `planner-step:${recordProjection.record.record_id}` === focus.id;
    }
    if (entry.key === 'LineageEdge') {
      const source = entry.value.source as TraceLineageRef | undefined;
      const target = entry.value.target as TraceLineageRef | undefined;
      return source?.id === focus.id || target?.id === focus.id;
    }
    if (entry.key === 'SignalSnapshot') {
      const appliesTo = entry.value.applies_to as TraceLineageRef | undefined;
      return appliesTo?.id === focus.id;
    }
    return false;
  }
  return true;
}

export function strongestSignalSnapshot(records: ForensicRecordProjection[]) {
  if (!records.length) {
    return null;
  }
  return records.reduce((best, candidate) => {
    const currentMagnitude = Number(kindEntry(candidate).value.magnitude_percent || 0);
    const bestMagnitude = Number(kindEntry(best).value.magnitude_percent || 0);
    return currentMagnitude > bestMagnitude ? candidate : best;
  });
}

export function aggregateSignalContributions(records: ForensicRecordProjection[]) {
  const totals = new Map<
    string,
    { source: string; share: number; rationales: Set<string> }
  >();
  let totalShare = 0;

  for (const record of records) {
    const contributions = (kindEntry(record).value.contributions as TraceSignalContribution[]) || [];
    for (const contribution of contributions) {
      const key = contribution.source || 'context';
      const bucket = totals.get(key) || {
        source: key,
        share: 0,
        rationales: new Set<string>(),
      };
      bucket.share += contribution.share_percent || 0;
      if (contribution.rationale) {
        bucket.rationales.add(contribution.rationale);
      }
      totals.set(key, bucket);
      totalShare += contribution.share_percent || 0;
    }
  }

  const items = Array.from(totals.values())
    .sort((left, right) => right.share - left.share)
    .map((bucket) => ({
      source: bucket.source,
      label: sourceLabel(bucket.source),
      color: sourceColor(bucket.source),
      share: bucket.share,
      percent: 0,
      rationale: Array.from(bucket.rationales)[0] || '',
    }));

  if (!items.length) {
    return [];
  }

  const denominator = totalShare || items.reduce((sum, item) => sum + item.share, 0) || 1;
  let assigned = 0;
  items.forEach((item, index) => {
    const percent =
      index + 1 === items.length
        ? Math.max(0, 100 - assigned)
        : Math.round((item.share / denominator) * 100);
    item.percent = percent;
    assigned += percent;
  });
  return items;
}

export function manifoldPrimitiveKindLabel(kind: string) {
  return titleCase(kind);
}

export function manifoldPrimitiveBasisLabel(basis: ManifoldPrimitiveBasis) {
  if (basis.kind === 'signal_family' && basis.signal_kind) {
    return signalKindLabel(basis.signal_kind);
  }
  if (basis.kind === 'steering_gate' && basis.gate) {
    return steeringGateLabel(basis.gate);
  }
  if (basis.kind === 'lineage_anchor' && basis.anchor) {
    return basis.anchor.label || titleCase(basis.anchor.kind);
  }
  return titleCase(basis.kind);
}

export function manifoldLifecycleClass(lifecycle: string) {
  return lifecycle || 'provisional';
}

export function manifoldAnchorLabel(anchor: { label?: string; kind?: string } | null | undefined) {
  if (!anchor) {
    return 'turn scope';
  }
  return anchor.label || titleCase(anchor.kind || 'anchor');
}

export function manifoldSignalLabel(signal: ManifoldSignalState) {
  return signalKindLabel(signal.kind);
}

export interface ResolverSignalDetails {
  status: 'resolved' | 'ambiguous' | 'missing';
  source: string | null;
  path: string | null;
  candidates: string[];
  attemptedHintCount: number | null;
  explanation: string;
}

function resolverSignalDetailsFromArtifact(
  artifact: ArtifactEnvelope | null | undefined
): ResolverSignalDetails | null {
  const text = artifact?.inline_content || '';
  if (!text) {
    return null;
  }
  try {
    const parsed = JSON.parse(text) as Record<string, unknown>;
    if (parsed.stage !== 'entity-resolution') {
      return null;
    }
    const status = String(parsed.status || '').toLowerCase();
    if (status !== 'resolved' && status !== 'ambiguous' && status !== 'missing') {
      return null;
    }
    return {
      status,
      source: typeof parsed.source === 'string' ? parsed.source : null,
      path: typeof parsed.path === 'string' ? parsed.path : null,
      candidates: Array.isArray(parsed.candidates)
        ? parsed.candidates.filter((candidate): candidate is string => typeof candidate === 'string')
        : [],
      attemptedHintCount:
        typeof parsed.attempted_hint_count === 'number' ? parsed.attempted_hint_count : null,
      explanation: typeof parsed.explanation === 'string' ? parsed.explanation : '',
    };
  } catch {
    return null;
  }
}

export function resolverSignalDetails(
  signal: Pick<ManifoldSignalState, 'artifact'> | null | undefined
) {
  return resolverSignalDetailsFromArtifact(signal?.artifact);
}

function resolverSourceLabel(source: string | null) {
  return source ? titleCase(source.replace(/[-_]/g, ' ')) : 'Resolver signal';
}

export function resolverOutcomeTitle(details: ResolverSignalDetails) {
  if (details.status === 'resolved') {
    return 'Resolved target';
  }
  if (details.status === 'ambiguous') {
    return 'Ambiguous target';
  }
  return 'Missing target';
}

export function resolverOutcomeMeta(details: ResolverSignalDetails) {
  if (details.status === 'resolved') {
    const alternatives = Math.max(0, details.candidates.length - 1);
    return `${resolverSourceLabel(details.source)} · ${alternatives} alternative${
      alternatives === 1 ? '' : 's'
    }`;
  }
  if (details.status === 'ambiguous') {
    return `${resolverSourceLabel(details.source)} · ${details.candidates.length} authored candidates`;
  }
  if (details.attemptedHintCount !== null) {
    return `${resolverSourceLabel(details.source)} · ${details.attemptedHintCount} hint${
      details.attemptedHintCount === 1 ? '' : 's'
    } checked`;
  }
  return resolverSourceLabel(details.source);
}

export function resolverOutcomeNarrative(details: ResolverSignalDetails) {
  if (details.path) {
    return details.explanation || details.path;
  }
  if (details.candidates.length) {
    return details.explanation || details.candidates.join(', ');
  }
  return details.explanation || 'Resolver metadata was recorded without a detailed explanation.';
}

function formatDuration(seconds: number) {
  if (seconds < 60) {
    return `${seconds}s`;
  }
  const minutes = Math.floor(seconds / 60);
  const remainder = seconds % 60;
  if (remainder === 0) {
    return `${minutes}m`;
  }
  return `${minutes}m ${String(remainder).padStart(2, '0')}s`;
}

export function primitivePhase(
  turn: { frames: ManifoldFrame[] } | null,
  frameIndex: number,
  primitive: ManifoldPrimitiveState
) {
  if (!turn || frameIndex >= turn.frames.length - 1) {
    return 'stable';
  }
  const laterFrame = turn.frames
    .slice(frameIndex + 1)
    .find((frame) => frame.primitives.some((candidate) => candidate.primitive_id === primitive.primitive_id));
  return laterFrame ? 'accumulating' : 'bleed_off';
}

export function eventRow(payload: ProjectionTurnEvent | TurnEvent) {
  const projectionEvent = payload as Partial<ProjectionTurnEvent>;
  const diff = eventDiff(payload);
  const event = ('event' in payload ? payload.event : payload) as TurnEvent;
  const type = String(event.type || '');
  if (projectionEvent.presentation) {
    const row = {
      badge: projectionEvent.presentation.badge,
      badgeClass: projectionEvent.presentation.badge_class,
      text: projectionEvent.presentation.text,
    };
    if (type === 'tool_output') {
      return {
        ...row,
        output: projectionEvent.presentation.detail,
        streamKey:
          typeof event.call_id === 'string' && typeof event.stream === 'string'
            ? `tool-stream:${event.call_id}:${event.stream}`
            : undefined,
      };
    }
    if (type === 'plan_updated') {
      return {
        ...row,
        output: projectionEvent.presentation.detail,
      };
    }
    return diff ? { ...row, diff } : row;
  }

  if (type === 'intent_classified') {
    return { badge: 'route', badgeClass: 'route', text: `Intent: ${String(event.intent || '')}` };
  }
  if (type === 'interpretation_context') {
    return { badge: 'context', badgeClass: 'route', text: String(event.summary || 'Interpretation updated') };
  }
  if (type === 'planner_action_selected') {
    return {
      badge: 'planner',
      badgeClass: 'planner',
      text: `Step ${String(event.sequence || '')}: ${String(event.action || '')}`,
    };
  }
  if (type === 'tool_called') {
    return {
      badge: 'tool',
      badgeClass: 'tool',
      text: `${String(event.tool_name || 'tool')}: ${truncate(String(event.invocation || ''), 60)}`,
    };
  }
  if (type === 'plan_updated') {
    return {
      badge: 'plan',
      badgeClass: 'planner',
      text: 'Updated Plan',
      output: Array.isArray(event.items)
        ? event.items
            .map((item) => {
              const status = item && typeof item === 'object' ? String(item.status || '') : '';
              const label = item && typeof item === 'object' ? String(item.label || '') : '';
              const marker = status === 'completed' ? '✓' : '□';
              return `${marker} ${label}`.trimEnd();
            })
            .join('\n')
        : '',
    };
  }
  if (type === 'tool_output') {
    return {
      badge: 'term',
      badgeClass: 'tool-terminal',
      text: `${String(event.tool_name || 'tool')} ${String(event.stream || 'output')}`,
      output: String(event.output || ''),
      streamKey:
        typeof event.call_id === 'string' && typeof event.stream === 'string'
          ? `tool-stream:${event.call_id}:${event.stream}`
          : undefined,
    };
  }
  if (type === 'workspace_edit_applied') {
    const row = {
      badge: 'tool',
      badgeClass: 'tool-diff',
      text: `${String(event.tool_name || 'tool')} applied`,
    };
    return diff ? { ...row, diff } : row;
  }
  if (type === 'tool_finished') {
    const toolName = String(event.tool_name || 'tool');
    const row = {
      badge: 'tool',
      badgeClass: diff ? 'tool-diff' : 'tool',
      text: `${toolName} done`,
    };
    return diff ? { ...row, diff } : row;
  }
  if (type === 'gatherer_summary') {
    return { badge: 'gather', badgeClass: 'gatherer', text: String(event.summary || '') };
  }
  if (type === 'harness_state') {
    const snapshot = (event.snapshot as Record<string, unknown> | undefined) || {};
    const governor = (snapshot.governor as Record<string, unknown> | undefined) || {};
    const timeout = (governor.timeout as Record<string, unknown> | undefined) || {};
    const chamber = String(snapshot.chamber || 'unknown');
    const status = String(governor.status || 'active');
    const phase = String(timeout.phase || 'nominal');
    const detail = String(snapshot.detail || governor.intervention || '').trim();
    return {
      badge: 'gov',
      badgeClass: 'governor',
      text: [chamber, `status ${status}`, `timeout ${phase}`, detail]
        .filter(Boolean)
        .join(' · '),
    };
  }
  if (type === 'gatherer_search_progress') {
    const strategy = String(event.strategy || '').trim();
    const detail = String(event.detail || '').trim();
    const etaSeconds =
      typeof event.eta_seconds === 'number' ? Number(event.eta_seconds) : null;
    const fallback = `hunting (${String(event.phase || 'phase')})`;
    const text = [
      strategy || null,
      detail || fallback,
      etaSeconds == null ? null : `eta ${formatDuration(etaSeconds)}`,
    ]
      .filter(Boolean)
      .join(' · ');
    return {
      badge: 'gather',
      badgeClass: 'gatherer',
      text,
    };
  }
  if (type === 'planner_summary') {
    return {
      badge: 'planner',
      badgeClass: 'planner',
      text: `${String(event.strategy || '')} ${String(event.mode || '')} (${String(
        event.steps || 0
      )} steps)`,
    };
  }
  if (type === 'synthesis_ready') {
    return {
      badge: 'synth',
      badgeClass: event.insufficient_evidence ? 'fallback' : 'synthesis',
      text: event.insufficient_evidence ? 'Insufficient evidence' : 'Grounded',
    };
  }
  if (type === 'route_selected') {
    return {
      badge: 'route',
      badgeClass: 'route',
      text: truncate(String(event.summary || 'Route selected'), 80),
    };
  }
  if (type === 'fallback') {
    return {
      badge: 'fallback',
      badgeClass: 'fallback',
      text: `${String(event.stage || 'fallback')}: ${String(event.reason || '')}`,
    };
  }
  if (type === 'planner_capability' || type === 'gatherer_capability') {
    return {
      badge: 'cap',
      badgeClass: 'route',
      text: `${String(event.provider || '')}: ${String(event.capability || '')}`,
    };
  }
  if (type === 'thread_merged') {
    return {
      badge: 'merge',
      badgeClass: 'planner',
      text: `${String(event.source_thread || '')} -> ${String(event.target_thread || '')}`,
    };
  }
  return null;
}

function eventDiff(payload: ProjectionTurnEvent | TurnEvent) {
  const event = ('event' in payload ? payload.event : payload) as TurnEvent;
  const type = String(event.type || '');
  if (type === 'workspace_edit_applied') {
    const diff = String(
      ((event as Record<string, unknown>).edit as Record<string, unknown> | undefined)?.diff || ''
    ).trim();
    return diff || null;
  }
  if (type === 'tool_finished') {
    return extractMutationDiff(
      String((event as Record<string, unknown>).tool_name || ''),
      String((event as Record<string, unknown>).summary || '')
    );
  }
  return null;
}

function isMutationTool(toolName: string) {
  return toolName === 'diff' || toolName === 'apply_patch';
}

function extractMutationDiff(toolName: string, summary: string) {
  if (!isMutationTool(toolName)) {
    return null;
  }
  if (toolName === 'diff') {
    if (summary === 'No diff output.') {
      return summary;
    }
    return (
      stripPrefix(summary, 'Diff output:\n') || stripPrefix(summary, 'Diff output:\r\n') || null
    );
  }
  if (toolName === 'apply_patch' && summary.startsWith('Applied patch:\n')) {
    const body = summary.slice('Applied patch:\n'.length);
    return body.split('\n\nExit status:')[0].split('\nExit status:')[0].trim();
  }
  return null;
}

function stripPrefix(text: string, prefix: string) {
  return text.startsWith(prefix) ? text.slice(prefix.length) : null;
}
