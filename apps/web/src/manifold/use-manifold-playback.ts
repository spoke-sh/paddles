import { useEffect, useMemo, useRef, useState } from 'react';

import {
  MANIFOLD_PLAYBACK_STEP_MS,
  resolverSignalDetails,
} from '../runtime-helpers';
import type { ManifoldTurnProjection } from '../runtime-types';
import type { ManifoldForcePoint } from './gate-field';
import { buildGateField, frameForSelectedRecord } from './gate-field';
type ManifoldSelection =
  | { mode: 'auto' }
  | { mode: 'closed' }
  | {
      mode: 'point';
      key: string;
      gate: string;
      recordId: string | null;
      frameIndex: number;
    };

export function useManifoldPlayback(
  turns: ManifoldTurnProjection[],
  selectedTurnId: string | null
) {
  const [frameIndex, setFrameIndex] = useState<number | null>(null);
  const [tailMode, setTailMode] = useState(true);
  const [playing, setPlaying] = useState(false);
  const [selection, setSelection] = useState<ManifoldSelection>({ mode: 'auto' });
  const previousResolvedTurnIdRef = useRef<string | null>(null);

  const currentTurn =
    turns.find((turn) => turn.turn_id === selectedTurnId) ||
    turns[turns.length - 1] ||
    null;
  const resolvedTurnId = currentTurn?.turn_id || null;
  const effectiveFrameIndex = currentTurn
    ? tailMode
      ? Math.max(0, currentTurn.frames.length - 1)
      : Math.max(0, Math.min(currentTurn.frames.length - 1, frameIndex ?? 0))
    : 0;
  const gateField = useMemo(
    () => buildGateField(turns, selectedTurnId),
    [turns, selectedTurnId]
  );
  const currentFrame = currentTurn?.frames[effectiveFrameIndex] || null;
  const activeSignals = currentFrame?.active_signals || [];
  const selectedPoint =
    selection.mode === 'point'
      ? gateField.points.find((point) => point.key === selection.key) || null
      : null;
  const selectedSignal =
    selection.mode === 'closed'
      ? null
      : selection.mode === 'point'
        ? activeSignals.find((signal) => signal.snapshot_record_id === selection.recordId) ||
          (selection.gate
            ? activeSignals.find((signal) => signal.gate === selection.gate)
            : null) ||
          activeSignals[0] ||
          null
        : activeSignals[0] || null;
  const selectedSignalFrame =
    selection.mode === 'closed'
      ? null
      : selection.mode === 'point'
        ? selectedSignal
          ? frameForSelectedRecord(currentTurn, selectedSignal.snapshot_record_id) || currentFrame
          : currentFrame
        : currentFrame;
  const selectedGate =
    selection.mode === 'closed'
      ? null
      : selection.mode === 'point'
        ? currentFrame?.gates.find((gate) => gate.dominant_record_id === selection.recordId) ||
          (selection.gate
            ? currentFrame?.gates.find((gate) => gate.gate === selection.gate) || null
            : null) ||
          null
        : currentFrame?.gates[0] || null;
  const selectedResolverOutcome = resolverSignalDetails(selectedSignal);
  const totalFrames = turns.reduce((sum, turn) => sum + turn.frames.length, 0);
  const selectedSourceRecordId =
    selection.mode === 'point'
      ? selection.recordId || selectedSignal?.snapshot_record_id || null
      : selectedSignal?.snapshot_record_id || null;
  const selectedPointKey = selection.mode === 'point' ? selection.key : null;

  useEffect(() => {
    const previousResolvedTurnId = previousResolvedTurnIdRef.current;
    previousResolvedTurnIdRef.current = resolvedTurnId;
    if (previousResolvedTurnId == null || previousResolvedTurnId === resolvedTurnId) {
      return;
    }
    setTailMode(true);
    setFrameIndex(null);
    setSelection({ mode: 'auto' });
    setPlaying(false);
  }, [resolvedTurnId]);

  useEffect(() => {
    if (!playing || !currentTurn || currentTurn.frames.length <= 1) {
      return;
    }
    const handle = window.setInterval(() => {
      setTailMode(false);
      setFrameIndex((current) => {
        const next = (current ?? 0) + 1;
        if (next >= currentTurn.frames.length) {
          return 0;
        }
        return next;
      });
    }, MANIFOLD_PLAYBACK_STEP_MS);
    return () => window.clearInterval(handle);
  }, [currentTurn, playing]);

  function selectFrame(nextFrameIndex: number) {
    setTailMode(false);
    setFrameIndex(nextFrameIndex);
  }

  function selectPoint(point: ManifoldForcePoint) {
    setTailMode(false);
    setFrameIndex(point.frameIndex);
    setSelection((current) =>
      current.mode === 'point' &&
      current.key === point.key &&
      effectiveFrameIndex === point.frameIndex
        ? { mode: 'closed' }
        : {
            mode: 'point',
            key: point.key,
            gate: point.gate,
            recordId: point.dominantRecordId,
            frameIndex: point.frameIndex,
          }
    );
  }

  function clearSelection() {
    setSelection({ mode: 'closed' });
  }

  function replay() {
    setTailMode(false);
    setFrameIndex(0);
    setPlaying(false);
  }

  return {
    currentFrame,
    currentTurn,
    effectiveFrameIndex,
    gateField,
    playing,
    selectedGate,
    selectedPointKey,
    selectedResolverOutcome,
    selectedSignal,
    selectedSignalFrame,
    selectedSourceRecordId,
    totalFrames,
    clearSelection,
    selectFrame,
    selectPoint,
    replay,
    setPlaying,
  };
}
