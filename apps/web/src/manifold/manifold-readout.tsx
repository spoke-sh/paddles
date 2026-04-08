import {
  manifoldGateLabel,
  manifoldSignalLabel,
  resolverOutcomeMeta,
  resolverOutcomeNarrative,
  resolverOutcomeTitle,
  signalKindLabel,
  steeringGateClass,
  steeringGateLabel,
  steeringPhaseLabel,
} from '../runtime-helpers';
import type { ManifoldFrame, ManifoldTurnProjection } from '../runtime-types';

interface ManifoldReadoutProps {
  currentFrame: ManifoldFrame | null;
  currentTurn: ManifoldTurnProjection | null;
  effectiveFrameIndex: number;
  selectedGate: (ManifoldFrame['gates'][number]) | null;
  selectedResolverOutcome: ReturnType<typeof resolverOutcomeTitle> extends string
    ? ReturnType<typeof import('../runtime-helpers').resolverSignalDetails>
    : never;
  selectedSignal: (ManifoldFrame['active_signals'][number]) | null;
  selectedSignalFrame: ManifoldFrame | null;
  onFrameSelect: (frameIndex: number) => void;
  onSourceSelect: (recordId: string | null) => void;
}

export function ManifoldReadout({
  currentFrame,
  currentTurn,
  effectiveFrameIndex,
  selectedGate,
  selectedResolverOutcome,
  selectedSignal,
  selectedSignalFrame,
  onFrameSelect,
  onSourceSelect,
}: ManifoldReadoutProps) {
  return (
    <>
      <div className="manifold-readout">
        <div
          className={`manifold-readout-card is-${
            selectedGate ? steeringGateClass(selectedGate.gate) : 'containment'
          }`}
        >
          <div className="manifold-readout-card__eyebrow">
            <span>Selected gate</span>
            <span>{selectedGate ? steeringPhaseLabel(selectedGate.phase) : 'Idle'}</span>
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
            <span>{selectedSignalFrame ? `Frame ${selectedSignalFrame.sequence}` : 'No source'}</span>
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
            <span>{selectedResolverOutcome ? selectedResolverOutcome.status : 'No resolver signal'}</span>
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
                <div className="manifold-readout-card__path">{selectedResolverOutcome.path}</div>
              ) : null}
              <div>{resolverOutcomeNarrative(selectedResolverOutcome)}</div>
              {!selectedResolverOutcome.path && selectedResolverOutcome.candidates.length ? (
                <div>Candidates: {selectedResolverOutcome.candidates.join(', ')}</div>
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
            onClick={() => onFrameSelect(index)}
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
                onClick={() => onSourceSelect(gate.dominant_record_id || null)}
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
          <div className="manifold-panel-copy">No steering gates were active in the selected frame.</div>
        )}
      </div>
    </>
  );
}
