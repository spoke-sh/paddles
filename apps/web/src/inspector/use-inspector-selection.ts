import { useEffect, useState } from 'react';

import {
  aggregateSignalContributions,
  kindEntry,
  latestRecordForTurn,
  modelCallsForTurn,
  plannerStepsForTurn,
  recordsForTurn,
  strongestSignalSnapshot,
} from '../runtime-helpers';
import type { ForensicRecordProjection, ForensicTurnProjection } from '../runtime-types';

export type FocusState = {
  kind: 'all' | 'model_call' | 'planner_step';
  id: string | null;
};

export type InspectorSelectionMode = 'conversation' | 'turn' | 'record';
export type InspectorDetailMode = 'rendered' | 'raw';

export interface InspectorPlannerStep {
  id: string;
  recordId: string;
  label: string;
}

export interface InspectorModelCall {
  id: string;
  lane: string;
  category: string;
  provider: string;
  model: string;
  summary: string;
}

export function useInspectorSelection(turns: ForensicTurnProjection[]) {
  const [selectedTurnId, setSelectedTurnId] = useState<string | null>(null);
  const [selectedRecordId, setSelectedRecordId] = useState<string | null>(null);
  const [selectionMode, setSelectionMode] =
    useState<InspectorSelectionMode>('record');
  const [detailMode, setDetailMode] =
    useState<InspectorDetailMode>('rendered');
  const [focus, setFocus] = useState<FocusState>({ kind: 'all', id: null });

  const currentTurn =
    turns.find((turn) => turn.turn_id === selectedTurnId) ||
    turns[turns.length - 1] ||
    null;
  const records = recordsForTurn(currentTurn, focus);
  const currentRecord =
    selectionMode === 'record'
      ? records.find((record) => record.record.record_id === selectedRecordId) ||
        records[records.length - 1] ||
        null
      : null;
  const modelCalls = modelCallsForTurn(currentTurn);
  const plannerSteps = plannerStepsForTurn(currentTurn);
  const signalRecords = currentTurn
    ? currentTurn.records.filter((record) => kindEntry(record).key === 'SignalSnapshot')
    : [];
  const strongestSignal = strongestSignalSnapshot(signalRecords);
  const strongestSignalValue = strongestSignal ? kindEntry(strongestSignal).value : null;
  const contributions = aggregateSignalContributions(signalRecords);
  const comparisonRecord = currentRecord || latestRecordForTurn(currentTurn, focus);

  useEffect(() => {
    if (!turns.length) {
      setSelectedTurnId(null);
      setSelectedRecordId(null);
      return;
    }
    if (!selectedTurnId || !turns.some((turn) => turn.turn_id === selectedTurnId)) {
      const lastTurn = turns[turns.length - 1];
      setSelectedTurnId(lastTurn.turn_id);
      if (lastTurn.records.length) {
        setSelectedRecordId(lastTurn.records[lastTurn.records.length - 1].record.record_id);
      }
    }
  }, [selectedTurnId, turns]);

  useEffect(() => {
    if (selectionMode !== 'record') {
      return;
    }
    if (records.length && !records.some((record) => record.record.record_id === selectedRecordId)) {
      setSelectedRecordId(records[records.length - 1].record.record_id);
    }
  }, [records, selectedRecordId, selectionMode]);

  function selectConversation() {
    setSelectionMode('conversation');
  }

  function selectTurn(turnId: string) {
    const turn = turns.find((candidate) => candidate.turn_id === turnId);
    setSelectedTurnId(turnId);
    setSelectedRecordId(turn?.records[turn.records.length - 1]?.record.record_id || null);
    setSelectionMode('turn');
    setFocus({ kind: 'all', id: null });
  }

  function focusAllRecords() {
    setFocus({ kind: 'all', id: null });
    setSelectionMode('turn');
  }

  function focusModelCall(modelCallId: string) {
    setFocus({ kind: 'model_call', id: modelCallId });
    setSelectionMode('turn');
  }

  function focusPlannerStep(plannerStepId: string) {
    setFocus({ kind: 'planner_step', id: plannerStepId });
    setSelectionMode('turn');
  }

  function selectRecord(recordId: string) {
    setSelectionMode('record');
    setSelectedRecordId(recordId);
  }

  return {
    comparisonRecord,
    contributions,
    currentRecord,
    currentTurn,
    detailMode,
    focus,
    modelCalls,
    plannerSteps,
    records,
    selectionMode,
    signalRecords,
    strongestSignalValue,
    setDetailMode,
    focusAllRecords,
    focusModelCall,
    focusPlannerStep,
    selectConversation,
    selectRecord,
    selectTurn,
  };
}
