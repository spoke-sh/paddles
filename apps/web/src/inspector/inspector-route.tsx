import {
  kindLabel,
  lifecycleLabel,
  rawRecordBody,
  recordMeta,
  renderedRecordBody,
} from '../runtime-helpers';
import { useRuntimeStore } from '../runtime-store';
import { machineMomentLabel } from '../trace-machine/machine-model';
import { InspectorOverview } from './inspector-overview';
import { linkedRecordHeadline, useInspectorSelection } from './use-inspector-selection';

export function InspectorRoute() {
  const { projection } = useRuntimeStore();
  const {
    baseline,
    contributions,
    currentArtifact,
    currentMachineTurn,
    currentMoment,
    currentPayload,
    currentRecord,
    linkedRecords,
    machineTurns,
    showInternals,
    strongestSignalValue,
    taskId,
    toggleInternals,
    selectMoment,
    selectTurn,
  } = useInspectorSelection(projection);

  return (
    <div className="trace-view trace-view--active forensic-view" id="forensic-view">
      <InspectorOverview
        baseline={baseline}
        comparisonRecord={currentRecord}
        contributions={contributions}
        currentMoment={currentMoment}
        currentTurn={currentMachineTurn}
        strongestSignalValue={strongestSignalValue}
        turns={machineTurns}
        onSelectMoment={selectMoment}
      />

      <section className="forensic-machine-controls" id="forensic-machine-controls">
        <div className="forensic-turn-scrubber" id="forensic-turn-scrubber">
          {machineTurns.map((turn) => (
            <button
              className={`forensic-turn-scrubber__chip${
                turn.turnId === currentMachineTurn?.turnId ? ' is-active' : ''
              }`}
              data-forensic-turn-id={turn.turnId}
              key={turn.turnId}
              onClick={() => selectTurn(turn.turnId)}
              type="button"
            >
              <span>{lifecycleLabel(turn.lifecycle)}</span>
              <strong>{turn.turnId}</strong>
            </button>
          ))}
        </div>
        <button
          className={`forensic-internals-toggle${showInternals ? ' is-active' : ''}`}
          id="forensic-internals-toggle"
          onClick={toggleInternals}
          type="button"
        >
          {showInternals ? 'Hide internals' : 'Show internals'}
        </button>
      </section>

      <section className="forensic-machine-summary" id="forensic-machine-summary">
        <div className="forensic-detail-chip-row">
          <span className="forensic-chip">
            {currentMoment ? machineMomentLabel(currentMoment.kind) : 'No moment'}
          </span>
          <span className="forensic-chip">{currentMachineTurn?.turnId || taskId || 'task'}</span>
          {currentRecord ? <span className="forensic-chip">{currentRecord.record.record_id}</span> : null}
        </div>
        <div className="forensic-machine-summary__title">
          {currentMoment?.headline || 'Awaiting a selected machine moment'}
        </div>
        <div className="forensic-machine-summary__body">
          {currentMoment?.narrative ||
            'Select a machine moment from the atlas to inspect how the turn moved.'}
        </div>
        {currentRecord ? (
          <div className="forensic-machine-summary__note">
            Linked forensic evidence: {linkedRecordHeadline(currentRecord)}
          </div>
        ) : null}
      </section>

      {showInternals ? (
        <section className="forensic-internals-shell" id="forensic-internals-shell">
          <div className="forensic-internals-shell__head">
            <div>
              <div className="forensic-card-title">Internals</div>
              <div className="forensic-machine-summary__note">
                Raw payloads, record ids, and lineage anchors for the selected machine moment.
              </div>
            </div>
          </div>

          <div className="forensic-internals-grid">
            <section className="forensic-internals-detail">
              <div className="forensic-detail-toolbar">
                <div>
                  <div className="forensic-detail-title" id="forensic-detail-title">
                    {linkedRecordHeadline(currentRecord)}
                  </div>
                  <div className="forensic-detail-meta" id="forensic-detail-meta">
                    {currentRecord
                      ? recordMeta(currentRecord)
                      : 'No linked forensic record is available for this moment.'}
                  </div>
                </div>
              </div>
              <div className="forensic-detail-body" id="forensic-detail">
                {!currentRecord || !currentPayload ? (
                  <div className="forensic-empty-state">
                    No raw forensic payload is available for the selected machine moment.
                  </div>
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
                          <dd>{kindLabel(Object.keys(currentRecord.record.kind || {})[0] || 'record')}</dd>
                        </div>
                        <div>
                          <dt>Lifecycle</dt>
                          <dd>{lifecycleLabel(currentRecord.lifecycle)}</dd>
                        </div>
                        <div>
                          <dt>Artifact</dt>
                          <dd>{currentArtifact?.mime_type || 'record/json'}</dd>
                        </div>
                        <div>
                          <dt>Baseline</dt>
                          <dd>{baseline?.record.record_id || 'none'}</dd>
                        </div>
                      </dl>
                    </div>
                    <div className="forensic-detail-card">
                      <div className="forensic-detail-card-title">Rendered payload</div>
                      <pre className="forensic-code">{renderedRecordBody(currentRecord)}</pre>
                    </div>
                    <div className="forensic-detail-card">
                      <div className="forensic-detail-card-title">Raw payload</div>
                      <pre className="forensic-raw">{rawRecordBody(currentRecord)}</pre>
                    </div>
                  </>
                )}
              </div>
            </section>

            <section className="forensic-linked-records" id="forensic-records">
              <div className="forensic-section-head">
                <div className="forensic-section-title">Linked records</div>
                <div className="forensic-section-meta">{linkedRecords.length} records</div>
              </div>
              {linkedRecords.length ? (
                linkedRecords.map((recordProjection) => (
                  <div
                    className={`forensic-record${
                      currentRecord?.record.record_id === recordProjection.record.record_id
                        ? ' is-selected'
                        : ''
                    }${
                      recordProjection.lifecycle === 'superseded' ? ' is-superseded' : ''
                    }`}
                    data-record-id={recordProjection.record.record_id}
                    key={recordProjection.record.record_id}
                  >
                    <div className="forensic-record-head">
                      <div className="forensic-record-title">
                        {linkedRecordHeadline(recordProjection)}
                      </div>
                      <span className={`forensic-lifecycle is-${recordProjection.lifecycle}`}>
                        {lifecycleLabel(recordProjection.lifecycle)}
                      </span>
                    </div>
                    <div className="forensic-record-subtitle">{recordMeta(recordProjection)}</div>
                  </div>
                ))
              ) : (
                <div className="forensic-empty-state">
                  No forensic records were linked to this machine moment.
                </div>
              )}
            </section>
          </div>
        </section>
      ) : null}
    </div>
  );
}
