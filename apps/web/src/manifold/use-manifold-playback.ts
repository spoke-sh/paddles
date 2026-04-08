import { useEffect, useMemo, useState } from 'react';

import {
  MANIFOLD_PLAYBACK_STEP_MS,
  resolverSignalDetails,
} from '../runtime-helpers';
import type { ManifoldTurnProjection } from '../runtime-types';
import { buildGateField, frameForSelectedRecord } from './gate-field';

export function useManifoldPlayback(
  turns: ManifoldTurnProjection[],
  selectedTurnId: string | null
) {
  const [frameIndex, setFrameIndex] = useState<number | null>(null);
  const [tailMode, setTailMode] = useState(true);
  const [playing, setPlaying] = useState(false);
  const [selectedSourceRecordId, setSelectedSourceRecordId] = useState<string | null>(null);

  const currentTurn =
    turns.find((turn) => turn.turn_id === selectedTurnId) ||
    turns[turns.length - 1] ||
    null;
  const effectiveFrameIndex = currentTurn
    ? tailMode
      ? Math.max(0, currentTurn.frames.length - 1)
      : Math.max(0, Math.min(currentTurn.frames.length - 1, frameIndex ?? 0))
    : 0;
  const currentFrame = currentTurn?.frames[effectiveFrameIndex] || null;
  const activeSignals = currentFrame?.active_signals || [];
  const selectedSignal =
    activeSignals.find((signal) => signal.snapshot_record_id === selectedSourceRecordId) ||
    activeSignals[0] ||
    null;
  const selectedSignalFrame =
    frameForSelectedRecord(currentTurn, selectedSourceRecordId) || currentFrame;
  const selectedGate =
    currentFrame?.gates.find((gate) => gate.dominant_record_id === selectedSourceRecordId) ||
    currentFrame?.gates.find((gate) => gate.gate === selectedSignal?.gate) ||
    currentFrame?.gates[0] ||
    null;
  const selectedResolverOutcome = resolverSignalDetails(selectedSignal);
  const gateField = useMemo(() => buildGateField(currentTurn), [currentTurn]);
  const totalFrames = turns.reduce((sum, turn) => sum + turn.frames.length, 0);

  useEffect(() => {
    setTailMode(true);
    setFrameIndex(null);
    setSelectedSourceRecordId(null);
    setPlaying(false);
  }, [selectedTurnId]);

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

  function selectPoint(nextFrameIndex: number, recordId: string | null) {
    setTailMode(false);
    setFrameIndex(nextFrameIndex);
    setSelectedSourceRecordId(recordId);
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
    selectedResolverOutcome,
    selectedSignal,
    selectedSignalFrame,
    selectedSourceRecordId,
    totalFrames,
    selectFrame,
    selectPoint,
    replay,
    setPlaying,
    setSelectedSourceRecordId,
  };
}
