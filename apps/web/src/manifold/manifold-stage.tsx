import {
  STEERING_GATE_COLORS,
  STEERING_GATE_ORDER,
  manifoldAnchorLabel,
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
import type { ManifoldForceLink, ManifoldForcePoint, ManifoldForceSlice } from './gate-field';
import type { ManifoldCameraState } from './use-manifold-camera';

interface ManifoldStageProps {
  camera: ManifoldCameraState;
  currentFrame: ManifoldFrame | null;
  currentTurn: ManifoldTurnProjection | null;
  dragMode: 'tilt' | 'pan' | 'rotate' | null;
  effectiveFrameIndex: number;
  gateField: { points: ManifoldForcePoint[]; links: ManifoldForceLink[]; slices: ManifoldForceSlice[] };
  playing: boolean;
  selectedGate: (ManifoldFrame['gates'][number]) | null;
  selectedResolverOutcome: ReturnType<typeof resolverOutcomeTitle> extends string
    ? ReturnType<typeof import('../runtime-helpers').resolverSignalDetails>
    : never;
  selectedSignal: (ManifoldFrame['active_signals'][number]) | null;
  selectedSourceRecordId: string | null;
  taskId: string | null;
  totalFrames: number;
  turnsCount: number;
  onBeginCameraDrag: (event: React.MouseEvent<HTMLDivElement>) => void;
  onFrameChange: (frameIndex: number) => void;
  onPointSelect: (frameIndex: number, recordId: string | null) => void;
  onReplay: () => void;
  onResetView: () => void;
  onTogglePlay: () => void;
  onViewportWheel: (event: React.WheelEvent<HTMLDivElement>) => void;
}

export function ManifoldStage({
  camera,
  currentFrame,
  currentTurn,
  dragMode,
  effectiveFrameIndex,
  gateField,
  playing,
  selectedGate,
  selectedResolverOutcome,
  selectedSignal,
  selectedSourceRecordId,
  taskId,
  totalFrames,
  turnsCount,
  onBeginCameraDrag,
  onFrameChange,
  onPointSelect,
  onReplay,
  onResetView,
  onTogglePlay,
  onViewportWheel,
}: ManifoldStageProps) {
  const activeSourceRecordId = selectedSourceRecordId || selectedSignal?.snapshot_record_id || null;
  const selectedSlice =
    activeSourceRecordId == null
      ? null
      : gateField.slices.find(
          (slice) =>
            slice.turnId === currentTurn?.turn_id && slice.frameIndex === effectiveFrameIndex
        ) || null;

  return (
    <section className="manifold-stage">
      <div className="manifold-stage-head">
        <div>
          <div className="manifold-stage-title">Steering Gate Manifold</div>
          <div className="manifold-stage-meta" id="manifold-stage-meta">
            {!turnsCount
              ? 'Awaiting replay-backed manifold frames'
              : `${taskId || 'task'} · ${turnsCount} turns · ${totalFrames} frames · selected ${
                  effectiveFrameIndex + 1
                }`}
          </div>
        </div>
        <div className="manifold-stage-controls">
          <button
            className="trace-tab manifold-stage-button"
            id="manifold-play-toggle"
            onClick={onTogglePlay}
            type="button"
          >
            {playing ? 'Pause' : 'Play'}
          </button>
          <button
            className="trace-tab manifold-stage-button"
            id="manifold-replay-button"
            onClick={onReplay}
            type="button"
          >
            Replay
          </button>
          <button
            className="trace-tab manifold-stage-button"
            id="manifold-reset-view-button"
            onClick={onResetView}
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
          onChange={(event) => onFrameChange(Number(event.target.value))}
          type="range"
          value={effectiveFrameIndex}
        />
        <div className="manifold-stage-frame-meta" id="manifold-frame-meta">
          Frame {currentTurn ? effectiveFrameIndex + 1 : 0} / {currentTurn?.frames.length || 0}
        </div>
      </div>
      <div className="manifold-canvas" id="manifold-canvas">
        {!turnsCount ? (
          <div className="manifold-empty-state">
            <strong>Steering gate manifold route is armed.</strong>
            <p>
              Once replay-backed steering snapshots arrive, the temporal gate field will populate
              here.
            </p>
          </div>
        ) : (
          <div className="manifold-machine">
            <div className="manifold-spacefield">
              <div className="manifold-spacefield__meta">
                <span>Temporal gate field</span>
                <span>
                  {currentTurn?.turn_id || 'no-turn'} · anchor {manifoldAnchorLabel(currentFrame?.anchor)}
                </span>
              </div>
              <div
                className={`manifold-spacefield__viewport${
                  dragMode ? ` is-dragging is-${dragMode}` : ''
                }`}
                data-testid="manifold-spacefield-viewport"
                onDoubleClick={onResetView}
                onMouseDown={onBeginCameraDrag}
                onWheel={onViewportWheel}
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
                  Drag to tilt · Alt+drag to rotate · Shift+drag to pan · Wheel to zoom ·
                  Double-click to reset
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
                  {gateField.slices.map((slice) => {
                    const isSelected =
                      slice.turnId === currentTurn?.turn_id && slice.frameIndex === effectiveFrameIndex;
                    return (
                      <div
                        className={`manifold-force-slice${
                          slice.isSelectedTurn ? '' : ' is-contextual'
                        }${isSelected ? ' is-selected' : ''}`}
                        data-point-count={slice.pointCount.toString()}
                        key={slice.key}
                        style={{
                          left: `${slice.leftPercent}%`,
                          top: `${slice.topPercent}%`,
                          height: `${slice.heightPercent}%`,
                        } as React.CSSProperties}
                      >
                        <div className="manifold-force-slice__spine" />
                        {slice.keys.map((key) => (
                          <div
                            className={`manifold-force-slice__key is-${steeringGateClass(key.gate)}`}
                            key={key.key}
                            style={{
                              top: `${key.relativeTopPercent}%`,
                              ['--slice-key-intensity' as string]: `${Math.max(
                                0.28,
                                key.magnitudePercent / 100
                              )}`,
                            } as React.CSSProperties}
                          />
                        ))}
                      </div>
                    );
                  })}
                  {gateField.links.map((link) => (
                    <div
                      className={`manifold-force-link is-${steeringGateClass(link.gate)}${
                        link.isSelectedTurn ? '' : ' is-contextual'
                      }`}
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
                    const isCurrent =
                      point.turnId === currentTurn?.turn_id && point.frameIndex === effectiveFrameIndex;
                    const isSelected = point.dominantRecordId === activeSourceRecordId;
                    const isInteractive = point.turnId === currentTurn?.turn_id;
                    return (
                      <button
                        className={`manifold-force-point is-${steeringGateClass(point.gate)}${
                          isCurrent ? ' is-current' : ''
                        }${isSelected ? ' is-selected' : ''}${
                          point.isSelectedTurn ? '' : ' is-contextual'
                        }`}
                        disabled={!isInteractive}
                        key={point.key}
                        onMouseDown={(event) => event.stopPropagation()}
                        onClick={() => {
                          if (isInteractive) {
                            onPointSelect(point.frameIndex, point.dominantRecordId);
                          }
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
                {selectedSignal ? (
                  <div className="manifold-force-popup" role="dialog" aria-label="Selected steering point details">
                    <div className="manifold-force-popup__head">
                      <div>
                        <div className="manifold-force-popup__eyebrow">
                          <span>{selectedGate ? manifoldGateLabel(selectedGate) : 'Selected point'}</span>
                          <span>
                            {selectedGate ? steeringPhaseLabel(selectedGate.phase) : 'Active'}
                          </span>
                        </div>
                        <div className="manifold-force-popup__title">
                          {selectedSignal.summary || manifoldSignalLabel(selectedSignal)}
                        </div>
                      </div>
                    </div>
                    <div className="manifold-force-popup__meta">
                      {manifoldSignalLabel(selectedSignal)} · {steeringGateLabel(selectedSignal.gate)} ·{' '}
                      {steeringPhaseLabel(selectedSignal.phase)} · {selectedSignal.level} ·{' '}
                      {selectedSignal.magnitude_percent}%
                    </div>
                    <div className="manifold-force-popup__detail">
                      <div>
                        Frame {effectiveFrameIndex + 1} · sequence {currentFrame?.sequence || 'n/a'} · anchor{' '}
                        {manifoldAnchorLabel(selectedSignal.anchor)}
                      </div>
                      {selectedSlice ? (
                        <div>
                          Piano slice contains {selectedSlice.pointCount} steering
                          {selectedSlice.pointCount === 1 ? ' point' : ' points'} at this step.
                        </div>
                      ) : null}
                      {selectedResolverOutcome ? (
                        <>
                          <div className="manifold-force-popup__resolver">
                            <strong>{resolverOutcomeTitle(selectedResolverOutcome)}</strong>
                            <span>{resolverOutcomeMeta(selectedResolverOutcome)}</span>
                          </div>
                          {selectedResolverOutcome.path ? (
                            <div className="manifold-force-popup__path">
                              {selectedResolverOutcome.path}
                            </div>
                          ) : null}
                          <div>{resolverOutcomeNarrative(selectedResolverOutcome)}</div>
                        </>
                      ) : null}
                      {!!selectedSignal.contributions.length && (
                        <div className="manifold-force-popup__contributions">
                          {selectedSignal.contributions.slice(0, 3).map((contribution) => (
                            <div key={`${selectedSignal.snapshot_record_id}:${contribution.source}`}>
                              <strong>{contribution.source}</strong> · {contribution.share_percent}% ·{' '}
                              {contribution.rationale || signalKindLabel(selectedSignal.kind)}
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>
                ) : null}
              </div>
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
