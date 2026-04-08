import {
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import {
  Link,
  Outlet,
  RouterProvider,
  createRootRoute,
  createRoute,
  createRouter,
  useRouterState,
} from '@tanstack/react-router';

import {
  FORCE_KIND_COLORS,
  FORCE_LEVEL_COLORS,
  KIND_COLORS,
  MANIFOLD_PLAYBACK_STEP_MS,
  STEERING_GATE_COLORS,
  STEERING_GATE_ORDER,
  TRACE_DETAIL_LEVEL_LABELS,
  TRACE_VIEW_LABELS,
  TRACE_ZOOM_MAX,
  TRACE_ZOOM_MIN,
  aggregateSignalContributions,
  formatTraceKind,
  kindEntry,
  kindLabel,
  lifecycleLabel,
  latestRecordForTurn,
  manifoldAnchorLabel,
  manifoldGateLabel,
  manifoldSignalLabel,
  modelCallsForTurn,
  plannerStepsForTurn,
  primaryArtifact,
  rawRecordBody,
  recordMeta,
  recordSummary,
  recordsForTurn,
  renderedRecordBody,
  resolverOutcomeMeta,
  resolverOutcomeNarrative,
  resolverOutcomeTitle,
  resolverSignalDetails,
  signalKindLabel,
  sourceColor,
  steeringGateClass,
  steeringGateLabel,
  steeringPhaseLabel,
  strongestSignalSnapshot,
  traceDetailLevelForZoom,
  traceLayoutForZoom,
  traceNodeDirection,
  traceNodeFamily,
  traceNodeVisible,
  truncate,
} from './runtime-helpers';
import { ChatComposer } from './chat/composer';
import {
  ManifoldTurnSelectionProvider,
  useManifoldTurnSelection,
} from './chat/manifold-turn-selection-context';
import { TranscriptPane } from './chat/transcript-pane';
import { useChatComposer } from './chat/use-chat-composer';
import { useStickyTailScroll } from './chat/use-sticky-tail-scroll';
import { RuntimeStoreProvider, useRuntimeStore } from './runtime-store';
import type {
  ConversationTraceGraphNode,
  ForensicRecordProjection,
  ForensicTurnProjection,
  ManifoldFrame,
  ManifoldTurnProjection,
} from './runtime-types';

type FocusState = { kind: 'all' | 'model_call' | 'planner_step'; id: string | null };

function activeViewForPath(pathname: string) {
  if (pathname === '/manifold') {
    return 'manifold';
  }
  if (pathname === '/transit') {
    return 'transit';
  }
  return 'inspector';
}

function previousArtifactBaseline(
  turn: ForensicTurnProjection | null,
  recordProjection: ForensicRecordProjection | null
) {
  if (!turn || !recordProjection) {
    return null;
  }
  for (let index = turn.records.length - 1; index >= 0; index -= 1) {
    const candidate = turn.records[index];
    if (candidate.record.sequence >= recordProjection.record.sequence) {
      continue;
    }
    if (primaryArtifact(candidate)) {
      return candidate;
    }
  }
  return null;
}

function comparisonSnippet(
  recordProjection: ForensicRecordProjection | null,
  detailMode: 'rendered' | 'raw'
) {
  if (!recordProjection) {
    return 'No lineage artifact available.';
  }
  const body =
    detailMode === 'raw' ? rawRecordBody(recordProjection) : renderedRecordBody(recordProjection);
  return truncate(body.replace(/\s+/g, ' ').trim(), 180);
}

function RuntimeShellLayout() {
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const activeView = activeViewForPath(pathname);
  const { connected, error, events, projection, promptHistory, sending, sendTurn } =
    useRuntimeStore();
  const manifoldTurns = projection?.manifold.turns || [];
  const manifoldTurnIds = useMemo(
    () => new Set(manifoldTurns.map((turn) => turn.turn_id)),
    [manifoldTurns]
  );
  const [selectedManifoldTurnId, setSelectedManifoldTurnId] = useState<string | null>(null);
  const transcriptEntryCount = projection?.transcript.entries.length || 0;
  const { messagesRef, onMessagesScroll } = useStickyTailScroll({
    eventCount: events.length,
    transcriptEntryCount,
  });
  const { composerParts, onPromptKeyDown, onPromptPaste, onSubmit, prompt, setPrompt } =
    useChatComposer({
      promptHistory,
      onSubmitPrompt: sendTurn,
    });

  useEffect(() => {
    if (!manifoldTurns.length) {
      setSelectedManifoldTurnId(null);
      return;
    }
    if (
      !selectedManifoldTurnId ||
      !manifoldTurns.some((turn) => turn.turn_id === selectedManifoldTurnId)
    ) {
      setSelectedManifoldTurnId(manifoldTurns[manifoldTurns.length - 1].turn_id);
    }
  }, [manifoldTurns, selectedManifoldTurnId]);

  function selectManifoldTurnFromTranscript(turnId: string) {
    if (!manifoldTurnIds.has(turnId)) {
      return;
    }
    setSelectedManifoldTurnId(turnId);
  }

  return (
    <ManifoldTurnSelectionProvider
      value={{
        selectedTurnId: selectedManifoldTurnId,
        setSelectedTurnId: setSelectedManifoldTurnId,
      }}
    >
      <>
        <div className="chat-panel">
          <div className="chat-header">Paddles</div>
          <TranscriptPane
            activeView={activeView}
            connected={connected}
            error={error}
            events={events}
            manifoldTurnIds={manifoldTurnIds}
            messagesRef={messagesRef}
            onMessagesScroll={onMessagesScroll}
            onSelectManifoldTurn={selectManifoldTurnFromTranscript}
            projection={projection}
            selectedManifoldTurnId={selectedManifoldTurnId}
          />
          <ChatComposer
            composerParts={composerParts}
            onPromptChange={(event) => setPrompt(event.target.value)}
            onPromptKeyDown={onPromptKeyDown}
            onPromptPaste={onPromptPaste}
            onSubmit={onSubmit}
            prompt={prompt}
            sending={sending}
          />
        </div>

        <div className="trace-panel">
          <div className="trace-header-wrap">
            <div>
              <div className="trace-header">Transit Trace</div>
              <div className="trace-subhead" id="trace-subhead">
                {TRACE_VIEW_LABELS[activeView]}
              </div>
            </div>
            <div className="trace-tabs">
              <Link className={`trace-tab${activeView === 'inspector' ? ' is-active' : ''}`} to="/">
                Inspector
              </Link>
              <Link
                className={`trace-tab${activeView === 'manifold' ? ' is-active' : ''}`}
                to="/manifold"
              >
                Manifold
              </Link>
              <Link
                className={`trace-tab${activeView === 'transit' ? ' is-active' : ''}`}
                to="/transit"
              >
                Transit
              </Link>
            </div>
          </div>
          <Outlet />
        </div>
      </>
    </ManifoldTurnSelectionProvider>
  );
}

function InspectorRoute() {
  const { projection } = useRuntimeStore();
  const turns = projection?.forensics.turns || [];
  const [selectedTurnId, setSelectedTurnId] = useState<string | null>(null);
  const [selectedRecordId, setSelectedRecordId] = useState<string | null>(null);
  const [selectionMode, setSelectionMode] = useState<'conversation' | 'turn' | 'record'>('record');
  const [detailMode, setDetailMode] = useState<'rendered' | 'raw'>('rendered');
  const [focus, setFocus] = useState<FocusState>({ kind: 'all', id: null });

  const currentTurn =
    turns.find((turn) => turn.turn_id === selectedTurnId) || turns[turns.length - 1] || null;
  const records = recordsForTurn(currentTurn, focus);
  const currentRecord =
    selectionMode === 'record'
      ? records.find((record) => record.record.record_id === selectedRecordId) ||
        records[records.length - 1] ||
        null
      : null;
  const modelCalls = modelCallsForTurn(currentTurn);
  const plannerSteps = plannerStepsForTurn(currentTurn);
  const signalRecords = currentTurn
    ? currentTurn.records.filter((record) => kindEntry(record).key === 'SignalSnapshot')
    : [];
  const strongestSignal = strongestSignalSnapshot(signalRecords);
  const strongestSignalValue = strongestSignal ? kindEntry(strongestSignal).value : null;
  const contributions = aggregateSignalContributions(signalRecords);
  const baseline = previousArtifactBaseline(
    currentTurn,
    currentRecord || latestRecordForTurn(currentTurn, focus)
  );
  const comparisonRecord = currentRecord || latestRecordForTurn(currentTurn, focus);

  useEffect(() => {
    if (!turns.length) {
      setSelectedTurnId(null);
      setSelectedRecordId(null);
      return;
    }
    if (!selectedTurnId || !turns.some((turn) => turn.turn_id === selectedTurnId)) {
      const lastTurn = turns[turns.length - 1];
      setSelectedTurnId(lastTurn.turn_id);
      if (lastTurn.records.length) {
        setSelectedRecordId(lastTurn.records[lastTurn.records.length - 1].record.record_id);
      }
    }
  }, [selectedTurnId, turns]);

  useEffect(() => {
    if (selectionMode !== 'record') {
      return;
    }
    if (records.length && !records.some((record) => record.record.record_id === selectedRecordId)) {
      setSelectedRecordId(records[records.length - 1].record.record_id);
    }
  }, [records, selectedRecordId, selectionMode]);

  return (
    <div className="trace-view trace-view--active forensic-view" id="forensic-view">
      <div className="forensic-overview" id="forensic-overview">
        <section className="forensic-overview-card" id="forensic-topology-overview">
          {!turns.length ? (
            <div className="forensic-empty">Forensic replay appears here when transit records exist.</div>
          ) : (
            <>
              <div className="forensic-card-title">Topology</div>
              <dl className="forensic-topology-metrics">
                <div className="forensic-topology-metric">
                  <dt>Turns</dt>
                  <dd>{turns.length}</dd>
                </div>
                <div className="forensic-topology-metric">
                  <dt>Records</dt>
                  <dd>{currentTurn ? currentTurn.records.length : 0}</dd>
                </div>
                <div className="forensic-topology-metric">
                  <dt>Scope</dt>
                  <dd>{focus.kind === 'all' ? 'all records' : focus.kind.replace('_', ' ')}</dd>
                </div>
                <div className="forensic-topology-metric">
                  <dt>Selection</dt>
                  <dd>{comparisonRecord ? truncate(recordSummary(comparisonRecord), 20) : 'turn'}</dd>
                </div>
              </dl>
              <div className="forensic-topology-legend">
                <span className="forensic-chip">
                  <span
                    className="forensic-chip-swatch"
                    style={{ ['--chip-color' as string]: KIND_COLORS.forensic }}
                  />
                  lineage path
                </span>
                <span className="forensic-chip">
                  <span
                    className="forensic-chip-swatch"
                    style={{ ['--chip-color' as string]: KIND_COLORS.action }}
                  />
                  model/tool state
                </span>
                <span className="forensic-chip">
                  <span
                    className="forensic-chip-swatch"
                    style={{ ['--chip-color' as string]: KIND_COLORS.signal }}
                  />
                  steering signals
                </span>
              </div>
            </>
          )}
        </section>

        <section className="forensic-overview-card" id="forensic-signal-overview">
          {!signalRecords.length || !strongestSignalValue ? (
            <>
              <div className="forensic-card-title">Steering Signals</div>
              <div className="forensic-empty">
                No steering signals were recorded for the current lineage selection.
              </div>
            </>
          ) : (
            <>
              <div className="forensic-card-title">Steering Signals</div>
              <div className="forensic-signal-hero">
                <div
                  className="forensic-signal-gauge"
                  style={{
                    ['--signal-color' as string]:
                      FORCE_LEVEL_COLORS[String(strongestSignalValue.level)] ||
                      FORCE_KIND_COLORS[String(strongestSignalValue.kind)] ||
                      KIND_COLORS.signal,
                    ['--signal-sweep' as string]: `${Number(
                      strongestSignalValue.magnitude_percent || 0
                    )}%`,
                  }}
                >
                  <div className="forensic-signal-gauge-content">
                    <div className="forensic-signal-gauge-value">
                      {Number(strongestSignalValue.magnitude_percent || 0)}%
                    </div>
                    <div className="forensic-signal-gauge-label">
                      {String(strongestSignalValue.level || 'unknown')}
                    </div>
                  </div>
                </div>
                <div className="forensic-signal-summary">
                  <div className="forensic-signal-summary-row">
                    <strong>{signalKindLabel(String(strongestSignalValue.kind || 'signal'))}</strong>
                    <span>{signalRecords.length} snapshots</span>
                  </div>
                  <div className="forensic-contribs forensic-contribs--stacked">
                    {contributions.slice(0, 5).map((contribution) => (
                      <span
                        className="forensic-chip"
                        key={`${contribution.source}-${contribution.percent}`}
                        title={contribution.rationale}
                      >
                        <span
                          className="forensic-chip-swatch"
                          style={{ ['--chip-color' as string]: sourceColor(contribution.source) }}
                        />
                        {contribution.label} {contribution.percent}%
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            </>
          )}
        </section>

        <section className="forensic-overview-card" id="forensic-shadow-overview">
          <div className="forensic-card-title">Shadow Baseline</div>
          {!comparisonRecord ? (
            <div className="forensic-empty">
              Select a lineage artifact to compare it with the previous artifact in lineage.
            </div>
          ) : !baseline ? (
            <div className="forensic-empty">
              No previous artifact in lineage was available for this selection yet.
            </div>
          ) : (
            <>
              <div className="forensic-shadow-compare">
                <div className="forensic-shadow-pane">
                  <div className="forensic-shadow-pane-label">Current</div>
                  <div className="forensic-shadow-pane-title">
                    {recordSummary(comparisonRecord)}
                  </div>
                  <div className="forensic-shadow-pane-meta">{recordMeta(comparisonRecord)}</div>
                  <div className="forensic-shadow-pane-snippet">
                    {comparisonSnippet(comparisonRecord, detailMode)}
                  </div>
                </div>
                <div className="forensic-shadow-pane is-baseline">
                  <div className="forensic-shadow-pane-label">Baseline</div>
                  <div className="forensic-shadow-pane-title">{recordSummary(baseline)}</div>
                  <div className="forensic-shadow-pane-meta">{recordMeta(baseline)}</div>
                  <div className="forensic-shadow-pane-snippet">
                    {comparisonSnippet(baseline, detailMode)}
                  </div>
                </div>
              </div>
            </>
          )}
        </section>
      </div>

      <div className="forensic-shell">
        <aside className="forensic-nav" id="forensic-nav">
          {!turns.length ? (
            <div className="forensic-empty-state">No forensic replay is available yet.</div>
          ) : (
            <>
              <div className="forensic-nav-group">
                <div className="forensic-nav-group-title">Conversation</div>
                <button
                  className={`forensic-nav-button${selectionMode === 'conversation' ? ' is-active' : ''}`}
                  id="forensic-conversation-button"
                  onClick={() => setSelectionMode('conversation')}
                  type="button"
                >
                  <div className="forensic-nav-title">
                    <span>{projection?.forensics.task_id || 'conversation'}</span>
                    <span>{turns.length}</span>
                  </div>
                  <div className="forensic-nav-meta">turns · replay-backed lineage surface</div>
                </button>
              </div>

              <div className="forensic-nav-group">
                <div className="forensic-nav-group-title">Turns</div>
                {turns.map((turn) => (
                  <button
                    className={`forensic-nav-button${
                      currentTurn?.turn_id === turn.turn_id && focus.kind === 'all'
                        ? ' is-active'
                        : ''
                    } is-${turn.lifecycle}`}
                    data-turn-id={turn.turn_id}
                    key={turn.turn_id}
                    onClick={() => {
                      setSelectedTurnId(turn.turn_id);
                      setSelectedRecordId(turn.records[turn.records.length - 1]?.record.record_id || null);
                      setSelectionMode('turn');
                      setFocus({ kind: 'all', id: null });
                    }}
                    type="button"
                  >
                    <div className="forensic-nav-title">
                      <span>{truncate(turn.turn_id, 28)}</span>
                      <span className={`forensic-lifecycle is-${turn.lifecycle}`}>
                        {lifecycleLabel(turn.lifecycle)}
                      </span>
                    </div>
                    <div className="forensic-nav-meta">{turn.records.length} records</div>
                  </button>
                ))}
              </div>

              <div className="forensic-nav-group">
                <div className="forensic-nav-group-title">Focus</div>
                <button
                  className={`forensic-nav-button${focus.kind === 'all' ? ' is-active' : ''}`}
                  onClick={() => {
                    setFocus({ kind: 'all', id: null });
                    setSelectionMode('turn');
                  }}
                  type="button"
                >
                  <div className="forensic-nav-title">
                    <span>All records</span>
                    <span>{currentTurn?.records.length || 0}</span>
                  </div>
                  <div className="forensic-nav-meta">full turn sequence</div>
                </button>
                {modelCalls.map((modelCall) => (
                  <button
                    className={`forensic-nav-button${
                      focus.kind === 'model_call' && focus.id === modelCall.id ? ' is-active' : ''
                    }`}
                    key={modelCall.id}
                    onClick={() => {
                      setFocus({ kind: 'model_call', id: modelCall.id });
                      setSelectionMode('turn');
                    }}
                    type="button"
                  >
                    <div className="forensic-nav-title">
                      <span>{truncate(modelCall.summary, 34)}</span>
                      <span>{modelCall.lane}</span>
                    </div>
                    <div className="forensic-nav-meta">
                      {modelCall.provider}:{modelCall.model} · {modelCall.category}
                    </div>
                  </button>
                ))}
                {plannerSteps.map((step) => (
                  <button
                    className={`forensic-nav-button${
                      focus.kind === 'planner_step' && focus.id === step.id ? ' is-active' : ''
                    }`}
                    key={step.id}
                    onClick={() => {
                      setFocus({ kind: 'planner_step', id: step.id });
                      setSelectionMode('turn');
                    }}
                    type="button"
                  >
                    <div className="forensic-nav-title">
                      <span>{step.label}</span>
                      <span>step</span>
                    </div>
                    <div className="forensic-nav-meta">{step.recordId}</div>
                  </button>
                ))}
              </div>
            </>
          )}
        </aside>

        <div className="forensic-main">
          <section className="forensic-records" id="forensic-records">
            {!currentTurn ? (
              <div className="forensic-empty-state">
                Select a turn to inspect its transit lineage.
              </div>
            ) : (
              <>
                <div className="forensic-section-head">
                  <div className="forensic-section-title">{currentTurn.turn_id}</div>
                  <div className="forensic-section-meta">{records.length} records</div>
                </div>
                {records.length ? (
                  records.map((recordProjection) => {
                    const entry = kindEntry(recordProjection);
                    const artifact = primaryArtifact(recordProjection);
                    return (
                      <button
                        className={`forensic-record${
                          currentRecord?.record.record_id === recordProjection.record.record_id
                            ? ' is-selected'
                            : ''
                        }${
                          recordProjection.lifecycle === 'superseded' ? ' is-superseded' : ''
                        }`}
                        data-record-id={recordProjection.record.record_id}
                        key={recordProjection.record.record_id}
                        onClick={() => {
                          setSelectionMode('record');
                          setSelectedRecordId(recordProjection.record.record_id);
                        }}
                        type="button"
                      >
                        <div className="forensic-record-head">
                          <div className="forensic-record-title">
                            {recordSummary(recordProjection)}
                          </div>
                          <span className={`forensic-lifecycle is-${recordProjection.lifecycle}`}>
                            {lifecycleLabel(recordProjection.lifecycle)}
                          </span>
                        </div>
                        <div className="forensic-record-subtitle">
                          {recordMeta(recordProjection)}
                        </div>
                        <div className="forensic-pill-row">
                          <span className="forensic-pill">{kindLabel(entry.key)}</span>
                          {artifact?.mime_type ? (
                            <span className="forensic-pill">{artifact.mime_type}</span>
                          ) : null}
                          {recordProjection.superseded_by_record_id ? (
                            <span className="forensic-pill">
                              superseded by {recordProjection.superseded_by_record_id}
                            </span>
                          ) : null}
                        </div>
                      </button>
                    );
                  })
                ) : (
                  <div className="forensic-empty-state">
                    No records match the current lineage focus.
                  </div>
                )}
              </>
            )}
          </section>

          <section className="forensic-detail-pane">
            <div className="forensic-detail-toolbar">
              <div>
                <div className="forensic-detail-title" id="forensic-detail-title">
                  {!turns.length
                    ? 'No selection'
                    : selectionMode === 'conversation'
                      ? projection?.forensics.task_id
                      : currentRecord
                        ? recordSummary(currentRecord)
                        : currentTurn?.turn_id}
                </div>
                <div className="forensic-detail-meta" id="forensic-detail-meta">
                  {!turns.length
                    ? 'Transit-backed forensic details appear here.'
                    : selectionMode === 'conversation'
                      ? `Context-lineage-first replay for ${turns.length} turns.`
                      : currentRecord
                        ? recordMeta(currentRecord)
                        : `Turn summary · ${currentTurn?.records.length || 0} records · ${lifecycleLabel(currentTurn?.lifecycle)}`}
                </div>
              </div>
              <div className="forensic-toggle-row">
                <button
                  className={`forensic-toggle${detailMode === 'rendered' ? ' is-active' : ''}`}
                  data-detail-mode="rendered"
                  onClick={() => setDetailMode('rendered')}
                  type="button"
                >
                  Rendered
                </button>
                <button
                  className={`forensic-toggle${detailMode === 'raw' ? ' is-active' : ''}`}
                  data-detail-mode="raw"
                  onClick={() => setDetailMode('raw')}
                  type="button"
                >
                  Raw
                </button>
              </div>
            </div>
            <div className="forensic-detail-body" id="forensic-detail">
              {!turns.length ? (
                <div className="forensic-empty-state">
                  Submit a prompt or wait for a trace-producing turn to inspect raw and rendered
                  model context.
                </div>
              ) : selectionMode === 'conversation' ? (
                <>
                  <div className="forensic-detail-card">
                    <div className="forensic-detail-card-title">Conversation Summary</div>
                    <dl className="forensic-detail-grid">
                      <div>
                        <dt>Task</dt>
                        <dd>{projection?.forensics.task_id}</dd>
                      </div>
                      <div>
                        <dt>Turns</dt>
                        <dd>{turns.length}</dd>
                      </div>
                      <div>
                        <dt>Records</dt>
                        <dd>{turns.reduce((sum, turn) => sum + turn.records.length, 0)}</dd>
                      </div>
                      <div>
                        <dt>Latest Turn</dt>
                        <dd>{turns[turns.length - 1]?.turn_id || 'none'}</dd>
                      </div>
                    </dl>
                  </div>
                  <div className="forensic-detail-card">
                    <div className="forensic-detail-card-title">Navigation</div>
                    <div className="forensic-inline-note">
                      Choose a turn on the left, then narrow the lineage focus to model calls or
                      planner steps before drilling into an exact trace record.
                    </div>
                  </div>
                </>
              ) : !currentRecord ? (
                <>
                  <div className="forensic-detail-card">
                    <div className="forensic-detail-card-title">Turn Summary</div>
                    <dl className="forensic-detail-grid">
                      <div>
                        <dt>Lifecycle</dt>
                        <dd>{lifecycleLabel(currentTurn?.lifecycle)}</dd>
                      </div>
                      <div>
                        <dt>Records</dt>
                        <dd>{currentTurn?.records.length || 0}</dd>
                      </div>
                      <div>
                        <dt>Model Calls</dt>
                        <dd>{modelCalls.length}</dd>
                      </div>
                      <div>
                        <dt>Planner Steps</dt>
                        <dd>{plannerSteps.length}</dd>
                      </div>
                    </dl>
                  </div>
                  <div className="forensic-detail-card">
                    <div className="forensic-detail-card-title">Lineage Scope</div>
                    <div className="forensic-inline-note">
                      Current focus:{' '}
                      {focus.kind === 'all' ? 'all records' : `${focus.kind} ${focus.id || ''}`}
                    </div>
                    <pre className="forensic-code">
                      {(currentTurn?.records || [])
                        .map((record) => `[${record.record.sequence}] ${recordSummary(record)}`)
                        .join('\n')}
                    </pre>
                  </div>
                </>
              ) : (
                <>
                  <div className="forensic-detail-card">
                    <div className="forensic-detail-card-title">Record Metadata</div>
                    <dl className="forensic-detail-grid">
                      <div>
                        <dt>Record</dt>
                        <dd>{currentRecord.record.record_id}</dd>
                      </div>
                      <div>
                        <dt>Sequence</dt>
                        <dd>{currentRecord.record.sequence}</dd>
                      </div>
                      <div>
                        <dt>Kind</dt>
                        <dd>{kindLabel(kindEntry(currentRecord).key)}</dd>
                      </div>
                      <div>
                        <dt>Lifecycle</dt>
                        <dd>{lifecycleLabel(currentRecord.lifecycle)}</dd>
                      </div>
                      <div>
                        <dt>Turn</dt>
                        <dd>{currentRecord.record.lineage.turn_id}</dd>
                      </div>
                      <div>
                        <dt>Branch</dt>
                        <dd>{currentRecord.record.lineage.branch_id || 'mainline'}</dd>
                      </div>
                    </dl>
                  </div>
                  <div className="forensic-detail-card">
                    <div className="forensic-detail-card-title">
                      Payload ({detailMode})
                    </div>
                    <div className="forensic-inline-note">
                      mime: {primaryArtifact(currentRecord)?.mime_type || 'record/json'}
                      {primaryArtifact(currentRecord)?.truncated ? ' · truncated' : ''}
                    </div>
                    <pre className={detailMode === 'raw' ? 'forensic-raw' : 'forensic-code'}>
                      {detailMode === 'raw'
                        ? rawRecordBody(currentRecord)
                        : renderedRecordBody(currentRecord)}
                    </pre>
                  </div>
                  {baseline ? (
                    <div className="forensic-detail-card">
                      <div className="forensic-detail-card-title">Shadow Baseline</div>
                      <dl className="forensic-detail-grid">
                        <div>
                          <dt>Current</dt>
                          <dd>{recordSummary(currentRecord)}</dd>
                        </div>
                        <div>
                          <dt>Baseline</dt>
                          <dd>{recordSummary(baseline)}</dd>
                        </div>
                        <div>
                          <dt>Delta chars</dt>
                          <dd>
                            {(
                              (detailMode === 'raw'
                                ? rawRecordBody(currentRecord)
                                : renderedRecordBody(currentRecord)
                              ).length -
                              (
                                detailMode === 'raw'
                                  ? rawRecordBody(baseline)
                                  : renderedRecordBody(baseline)
                              ).length
                            ).toString()}
                          </dd>
                        </div>
                        <div>
                          <dt>Lineage gap</dt>
                          <dd>{currentRecord.record.sequence - baseline.record.sequence} steps</dd>
                        </div>
                      </dl>
                    </div>
                  ) : null}
                </>
              )}
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}

type ManifoldForcePoint = {
  key: string;
  gate: string;
  laneIndex: number;
  frameIndex: number;
  sequence: number;
  leftPercent: number;
  topPercent: number;
  magnitudePercent: number;
  phase: string;
  level: string;
  dominantRecordId: string | null;
  dominantSignalKind: string;
  label: string;
};

type ManifoldForceLink = {
  key: string;
  gate: string;
  leftPercent: number;
  topPercent: number;
  widthPercent: number;
  magnitudePercent: number;
};

type ManifoldCameraState = {
  pitch: number;
  yaw: number;
  roll: number;
  panX: number;
  panY: number;
  zoom: number;
};

type ManifoldDragState = {
  mode: 'tilt' | 'pan' | 'rotate';
  startX: number;
  startY: number;
  origin: ManifoldCameraState;
};

const DEFAULT_MANIFOLD_CAMERA: ManifoldCameraState = {
  pitch: 62,
  yaw: -18,
  roll: 0,
  panX: 0,
  panY: 0,
  zoom: 1,
};

function laneIndexForGate(gate: string) {
  const index = STEERING_GATE_ORDER.indexOf(gate);
  return index >= 0 ? index : STEERING_GATE_ORDER.length - 1;
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function buildGateField(turn: ManifoldTurnProjection | null): {
  points: ManifoldForcePoint[];
  links: ManifoldForceLink[];
} {
  if (!turn || !turn.frames.length) {
    return { points: [], links: [] };
  }

  const points: ManifoldForcePoint[] = [];
  const pointsByGate = new Map<string, ManifoldForcePoint[]>();
  const lastFrameIndex = Math.max(1, turn.frames.length - 1);

  turn.frames.forEach((frame, frameIndex) => {
    frame.gates.forEach((gate) => {
      const normalizedGate = gate.gate || 'containment';
      const laneIndex = laneIndexForGate(normalizedGate);
      const point: ManifoldForcePoint = {
        key: `${frame.record_id}:${normalizedGate}`,
        gate: normalizedGate,
        laneIndex,
        frameIndex,
        sequence: frame.sequence,
        leftPercent: 12 + (frameIndex / lastFrameIndex) * 76,
        topPercent: 18 + laneIndex * 28,
        magnitudePercent: gate.magnitude_percent || 0,
        phase: gate.phase,
        level: gate.level,
        dominantRecordId: gate.dominant_record_id || null,
        dominantSignalKind: gate.dominant_signal_kind,
        label: manifoldGateLabel(gate),
      };
      points.push(point);
      const gatePoints = pointsByGate.get(normalizedGate) || [];
      gatePoints.push(point);
      pointsByGate.set(normalizedGate, gatePoints);
    });
  });

  const links: ManifoldForceLink[] = [];
  for (const [gate, gatePoints] of pointsByGate.entries()) {
    for (let index = 0; index < gatePoints.length - 1; index += 1) {
      const current = gatePoints[index];
      const next = gatePoints[index + 1];
      links.push({
        key: `${current.key}->${next.key}`,
        gate,
        leftPercent: current.leftPercent,
        topPercent: current.topPercent,
        widthPercent: Math.max(3, next.leftPercent - current.leftPercent),
        magnitudePercent: Math.round((current.magnitudePercent + next.magnitudePercent) / 2),
      });
    }
  }

  return { points, links };
}

function frameForSelectedRecord(
  turn: ManifoldTurnProjection | null,
  recordId: string | null
): ManifoldFrame | null {
  if (!turn || !recordId) {
    return null;
  }
  return (
    turn.frames.find((frame) =>
      frame.active_signals.some((signal) => signal.snapshot_record_id === recordId)
    ) || null
  );
}

function ManifoldRoute() {
  const { projection } = useRuntimeStore();
  const turns = projection?.manifold.turns || [];
  const { selectedTurnId, setSelectedTurnId } = useManifoldTurnSelection();
  const [frameIndex, setFrameIndex] = useState<number | null>(null);
  const [tailMode, setTailMode] = useState(true);
  const [playing, setPlaying] = useState(false);
  const [selectedSourceRecordId, setSelectedSourceRecordId] = useState<string | null>(null);
  const [camera, setCamera] = useState<ManifoldCameraState>(DEFAULT_MANIFOLD_CAMERA);
  const [dragMode, setDragMode] = useState<'tilt' | 'pan' | 'rotate' | null>(null);
  const dragStateRef = useRef<ManifoldDragState | null>(null);

  const currentTurn =
    turns.find((turn) => turn.turn_id === selectedTurnId) || turns[turns.length - 1] || null;
  const effectiveFrameIndex = currentTurn
    ? tailMode
      ? Math.max(0, currentTurn.frames.length - 1)
      : Math.max(0, Math.min(currentTurn.frames.length - 1, frameIndex ?? 0))
    : 0;
  const currentFrame = currentTurn?.frames[effectiveFrameIndex] || null;
  const activeSignals = currentFrame?.active_signals || [];
  const selectedSignal =
    activeSignals.find((signal) => signal.snapshot_record_id === selectedSourceRecordId) ||
    activeSignals[0] ||
    null;
  const totalFrames = turns.reduce((sum, turn) => sum + turn.frames.length, 0);
  const selectedSignalFrame = frameForSelectedRecord(currentTurn, selectedSourceRecordId) || currentFrame;
  const selectedGate =
    currentFrame?.gates.find((gate) => gate.dominant_record_id === selectedSourceRecordId) ||
    currentFrame?.gates.find((gate) => gate.gate === selectedSignal?.gate) ||
    currentFrame?.gates[0] ||
    null;
  const selectedResolverOutcome = resolverSignalDetails(selectedSignal);
  const gateField = useMemo(() => buildGateField(currentTurn), [currentTurn]);

  useEffect(() => {
    setTailMode(true);
    setFrameIndex(null);
    setSelectedSourceRecordId(null);
    setPlaying(false);
  }, [selectedTurnId]);

  useEffect(() => {
    if (!playing || !currentTurn || currentTurn.frames.length <= 1) {
      return;
    }
    const handle = window.setInterval(() => {
      setTailMode(false);
      setFrameIndex((current) => {
        const next = (current ?? 0) + 1;
        if (next >= currentTurn.frames.length) {
          return 0;
        }
        return next;
      });
    }, MANIFOLD_PLAYBACK_STEP_MS);
    return () => window.clearInterval(handle);
  }, [currentTurn, playing]);

  useEffect(() => {
    function stopDragging() {
      dragStateRef.current = null;
      setDragMode(null);
    }

    function handleMouseMove(event: MouseEvent) {
      const dragState = dragStateRef.current;
      if (!dragState) {
        return;
      }

      const dx = event.clientX - dragState.startX;
      const dy = event.clientY - dragState.startY;

      if (dragState.mode === 'pan') {
        setCamera({
          ...dragState.origin,
          panX: clamp(dragState.origin.panX + dx, -320, 320),
          panY: clamp(dragState.origin.panY + dy, -220, 220),
        });
        return;
      }

      if (dragState.mode === 'rotate') {
        setCamera({
          ...dragState.origin,
          roll: clamp(dragState.origin.roll + dx * 0.28, -85, 85),
        });
        return;
      }

      setCamera({
        ...dragState.origin,
        pitch: clamp(dragState.origin.pitch - dy * 0.28, 6, 96),
        yaw: clamp(dragState.origin.yaw + dx * 0.32, -88, 88),
      });
    }

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', stopDragging);
    window.addEventListener('blur', stopDragging);

    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', stopDragging);
      window.removeEventListener('blur', stopDragging);
    };
  }, []);

  function resetCamera() {
    dragStateRef.current = null;
    setDragMode(null);
    setCamera(DEFAULT_MANIFOLD_CAMERA);
  }

  function beginCameraDrag(
    event: React.MouseEvent<HTMLDivElement>,
    mode: 'tilt' | 'pan' | 'rotate'
  ) {
    if (event.button !== 0 && event.button !== 1) {
      return;
    }
    event.preventDefault();
    dragStateRef.current = {
      mode,
      startX: event.clientX,
      startY: event.clientY,
      origin: camera,
    };
    setDragMode(mode);
  }

  return (
    <div className="trace-view trace-view--active trace-view--manifold manifold-view" id="manifold-view">
      <div className="manifold-shell" id="manifold-shell">
        <section className="manifold-stage">
          <div className="manifold-stage-head">
            <div>
              <div className="manifold-stage-title">Steering Gate Manifold</div>
              <div className="manifold-stage-meta" id="manifold-stage-meta">
                {!turns.length
                  ? 'Awaiting replay-backed manifold frames'
                  : `${projection?.manifold.task_id || 'task'} · ${turns.length} turns · ${totalFrames} frames · selected ${
                      effectiveFrameIndex + 1
                    }`}
              </div>
            </div>
            <div className="manifold-stage-controls">
              <button
                className="trace-tab manifold-stage-button"
                id="manifold-play-toggle"
                onClick={() => setPlaying((current) => !current)}
                type="button"
              >
                {playing ? 'Pause' : 'Play'}
              </button>
              <button
                className="trace-tab manifold-stage-button"
                id="manifold-replay-button"
                onClick={() => {
                  setTailMode(false);
                  setFrameIndex(0);
                  setPlaying(false);
                }}
                type="button"
              >
                Replay
              </button>
              <button
                className="trace-tab manifold-stage-button"
                id="manifold-reset-view-button"
                onClick={resetCamera}
                type="button"
              >
                Reset View
              </button>
            </div>
          </div>
          <div className="manifold-stage-timeline">
            <input
              id="manifold-time-scrubber"
              max={Math.max(0, (currentTurn?.frames.length || 1) - 1)}
              min="0"
              onChange={(event) => {
                setTailMode(false);
                setFrameIndex(Number(event.target.value));
              }}
              type="range"
              value={effectiveFrameIndex}
            />
            <div className="manifold-stage-frame-meta" id="manifold-frame-meta">
              Frame {currentTurn ? effectiveFrameIndex + 1 : 0} / {currentTurn?.frames.length || 0}
            </div>
          </div>
          <div className="manifold-canvas" id="manifold-canvas">
            {!turns.length ? (
              <div className="manifold-empty-state">
                <strong>Steering gate manifold route is armed.</strong>
                <p>
                  Once replay-backed steering snapshots arrive, the temporal gate field will
                  populate here.
                </p>
              </div>
            ) : (
              <div className="manifold-machine">
                <div className="manifold-playback-banner">
                  <strong>Temporal gate playback is active.</strong>
                  <p>
                    Time sweeps left to right, gate families stack top to bottom, and force rises
                    toward you with magnitude.
                    <br />
                    Current turn: {currentTurn?.turn_id || 'none'} · frame sequence{' '}
                    {currentFrame?.sequence || 'none'}
                    <br />
                    Active gates: {currentFrame?.gates.length || 0} · sources:{' '}
                    {currentFrame?.active_signals.length || 0} · projected anchors:{' '}
                    {currentFrame?.primitives.length || 0}
                  </p>
                </div>
                <div className="manifold-spacefield">
                  <div className="manifold-spacefield__meta">
                    <span>Temporal gate field</span>
                    <span>turn anchor {manifoldAnchorLabel(currentFrame?.anchor)}</span>
                  </div>
                  <div
                    className={`manifold-spacefield__viewport${
                      dragMode ? ` is-dragging is-${dragMode}` : ''
                    }`}
                    data-testid="manifold-spacefield-viewport"
                    onDoubleClick={resetCamera}
                    onMouseDown={(event) =>
                      beginCameraDrag(
                        event,
                        event.altKey
                          ? 'rotate'
                          : event.shiftKey || event.button === 1
                            ? 'pan'
                            : 'tilt'
                      )
                    }
                    onWheel={(event) => {
                      event.preventDefault();
                      event.stopPropagation();
                      setCamera((current) => ({
                        ...current,
                        zoom: clamp(
                          current.zoom * Math.exp(-event.deltaY * 0.0012),
                          0.68,
                          2.2
                        ),
                      }));
                    }}
                  >
                    <div className="manifold-spacefield__axes">
                      <div className="manifold-spacefield__axis manifold-spacefield__axis--gate">
                        Gate family
                      </div>
                      <div className="manifold-spacefield__axis manifold-spacefield__axis--time">
                        Time
                      </div>
                      <div className="manifold-spacefield__axis manifold-spacefield__axis--force">
                        Force
                      </div>
                    </div>
                    <div className="manifold-spacefield__hint">
                      Drag to tilt · Alt+drag to rotate · Shift+drag to pan · Wheel to zoom · Double-click to reset
                    </div>
                    <div className="manifold-spacefield__camera-stats">
                      pan {Math.round(camera.panX)},{Math.round(camera.panY)} · tilt{' '}
                      {Math.round(camera.pitch)}°/{Math.round(camera.yaw)}° · roll{' '}
                      {Math.round(camera.roll)}° · zoom {camera.zoom.toFixed(2)}x
                    </div>
                    <div
                      className="manifold-spacefield__deck"
                      data-pan-x={Math.round(camera.panX).toString()}
                      data-pan-y={Math.round(camera.panY).toString()}
                      data-pitch={Math.round(camera.pitch).toString()}
                      data-roll={Math.round(camera.roll).toString()}
                      data-testid="manifold-spacefield-deck"
                      data-yaw={Math.round(camera.yaw).toString()}
                      data-zoom={camera.zoom.toFixed(2)}
                      style={{
                        transform: `translate(${camera.panX}px, ${camera.panY}px) scale(${camera.zoom}) rotateX(${camera.pitch}deg) rotateY(${camera.yaw}deg) rotateZ(${camera.roll}deg)`,
                      } as React.CSSProperties}
                    >
                      <div className="manifold-spacefield__floor" />
                      {STEERING_GATE_ORDER.map((gate, laneIndex) => (
                        <div
                          className={`manifold-gate-lane is-${steeringGateClass(gate)}`}
                          key={gate}
                          style={{ top: `${18 + laneIndex * 28}%` } as React.CSSProperties}
                        >
                          <div className="manifold-gate-lane__label">{steeringGateLabel(gate)}</div>
                          <div className="manifold-gate-lane__rail" />
                        </div>
                      ))}
                      {gateField.links.map((link) => (
                        <div
                          className={`manifold-force-link is-${steeringGateClass(link.gate)}`}
                          key={link.key}
                          style={{
                            left: `${link.leftPercent}%`,
                            top: `${link.topPercent}%`,
                            width: `${link.widthPercent}%`,
                            transform: `translate3d(0, -50%, ${Math.max(
                              12,
                              link.magnitudePercent * 1.2
                            )}px)`,
                          } as React.CSSProperties}
                        />
                      ))}
                      {gateField.points.map((point) => {
                        const isCurrent = point.frameIndex === effectiveFrameIndex;
                        const isSelected = point.dominantRecordId === selectedSourceRecordId;
                        return (
                          <button
                            className={`manifold-force-point is-${steeringGateClass(point.gate)}${
                              isCurrent ? ' is-current' : ''
                            }${isSelected ? ' is-selected' : ''}`}
                            key={point.key}
                            onMouseDown={(event) => event.stopPropagation()}
                            onClick={() => {
                              setTailMode(false);
                              setFrameIndex(point.frameIndex);
                              setSelectedSourceRecordId(point.dominantRecordId);
                            }}
                            style={{
                              left: `${point.leftPercent}%`,
                              top: `${point.topPercent}%`,
                              transform: `translate3d(-50%, -50%, ${Math.max(
                                18,
                                point.magnitudePercent * 1.4
                              )}px)`,
                              ['--gate-color' as string]:
                                STEERING_GATE_COLORS[point.gate] || STEERING_GATE_COLORS.containment,
                            } as React.CSSProperties}
                            type="button"
                          >
                            <span className="manifold-force-point__halo" />
                            <span className="manifold-force-point__core" />
                            <span className="manifold-force-point__label">
                              {point.label} · {point.magnitudePercent}%
                            </span>
                            <span className="sr-only">
                              {point.label}, frame {point.frameIndex + 1}, {point.magnitudePercent}%
                            </span>
                          </button>
                        );
                      })}
                    </div>
                  </div>
                  <div className="manifold-readout">
                    <div
                      className={`manifold-readout-card is-${
                        selectedGate ? steeringGateClass(selectedGate.gate) : 'containment'
                      }`}
                    >
                      <div className="manifold-readout-card__eyebrow">
                        <span>Selected gate</span>
                        <span>
                          {selectedGate ? steeringPhaseLabel(selectedGate.phase) : 'Idle'}
                        </span>
                      </div>
                      <div className="manifold-readout-card__title">
                        {selectedGate ? manifoldGateLabel(selectedGate) : 'No active gate'}
                      </div>
                      <div className="manifold-readout-card__meta">
                        {selectedGate
                          ? `${selectedGate.magnitude_percent}% · ${signalKindLabel(
                              selectedGate.dominant_signal_kind
                            )} · ${selectedGate.level}`
                          : 'Awaiting replay-backed steering state.'}
                      </div>
                    </div>
                    <div
                      className={`manifold-readout-card is-${
                        selectedSignal ? steeringGateClass(selectedSignal.gate) : 'containment'
                      }`}
                    >
                      <div className="manifold-readout-card__eyebrow">
                        <span>Selected source</span>
                        <span>
                          {selectedSignalFrame
                            ? `Frame ${selectedSignalFrame.sequence}`
                            : 'No source'}
                        </span>
                      </div>
                      <div className="manifold-readout-card__title">
                        {selectedSignal?.summary || 'Select a force point or gate card'}
                      </div>
                      <div className="manifold-readout-card__meta">
                        {selectedSignal
                          ? `${manifoldSignalLabel(selectedSignal)} · ${steeringGateLabel(
                              selectedSignal.gate
                            )} · ${steeringPhaseLabel(selectedSignal.phase)}`
                          : 'The readout follows the selected orbit in the field.'}
                      </div>
                    </div>
                    <div
                      className={`manifold-readout-card is-${
                        selectedSignal ? steeringGateClass(selectedSignal.gate) : 'containment'
                      }`}
                    >
                      <div className="manifold-readout-card__eyebrow">
                        <span>Resolver outcome</span>
                        <span>
                          {selectedResolverOutcome
                            ? selectedResolverOutcome.status
                            : 'No resolver signal'}
                        </span>
                      </div>
                      <div className="manifold-readout-card__title">
                        {selectedResolverOutcome
                          ? resolverOutcomeTitle(selectedResolverOutcome)
                          : 'Select an entity-resolution force point'}
                      </div>
                      <div className="manifold-readout-card__meta">
                        {selectedResolverOutcome
                          ? resolverOutcomeMeta(selectedResolverOutcome)
                          : 'Resolved, ambiguous, and missing targets render here when present.'}
                      </div>
                      {selectedResolverOutcome ? (
                        <div className="manifold-readout-card__detail">
                          {selectedResolverOutcome.path ? (
                            <div className="manifold-readout-card__path">
                              {selectedResolverOutcome.path}
                            </div>
                          ) : null}
                          <div>{resolverOutcomeNarrative(selectedResolverOutcome)}</div>
                          {!selectedResolverOutcome.path &&
                          selectedResolverOutcome.candidates.length ? (
                            <div>
                              Candidates: {selectedResolverOutcome.candidates.join(', ')}
                            </div>
                          ) : null}
                        </div>
                      ) : null}
                    </div>
                  </div>
                  <div className="manifold-frame-ruler">
                    {(currentTurn?.frames || []).map((frame, index) => (
                      <button
                        className={`manifold-frame-ruler__tick${
                          index === effectiveFrameIndex ? ' is-active' : ''
                        }`}
                        key={frame.record_id}
                        onClick={() => {
                          setTailMode(false);
                          setFrameIndex(index);
                        }}
                        type="button"
                      >
                        <strong>F{index + 1}</strong>
                        <span>seq {frame.sequence}</span>
                      </button>
                    ))}
                  </div>
                  <div className="manifold-gate-ledger">
                    {(currentFrame?.gates || []).length ? (
                      currentFrame!.gates.map((gate) => {
                        const isSelected = gate.gate === selectedGate?.gate;
                        return (
                          <button
                            className={`manifold-gate-card is-${steeringGateClass(gate.gate)}${
                              isSelected ? ' is-selected' : ''
                            }`}
                            key={gate.gate}
                            onClick={() => setSelectedSourceRecordId(gate.dominant_record_id || null)}
                            type="button"
                          >
                            <div className="manifold-gate-card__eyebrow">
                              <span>{manifoldGateLabel(gate)}</span>
                              <span>{steeringPhaseLabel(gate.phase)}</span>
                            </div>
                            <div className="manifold-gate-card__value">{gate.magnitude_percent}%</div>
                            <div className="manifold-gate-card__meta">
                              {signalKindLabel(gate.dominant_signal_kind)} · {gate.level}
                            </div>
                          </button>
                        );
                      })
                    ) : (
                      <div className="manifold-panel-copy">
                        No steering gates were active in the selected frame.
                      </div>
                    )}
                  </div>
                </div>
              </div>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}

function TransitRoute() {
  const { projection } = useRuntimeStore();
  const graph = projection?.trace_graph || null;
  const boardRef = useRef<HTMLDivElement | null>(null);
  const canvasRef = useRef<HTMLDivElement | null>(null);
  const [boardWidth, setBoardWidth] = useState(960);
  const [zoom, setZoom] = useState(1);
  const [scope, setScope] = useState<'significant' | 'full'>('significant');
  const [familyVisibility, setFamilyVisibility] = useState<Record<string, boolean>>({
    model_io: false,
    lineage: false,
    signals: false,
    threads: false,
    tool_results: false,
  });
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [pathD, setPathD] = useState('');
  const panRef = useRef<{
    dragging: boolean;
    startX: number;
    startY: number;
    originX: number;
    originY: number;
  } | null>(null);

  useEffect(() => {
    const board = boardRef.current;
    if (!board || typeof ResizeObserver === 'undefined') {
      return;
    }
    const observer = new ResizeObserver(() => {
      setBoardWidth(board.clientWidth || 960);
    });
    observer.observe(board);
    setBoardWidth(board.clientWidth || 960);
    return () => observer.disconnect();
  }, []);

  const sortedNodes = useMemo(
    () => [...(graph?.nodes || [])].sort((left, right) => left.sequence - right.sequence),
    [graph?.nodes]
  );

  const visibleNodes = useMemo(() => {
    let nodes = sortedNodes.filter((node) => traceNodeVisible(node, scope, familyVisibility));
    if (!nodes.length && scope === 'significant') {
      nodes = sortedNodes.slice();
    }
    return nodes;
  }, [familyVisibility, scope, sortedNodes]);

  const detailLevel = traceDetailLevelForZoom(zoom);
  const layout = traceLayoutForZoom(zoom);
  const effectiveScope =
    scope === 'significant' && !sortedNodes.filter((node) => traceNodeVisible(node, scope, familyVisibility)).length
      ? 'full'
      : scope;
  const columns = Math.max(
    2,
    Math.floor((Math.max(boardWidth, 280) + layout.columnGap) / (layout.tileMin + layout.columnGap))
  );
  const rows: ConversationTraceGraphNode[][] = [];
  for (let index = 0; index < visibleNodes.length; index += columns) {
    rows.push(visibleNodes.slice(index, index + columns));
  }
  const branchLabels = Object.fromEntries((graph?.branches || []).map((branch) => [branch.id, branch.label]));

  useEffect(() => {
    const board = boardRef.current;
    const canvas = canvasRef.current;
    if (!board || !canvas) {
      return;
    }
    const nodes = Array.from(canvas.querySelectorAll<HTMLElement>('.trace-node'));
    if (nodes.length < 2) {
      setPathD('');
      return;
    }
    const canvasRect = canvas.getBoundingClientRect();
    const points = nodes.map((node) => {
      const rect = node.getBoundingClientRect();
      return {
        x: rect.left - canvasRect.left + rect.width / 2,
        y: rect.top - canvasRect.top + rect.height / 2,
      };
    });
    let nextPath = `M ${points[0].x} ${points[0].y}`;
    for (let index = 1; index < points.length; index += 1) {
      const prev = points[index - 1];
      const next = points[index];
      const midX = (prev.x + next.x) / 2;
      nextPath += ` C ${midX} ${prev.y}, ${midX} ${next.y}, ${next.x} ${next.y}`;
    }
    setPathD(nextPath);
  }, [detailLevel, rows, zoom]);

  useEffect(() => {
    function handleMouseMove(event: MouseEvent) {
      if (!panRef.current?.dragging) {
        return;
      }
      setPan({
        x: panRef.current.originX + (event.clientX - panRef.current.startX),
        y: panRef.current.originY + (event.clientY - panRef.current.startY),
      });
    }
    function handleMouseUp() {
      panRef.current = null;
    }
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    window.addEventListener('blur', handleMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
      window.removeEventListener('blur', handleMouseUp);
    };
  }, []);

  function onWheel(event: React.WheelEvent<HTMLDivElement>) {
    if (!visibleNodes.length) {
      return;
    }
    event.preventDefault();
    const nextZoom = Math.max(
      TRACE_ZOOM_MIN,
      Math.min(TRACE_ZOOM_MAX, zoom * Math.exp(-event.deltaY * 0.0015))
    );
    setZoom(nextZoom);
  }

  return (
    <div className="trace-view trace-view--active trace-view--transit" id="transit-view">
      <div className="trace-transit-toolbar" id="trace-transit-toolbar">
        <div className="trace-transit-toggle-row">
          <div className="trace-transit-toggle-group">
            <button
              className={`trace-transit-toggle${scope === 'significant' ? ' is-active' : ''}`}
              data-trace-scope="significant"
              onClick={() => setScope('significant')}
              type="button"
            >
              Significant
            </button>
            <button
              className={`trace-transit-toggle${effectiveScope === 'full' ? ' is-active' : ''}`}
              data-trace-scope="full"
              onClick={() => setScope('full')}
              type="button"
            >
              Full Trace
            </button>
          </div>
          <div className="trace-transit-toggle-group">
            {[
              ['model_io', 'Model I/O'],
              ['lineage', 'Lineage'],
              ['signals', 'Signals'],
              ['threads', 'Threads'],
              ['tool_results', 'Tool Done'],
            ].map(([family, label]) => (
              <button
                className={`trace-transit-toggle${familyVisibility[family] ? ' is-active' : ''}`}
                data-trace-family={family}
                key={family}
                onClick={() =>
                  setFamilyVisibility((current) => ({
                    ...current,
                    [family]: !current[family],
                  }))
                }
                type="button"
              >
                {label}
              </button>
            ))}
          </div>
        </div>
        <div className="trace-transit-meta" id="trace-transit-meta">
          {visibleNodes.length
            ? `Showing ${visibleNodes.length} of ${sortedNodes.length} steps · ${
                effectiveScope === 'full' ? 'full trace' : 'significant steps'
              } · ${TRACE_DETAIL_LEVEL_LABELS[detailLevel] || detailLevel}`
            : `Showing ${effectiveScope === 'full' ? 'full trace' : 'significant steps'} · ${
                TRACE_DETAIL_LEVEL_LABELS[detailLevel] || detailLevel
              }`}
        </div>
      </div>
      <div
        className="trace-board"
        data-detail-level={detailLevel}
        id="trace-board"
        onMouseDown={(event) => {
          if (event.button !== 0) {
            return;
          }
          panRef.current = {
            dragging: true,
            startX: event.clientX,
            startY: event.clientY,
            originX: pan.x,
            originY: pan.y,
          };
        }}
        onWheel={onWheel}
        ref={boardRef}
        style={
          {
            ['--trace-scale' as string]: zoom.toFixed(3),
            ['--trace-column-gap' as string]: `${layout.columnGap.toFixed(2)}px`,
            ['--trace-row-gap' as string]: `${layout.rowGap.toFixed(2)}px`,
            ['--trace-pan-x' as string]: `${pan.x.toFixed(2)}px`,
            ['--trace-pan-y' as string]: `${pan.y.toFixed(2)}px`,
          } as React.CSSProperties
        }
      >
        {visibleNodes.length ? (
          <div className="trace-canvas" ref={canvasRef}>
            <svg aria-hidden="true" className="trace-overlay" id="trace-overlay">
              <path className="trace-overlay__glow" d={pathD} id="trace-overlay-glow" />
              <path className="trace-overlay__trench" d={pathD} id="trace-overlay-trench" />
              <path className="trace-overlay__line" d={pathD} id="trace-overlay-line" />
            </svg>
            {rows.map((row, rowIndex) => (
              <div className={`trace-row${rowIndex % 2 === 1 ? ' reverse' : ''}`} key={`row-${rowIndex}`}>
                {row.map((node, nodeIndex) => {
                  const recencyIndex =
                    visibleNodes.length - (rowIndex * columns + nodeIndex) - 1;
                  const recencyRatio =
                    visibleNodes.length <= 1
                      ? 1
                      : 1 - recencyIndex / Math.max(visibleNodes.length - 1, 1);
                  const depth = Math.max(0, Math.min(1, recencyRatio));
                  const sunTrailDepth = Math.max(0, 1 - recencyIndex / 3);
                  const direction = traceNodeDirection(
                    rowIndex,
                    nodeIndex,
                    row.length,
                    rows.length
                  );
                  const branchLabel = node.branch_id ? branchLabels[node.branch_id] : null;
                  const summary =
                    detailLevel === 'overview'
                      ? node.id
                      : branchLabel || formatTraceKind(node.kind);

                  return (
                    <div
                      className={`trace-node${
                        recencyIndex === 0 ? ' trace-node--latest' : ''
                      } trace-node--${direction}`}
                      key={node.id}
                      style={
                        {
                          ['--node-color' as string]: KIND_COLORS[node.kind] || '#8b949e',
                          ['--node-raise' as string]: `${(-(2 + depth * 6)).toFixed(2)}px`,
                          ['--node-shadow-x' as string]: `${(5 + depth * 2.5).toFixed(2)}px`,
                          ['--node-shadow-y' as string]: `${(6 + depth * 4).toFixed(2)}px`,
                          ['--node-shadow-blur' as string]: `${(12 + depth * 7).toFixed(2)}px`,
                          ['--node-shadow-alpha' as string]: `${(0.1 + depth * 0.12).toFixed(3)}`,
                          ['--node-shadow-warm-alpha' as string]: `${(
                            0.02 +
                            sunTrailDepth * 0.1
                          ).toFixed(3)}`,
                          ['--node-shadow-warm-x' as string]: `${(
                            1 +
                            sunTrailDepth * 2
                          ).toFixed(2)}px`,
                          ['--node-shadow-warm-y' as string]: `${(
                            2 +
                            sunTrailDepth * 4
                          ).toFixed(2)}px`,
                          ['--node-shadow-warm-blur' as string]: `${(
                            4 +
                            sunTrailDepth * 10
                          ).toFixed(2)}px`,
                          ['--node-tilt-x' as string]: `${direction === 'down' ? 8 : 5}deg`,
                          ['--node-tilt-y' as string]: `${
                            direction === 'ltr' ? -4 : direction === 'rtl' ? 4 : 0
                          }deg`,
                          ['--node-specular-alpha' as string]: `${(0.68 + depth * 0.2).toFixed(3)}`,
                        } as React.CSSProperties
                      }
                    >
                      <div className="trace-node__hex">
                        <div className="trace-node__sequence">step {node.sequence}</div>
                        <div className="trace-node__kind">{formatTraceKind(node.kind)}</div>
                        <div className="trace-node__label">{node.label}</div>
                        <div className="trace-node__summary">{summary}</div>
                        <div className="trace-node__branch">{branchLabel || 'mainline'}</div>
                      </div>
                      <div className="trace-node__detail">
                        <div className="trace-node__detail-title">{node.label}</div>
                        <div className="trace-node__detail-meta">
                          {`kind: ${formatTraceKind(node.kind)}\n${
                            branchLabel ? `branch: ${branchLabel}\n` : ''
                          }record: ${node.id}`}
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            ))}
          </div>
        ) : null}
      </div>
      <div className="trace-empty" id="trace-empty" style={{ display: visibleNodes.length ? 'none' : 'block' }}>
        {sortedNodes.length
          ? 'Current transit toggles hide every step. Re-enable a family or switch to full trace.'
          : 'Submit a prompt to see the trace railroad.'}
      </div>
    </div>
  );
}

const rootRoute = createRootRoute({
  component: RuntimeShellLayout,
});

const inspectorRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: InspectorRoute,
});

const transitRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/transit',
  component: TransitRoute,
});

const manifoldRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/manifold',
  component: ManifoldRoute,
});

const routeTree = rootRoute.addChildren([inspectorRoute, transitRoute, manifoldRoute]);

export function buildRuntimeRouter() {
  return createRouter({
    routeTree,
  });
}

declare module '@tanstack/react-router' {
  interface Register {
    router: ReturnType<typeof buildRuntimeRouter>;
  }
}

export function RuntimeApp() {
  const [router] = useState(() => buildRuntimeRouter());

  return (
    <RuntimeStoreProvider>
      <div className="runtime-shell-host">
        <RouterProvider router={router} />
      </div>
    </RuntimeStoreProvider>
  );
}
