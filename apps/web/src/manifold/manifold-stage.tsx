import {
  STEERING_GATE_COLORS,
  STEERING_GATE_ORDER,
  manifoldAnchorLabel,
  steeringGateClass,
  steeringGateLabel,
} from '../runtime-helpers';
import type { ManifoldFrame, ManifoldTurnProjection } from '../runtime-types';
import type { ManifoldForceLink, ManifoldForcePoint } from './gate-field';
import type { ManifoldCameraState } from './use-manifold-camera';

interface ManifoldStageProps {
  camera: ManifoldCameraState;
  currentFrame: ManifoldFrame | null;
  currentTurn: ManifoldTurnProjection | null;
  dragMode: 'tilt' | 'pan' | 'rotate' | null;
  effectiveFrameIndex: number;
  gateField: { points: ManifoldForcePoint[]; links: ManifoldForceLink[] };
  playing: boolean;
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
                        onClick={() => onPointSelect(point.frameIndex, point.dominantRecordId)}
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
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
