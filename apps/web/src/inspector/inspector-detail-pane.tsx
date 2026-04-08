import {
  kindEntry,
  kindLabel,
  lifecycleLabel,
  primaryArtifact,
  rawRecordBody,
  recordMeta,
  recordSummary,
  renderedRecordBody,
} from '../runtime-helpers';
import type { ForensicRecordProjection, ForensicTurnProjection } from '../runtime-types';
import type {
  FocusState,
  InspectorDetailMode,
  InspectorSelectionMode,
} from './use-inspector-selection';

interface InspectorDetailPaneProps {
  baseline: ForensicRecordProjection | null;
  currentRecord: ForensicRecordProjection | null;
  currentTurn: ForensicTurnProjection | null;
  detailMode: InspectorDetailMode;
  focus: FocusState;
  modelCalls: Array<unknown>;
  plannerSteps: Array<unknown>;
  selectionMode: InspectorSelectionMode;
  taskId: string | null;
  turns: ForensicTurnProjection[];
  onSelectDetailMode: (mode: InspectorDetailMode) => void;
}

export function InspectorDetailPane({
  baseline,
  currentRecord,
  currentTurn,
  detailMode,
  focus,
  modelCalls,
  plannerSteps,
  selectionMode,
  taskId,
  turns,
  onSelectDetailMode,
}: InspectorDetailPaneProps) {
  return (
    <section className="forensic-detail-pane">
      <div className="forensic-detail-toolbar">
        <div>
          <div className="forensic-detail-title" id="forensic-detail-title">
            {!turns.length
              ? 'No selection'
              : selectionMode === 'conversation'
                ? taskId
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
            onClick={() => onSelectDetailMode('rendered')}
            type="button"
          >
            Rendered
          </button>
          <button
            className={`forensic-toggle${detailMode === 'raw' ? ' is-active' : ''}`}
            data-detail-mode="raw"
            onClick={() => onSelectDetailMode('raw')}
            type="button"
          >
            Raw
          </button>
        </div>
      </div>
      <div className="forensic-detail-body" id="forensic-detail">
        {!turns.length ? (
          <div className="forensic-empty-state">
            Submit a prompt or wait for a trace-producing turn to inspect raw and rendered model
            context.
          </div>
        ) : selectionMode === 'conversation' ? (
          <>
            <div className="forensic-detail-card">
              <div className="forensic-detail-card-title">Conversation Summary</div>
              <dl className="forensic-detail-grid">
                <div>
                  <dt>Task</dt>
                  <dd>{taskId}</dd>
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
                Choose a turn on the left, then narrow the lineage focus to model calls or planner
                steps before drilling into an exact trace record.
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
                Current focus: {focus.kind === 'all' ? 'all records' : `${focus.kind} ${focus.id || ''}`}
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
              <div className="forensic-detail-card-title">Payload ({detailMode})</div>
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
  );
}
