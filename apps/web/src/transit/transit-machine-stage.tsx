import { machineMomentEntry, machineMomentLabel } from '../trace-machine/machine-model';
import type {
  MachineMomentProjection,
  MachineTurnProjection,
} from '../trace-machine/machine-projection';

interface TransitMachineStageProps {
  currentMoment: MachineMomentProjection | null;
  currentTurn: MachineTurnProjection | null;
  taskId: string | null;
  turns: MachineTurnProjection[];
  onSelectMoment: (momentId: string) => void;
}

function stageLeft(index: number, count: number) {
  if (count <= 1) {
    return 50;
  }
  return 10 + (index / Math.max(count - 1, 1)) * 80;
}

export function TransitMachineStage({
  currentMoment,
  currentTurn,
  taskId,
  turns,
  onSelectMoment,
}: TransitMachineStageProps) {
  const moments = currentTurn?.moments || [];
  const activeMomentId = currentMoment?.momentId || null;
  const activeEntry = currentMoment ? machineMomentEntry(currentMoment.kind) : null;

  return (
    <section className="transit-machine" id="transit-machine">
      <div className="transit-machine__head">
        <div>
          <div className="transit-machine__eyebrow">Narrative transit</div>
          <div className="transit-machine__title">Turn Machine</div>
          <div className="transit-machine__meta" id="transit-machine-meta">
            {currentTurn
              ? `${currentTurn.turnId} · ${moments.length} machine parts · ${turns.length} turns`
              : `${taskId || 'task'} · awaiting machine moments`}
          </div>
        </div>
      </div>

      {!moments.length ? (
        <div className="trace-empty" id="trace-empty">
          Submit a prompt to see the turn machine.
        </div>
      ) : (
        <>
          <div className="transit-machine__stage" id="transit-machine-stage">
            <div className="transit-machine__rail" aria-hidden="true" />
            {moments.map((moment, index) => {
              const entry = machineMomentEntry(moment.kind);
              return (
                <button
                  className={`transit-machine__part is-${moment.kind}${
                    moment.momentId === activeMomentId ? ' is-selected' : ''
                  }`}
                  data-transit-moment-id={moment.momentId}
                  key={moment.momentId}
                  onClick={() => onSelectMoment(moment.momentId)}
                  style={{ left: `${stageLeft(index, moments.length)}%` } as React.CSSProperties}
                  type="button"
                >
                  <span className="transit-machine__part-sequence">
                    {String(moment.sequence).padStart(2, '0')}
                  </span>
                  <strong className="transit-machine__part-label">
                    {machineMomentLabel(moment.kind)}
                  </strong>
                  <span className="transit-machine__part-headline">{moment.headline}</span>
                  <span className="sr-only">{entry.narrative}</span>
                </button>
              );
            })}
          </div>

          <div className="transit-machine__detail" id="transit-machine-detail">
            <div className="transit-machine__detail-chip-row">
              <span className="transit-machine__chip">
                {currentMoment ? machineMomentLabel(currentMoment.kind) : 'No part'}
              </span>
              <span className="transit-machine__chip">
                {currentTurn ? currentTurn.turnId : taskId || 'task'}
              </span>
              {currentMoment ? (
                <span className="transit-machine__chip">moment {currentMoment.sequence}</span>
              ) : null}
            </div>
            <div className="transit-machine__detail-title">
              {currentMoment?.headline || 'Awaiting a selected machine part'}
            </div>
            <div className="transit-machine__detail-body">
              {activeEntry?.narrative ||
                'Select a machine part from the stage to inspect the causal story.'}
            </div>
            {currentMoment ? (
              <div className="transit-machine__detail-note">{currentMoment.narrative}</div>
            ) : null}
          </div>

          <div className="transit-machine__scrubber" id="transit-machine-scrubber">
            {moments.map((moment) => (
              <button
                className={`transit-machine__scrubber-chip${
                  moment.momentId === activeMomentId ? ' is-active' : ''
                }`}
                data-transit-scrub-moment-id={moment.momentId}
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
