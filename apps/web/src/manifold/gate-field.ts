import {
  STEERING_GATE_ORDER,
  manifoldGateLabel,
} from '../runtime-helpers';
import type { ManifoldFrame, ManifoldTurnProjection } from '../runtime-types';

export type ManifoldForcePoint = {
  key: string;
  turnId: string;
  gate: string;
  laneIndex: number;
  frameIndex: number;
  globalFrameIndex: number;
  sequence: number;
  leftPercent: number;
  topPercent: number;
  magnitudePercent: number;
  phase: string;
  level: string;
  dominantRecordId: string | null;
  dominantSignalKind: string;
  isSelectedTurn: boolean;
  label: string;
};

export type ManifoldForceLink = {
  key: string;
  turnId: string;
  gate: string;
  leftPercent: number;
  topPercent: number;
  widthPercent: number;
  magnitudePercent: number;
  isSelectedTurn: boolean;
};

function laneIndexForGate(gate: string) {
  const index = STEERING_GATE_ORDER.indexOf(gate);
  return index >= 0 ? index : STEERING_GATE_ORDER.length - 1;
}

export function buildGateField(
  turns: ManifoldTurnProjection[],
  selectedTurnId: string | null
): {
  points: ManifoldForcePoint[];
  links: ManifoldForceLink[];
} {
  const totalFrames = turns.reduce((sum, turn) => sum + turn.frames.length, 0);
  if (!totalFrames) {
    return { points: [], links: [] };
  }

  const points: ManifoldForcePoint[] = [];
  const links: ManifoldForceLink[] = [];
  const lastGlobalFrameIndex = Math.max(1, totalFrames - 1);
  let globalFrameIndex = 0;

  turns.forEach((turn) => {
    const pointsByGate = new Map<string, ManifoldForcePoint[]>();
    const isSelectedTurn = !selectedTurnId || turn.turn_id === selectedTurnId;

    turn.frames.forEach((frame, frameIndex) => {
      frame.gates.forEach((gate) => {
        const normalizedGate = gate.gate || 'containment';
        const laneIndex = laneIndexForGate(normalizedGate);
        const point: ManifoldForcePoint = {
          key: `${turn.turn_id}:${frame.record_id}:${normalizedGate}`,
          turnId: turn.turn_id,
          gate: normalizedGate,
          laneIndex,
          frameIndex,
          globalFrameIndex,
          sequence: frame.sequence,
          leftPercent: 12 + (globalFrameIndex / lastGlobalFrameIndex) * 76,
          topPercent: 18 + laneIndex * 28,
          magnitudePercent: gate.magnitude_percent || 0,
          phase: gate.phase,
          level: gate.level,
          dominantRecordId: gate.dominant_record_id || null,
          dominantSignalKind: gate.dominant_signal_kind,
          isSelectedTurn,
          label: manifoldGateLabel(gate),
        };
        points.push(point);
        const gatePoints = pointsByGate.get(normalizedGate) || [];
        gatePoints.push(point);
        pointsByGate.set(normalizedGate, gatePoints);
      });

      globalFrameIndex += 1;
    });

    for (const [gate, gatePoints] of pointsByGate.entries()) {
      for (let index = 0; index < gatePoints.length - 1; index += 1) {
        const current = gatePoints[index];
        const next = gatePoints[index + 1];
        links.push({
          key: `${current.key}->${next.key}`,
          turnId: turn.turn_id,
          gate,
          leftPercent: current.leftPercent,
          topPercent: current.topPercent,
          widthPercent: Math.max(3, next.leftPercent - current.leftPercent),
          magnitudePercent: Math.round((current.magnitudePercent + next.magnitudePercent) / 2),
          isSelectedTurn,
        });
      }
    }
  });

  return { points, links };
}

export function frameForSelectedRecord(
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
