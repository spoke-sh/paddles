import { useEffect, useRef, useState } from 'react';

import {
  KIND_COLORS,
  kindEntry,
  recordMeta,
  recordSummary,
  sourceColor,
} from '../runtime-helpers';
import type { ForensicRecordProjection, ForensicTurnProjection } from '../runtime-types';

const ATLAS_LANES = [
  { id: 'prompt', label: 'Prompt', color: '#5f6d7a' },
  { id: 'planning', label: 'Planning', color: '#d27820' },
  { id: 'model', label: 'Model I/O', color: KIND_COLORS.action },
  { id: 'signal', label: 'Signals', color: KIND_COLORS.signal },
  { id: 'artifact', label: 'Artifacts', color: KIND_COLORS.forensic },
] as const;

type AtlasLaneId = (typeof ATLAS_LANES)[number]['id'];

interface InspectorAtlasProps {
  comparisonRecord: ForensicRecordProjection | null;
  currentTurn: ForensicTurnProjection | null;
  turns: ForensicTurnProjection[];
  onSelectRecord: (recordId: string) => void;
}

function atlasLaneForRecord(recordProjection: ForensicRecordProjection): AtlasLaneId {
  const key = kindEntry(recordProjection).key;
  if (key === 'TaskRootStarted' || key === 'TurnStarted') {
    return 'prompt';
  }
  if (key === 'PlannerAction') {
    return 'planning';
  }
  if (key === 'ModelExchangeArtifact') {
    return 'model';
  }
  if (key === 'SignalSnapshot') {
    return 'signal';
  }
  return 'artifact';
}

function atlasColorForRecord(recordProjection: ForensicRecordProjection) {
  const key = kindEntry(recordProjection).key;
  if (key === 'PlannerAction') {
    return '#d27820';
  }
  if (key === 'TaskRootStarted' || key === 'TurnStarted') {
    return '#6b7b87';
  }
  return KIND_COLORS[kindEntry(recordProjection).key === 'SignalSnapshot' ? 'signal' : 'forensic'];
}

function selectedContributionChips(recordProjection: ForensicRecordProjection | null) {
  if (!recordProjection || kindEntry(recordProjection).key !== 'SignalSnapshot') {
    return [];
  }
  const value = kindEntry(recordProjection).value;
  return ((value.contributions as Array<{ source: string; share_percent: number; rationale?: string }>) || [])
    .slice(0, 4)
    .map((contribution) => ({
      label: contribution.source,
      percent: contribution.share_percent,
      rationale: contribution.rationale,
    }));
}

export function InspectorAtlas({
  comparisonRecord,
  currentTurn,
  turns,
  onSelectRecord,
}: InspectorAtlasProps) {
  const records = currentTurn?.records || [];
  const [atlasSelectedRecordId, setAtlasSelectedRecordId] = useState<string | null>(
    comparisonRecord?.record.record_id || records[records.length - 1]?.record.record_id || null
  );
  const previousComparisonRecordId = useRef<string | null>(
    comparisonRecord?.record.record_id || null
  );
  const selectedRecord =
    records.find((recordProjection) => recordProjection.record.record_id === atlasSelectedRecordId) ||
    comparisonRecord ||
    records[records.length - 1] ||
    null;
  const selectedRecordId = selectedRecord?.record.record_id || null;
  const contributionChips = selectedContributionChips(selectedRecord);
  const points = records.map((recordProjection, index) => {
    const laneId = atlasLaneForRecord(recordProjection);
    const laneIndex = ATLAS_LANES.findIndex((lane) => lane.id === laneId);
    return {
      color: atlasColorForRecord(recordProjection),
      lane: ATLAS_LANES[laneIndex],
      record: recordProjection,
      x:
        records.length <= 1
          ? 50
          : 8 + (index / Math.max(records.length - 1, 1)) * 84,
      y: 16 + laneIndex * 18,
    };
  });

  const stagePath = points
    .map((point, index) => `${index === 0 ? 'M' : 'L'} ${point.x} ${point.y}`)
    .join(' ');

  useEffect(() => {
    if (!records.length) {
      setAtlasSelectedRecordId(null);
      return;
    }
    if (!atlasSelectedRecordId || !records.some((record) => record.record.record_id === atlasSelectedRecordId)) {
      setAtlasSelectedRecordId(
        comparisonRecord?.record.record_id || records[records.length - 1].record.record_id
      );
    }
  }, [atlasSelectedRecordId, comparisonRecord?.record.record_id, records]);

  useEffect(() => {
    const nextComparisonId = comparisonRecord?.record.record_id || null;
    if (
      nextComparisonId &&
      previousComparisonRecordId.current &&
      nextComparisonId !== previousComparisonRecordId.current
    ) {
      setAtlasSelectedRecordId(nextComparisonId);
    }
    previousComparisonRecordId.current = nextComparisonId;
  }, [comparisonRecord?.record.record_id]);

  return (
    <section className="forensic-atlas-card" id="forensic-atlas">
      <div className="forensic-atlas-head">
        <div>
          <div className="forensic-card-title">Forensic Atlas</div>
          <div className="forensic-atlas-subhead">
            {currentTurn
              ? `${currentTurn.turn_id} · ${records.length} records · ${turns.length} turns`
              : 'Forensic replay appears here when transit records exist.'}
          </div>
        </div>
        <div className="forensic-atlas-metrics">
          <span>{currentTurn?.lifecycle || 'idle'}</span>
          <span>{selectedRecord ? recordSummary(selectedRecord) : 'no record selected'}</span>
        </div>
      </div>

      {!records.length ? (
        <div className="forensic-empty">Forensic replay appears here when transit records exist.</div>
      ) : (
        <>
          <div className="forensic-atlas-stage">
            {ATLAS_LANES.map((lane, laneIndex) => (
              <div
                className="forensic-atlas-lane"
                key={lane.id}
                style={
                  {
                    ['--atlas-lane-color' as string]: lane.color,
                    ['--atlas-lane-index' as string]: String(laneIndex),
                  } as React.CSSProperties
                }
              >
                <div className="forensic-atlas-lane__label">{lane.label}</div>
              </div>
            ))}

            <svg aria-hidden="true" className="forensic-atlas-overlay" viewBox="0 0 100 100" preserveAspectRatio="none">
              <path className="forensic-atlas-overlay__glow" d={stagePath} />
              <path className="forensic-atlas-overlay__line" d={stagePath} />
            </svg>

            {points.map((point) => (
              <button
                className={`forensic-atlas-point${
                  point.record.record.record_id === selectedRecordId ? ' is-selected' : ''
                }`}
                data-atlas-record-id={point.record.record.record_id}
                key={point.record.record.record_id}
                onClick={() => {
                  setAtlasSelectedRecordId(point.record.record.record_id);
                  onSelectRecord(point.record.record.record_id);
                }}
                style={
                  {
                    ['--atlas-point-color' as string]: point.color,
                    ['--atlas-point-x' as string]: `${point.x}%`,
                    ['--atlas-point-y' as string]: `${point.y}%`,
                  } as React.CSSProperties
                }
                title={recordSummary(point.record)}
                type="button"
              >
                <span className="sr-only">{recordSummary(point.record)}</span>
              </button>
            ))}

            {selectedRecord ? (
              <div className="forensic-atlas-popup" id="forensic-atlas-popup">
                <div className="forensic-atlas-popup__title">{recordSummary(selectedRecord)}</div>
                <div className="forensic-atlas-popup__meta">{recordMeta(selectedRecord)}</div>
                <div className="forensic-atlas-popup__record-id">{selectedRecord.record.record_id}</div>
                <div className="forensic-atlas-popup__body">
                  {kindEntry(selectedRecord).key === 'SignalSnapshot'
                    ? String(kindEntry(selectedRecord).value.summary || recordSummary(selectedRecord))
                    : String(recordSummary(selectedRecord))}
                </div>
                {contributionChips.length ? (
                  <div className="forensic-contribs">
                    {contributionChips.map((chip) => (
                      <span
                        className="forensic-chip"
                        key={`${chip.label}-${chip.percent}`}
                        title={chip.rationale}
                      >
                        <span
                          className="forensic-chip-swatch"
                          style={{ ['--chip-color' as string]: sourceColor(chip.label) }}
                        />
                        {chip.label} {chip.percent}%
                      </span>
                    ))}
                  </div>
                ) : null}
              </div>
            ) : null}
          </div>

          <div className="forensic-atlas-scrubber" id="forensic-atlas-scrubber">
            {records.map((recordProjection) => (
              <button
                className={`forensic-atlas-scrubber__chip${
                  recordProjection.record.record_id === selectedRecordId ? ' is-active' : ''
                }`}
                data-atlas-scrub-record-id={recordProjection.record.record_id}
                key={recordProjection.record.record_id}
                onClick={() => {
                  setAtlasSelectedRecordId(recordProjection.record.record_id);
                  onSelectRecord(recordProjection.record.record_id);
                }}
                type="button"
              >
                <span>{recordProjection.record.sequence}</span>
                <strong>{recordSummary(recordProjection)}</strong>
              </button>
            ))}
          </div>
        </>
      )}
    </section>
  );
}
