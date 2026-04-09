import {
  machineMomentEntry,
  machineMomentLabel,
  type MachineMomentKind,
} from '../trace-machine/machine-model';
import type { MachineMomentProjection, MachineTurnProjection } from '../trace-machine/machine-projection';

const ATLAS_LANES = [
  { id: 'entry', label: 'Entry', color: '#6b7b87' },
  { id: 'planning', label: 'Planning', color: '#d27820' },
  { id: 'execution', label: 'Execution', color: '#2d90c8' },
  { id: 'steering', label: 'Steering', color: '#c55f2a' },
  { id: 'output', label: 'Output', color: '#5f6dcc' },
] as const;

type AtlasLaneId = (typeof ATLAS_LANES)[number]['id'];

interface InspectorAtlasProps {
  currentMoment: MachineMomentProjection | null;
  currentTurn: MachineTurnProjection | null;
  turns: MachineTurnProjection[];
  onSelectMoment: (momentId: string) => void;
}

function atlasLaneForMoment(kind: MachineMomentKind): AtlasLaneId {
  switch (kind) {
    case 'input':
      return 'entry';
    case 'planner':
    case 'diverter':
    case 'spring_return':
      return 'planning';
    case 'evidence_probe':
    case 'tool_run':
      return 'execution';
    case 'force':
    case 'jam':
      return 'steering';
    case 'output':
      return 'output';
    default:
      return 'execution';
  }
}

function atlasColorForMoment(kind: MachineMomentKind) {
  switch (kind) {
    case 'input':
      return '#6b7b87';
    case 'planner':
      return '#cf8f2c';
    case 'evidence_probe':
      return '#2d90c8';
    case 'diverter':
      return '#d27820';
    case 'jam':
      return '#c55f2a';
    case 'spring_return':
      return '#4f9b5d';
    case 'tool_run':
      return '#2787a0';
    case 'force':
      return '#9b6ac9';
    case 'output':
      return '#5f6dcc';
    default:
      return '#8393a0';
  }
}

function selectedMomentMeta(moment: MachineMomentProjection | null) {
  if (!moment) {
    return 'No machine moment selected';
  }
  return `${machineMomentLabel(moment.kind)} · sequence ${moment.sequence}`;
}

export function InspectorAtlas({
  currentMoment,
  currentTurn,
  turns,
  onSelectMoment,
}: InspectorAtlasProps) {
  const moments = currentTurn?.moments || [];
  const selectedMoment = currentMoment || moments[moments.length - 1] || null;
  const selectedMomentId = selectedMoment?.momentId || null;
  const points = moments.map((moment, index) => {
    const laneId = atlasLaneForMoment(moment.kind);
    const laneIndex = ATLAS_LANES.findIndex((lane) => lane.id === laneId);
    return {
      color: atlasColorForMoment(moment.kind),
      lane: ATLAS_LANES[laneIndex],
      moment,
      x: moments.length <= 1 ? 50 : 8 + (index / Math.max(moments.length - 1, 1)) * 84,
      y: 16 + laneIndex * 18,
    };
  });

  const stagePath = points
    .map((point, index) => `${index === 0 ? 'M' : 'L'} ${point.x} ${point.y}`)
    .join(' ');

  return (
    <section className="forensic-atlas-card" id="forensic-atlas">
      <div className="forensic-atlas-head">
        <div>
          <div className="forensic-card-title">Forensic Atlas</div>
          <div className="forensic-atlas-subhead">
            {currentTurn
              ? `${currentTurn.turnId} · ${moments.length} machine moments · ${turns.length} turns`
              : 'Forensic replay appears here when transit records exist.'}
          </div>
        </div>
        <div className="forensic-atlas-metrics">
          <span>{currentTurn?.lifecycle || 'idle'}</span>
          <span>{selectedMomentMeta(selectedMoment)}</span>
        </div>
      </div>

      {!moments.length ? (
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

            <svg
              aria-hidden="true"
              className="forensic-atlas-overlay"
              viewBox="0 0 100 100"
              preserveAspectRatio="none"
            >
              <path className="forensic-atlas-overlay__glow" d={stagePath} />
              <path className="forensic-atlas-overlay__line" d={stagePath} />
            </svg>

            {points.map((point) => (
              <button
                className={`forensic-atlas-point${
                  point.moment.momentId === selectedMomentId ? ' is-selected' : ''
                }`}
                data-atlas-moment-id={point.moment.momentId}
                key={point.moment.momentId}
                onClick={() => onSelectMoment(point.moment.momentId)}
                style={
                  {
                    ['--atlas-point-color' as string]: point.color,
                    ['--atlas-point-x' as string]: `${point.x}%`,
                    ['--atlas-point-y' as string]: `${point.y}%`,
                  } as React.CSSProperties
                }
                title={point.moment.headline}
                type="button"
              >
                <span className="sr-only">{point.moment.headline}</span>
              </button>
            ))}

            {selectedMoment ? (
              <div className="forensic-atlas-popup" id="forensic-atlas-popup">
                <div className="forensic-atlas-popup__title">{selectedMoment.headline}</div>
                <div className="forensic-atlas-popup__meta">{selectedMomentMeta(selectedMoment)}</div>
                <div className="forensic-atlas-popup__record-id">
                  {selectedMoment.raw.primaryForensicRecordId || 'no linked record'}
                </div>
                <div className="forensic-atlas-popup__body">{selectedMoment.narrative}</div>
                <div className="forensic-contribs">
                  {selectedMoment.raw.forensicRecordIds.map((recordId) => (
                    <span className="forensic-chip" key={recordId}>
                      {recordId}
                    </span>
                  ))}
                </div>
              </div>
            ) : null}
          </div>

          <div className="forensic-atlas-scrubber" id="forensic-atlas-scrubber">
            {moments.map((moment) => (
              <button
                className={`forensic-atlas-scrubber__chip${
                  moment.momentId === selectedMomentId ? ' is-active' : ''
                }`}
                data-atlas-scrub-moment-id={moment.momentId}
                key={`${moment.momentId}-scrubber`}
                onClick={() => onSelectMoment(moment.momentId)}
                type="button"
              >
                <span>{moment.sequence}</span>
                <strong>{machineMomentLabel(moment.kind)}</strong>
              </button>
            ))}
          </div>
        </>
      )}
    </section>
  );
}
