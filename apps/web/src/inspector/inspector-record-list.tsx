import {
  kindEntry,
  kindLabel,
  lifecycleLabel,
  primaryArtifact,
  recordMeta,
  recordSummary,
} from '../runtime-helpers';
import type { ForensicRecordProjection, ForensicTurnProjection } from '../runtime-types';

interface InspectorRecordListProps {
  currentRecord: ForensicRecordProjection | null;
  currentTurn: ForensicTurnProjection | null;
  records: ForensicRecordProjection[];
  onSelectRecord: (recordId: string) => void;
}

export function InspectorRecordList({
  currentRecord,
  currentTurn,
  records,
  onSelectRecord,
}: InspectorRecordListProps) {
  return (
    <section className="forensic-records" id="forensic-records">
      {!currentTurn ? (
        <div className="forensic-empty-state">Select a turn to inspect its transit lineage.</div>
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
                  onClick={() => onSelectRecord(recordProjection.record.record_id)}
                  type="button"
                >
                  <div className="forensic-record-head">
                    <div className="forensic-record-title">{recordSummary(recordProjection)}</div>
                    <span className={`forensic-lifecycle is-${recordProjection.lifecycle}`}>
                      {lifecycleLabel(recordProjection.lifecycle)}
                    </span>
                  </div>
                  <div className="forensic-record-subtitle">{recordMeta(recordProjection)}</div>
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
            <div className="forensic-empty-state">No records match the current lineage focus.</div>
          )}
        </>
      )}
    </section>
  );
}
