import { useEffect, useMemo, useState } from 'react';

import {
  aggregateSignalContributions,
  kindEntry,
  primaryArtifact,
  rawRecordBody,
  recordSummary,
  renderedRecordBody,
  strongestSignalSnapshot,
} from '../runtime-helpers';
import type { ConversationProjectionSnapshot, ForensicRecordProjection } from '../runtime-types';
import { DEFAULT_MACHINE_SELECTION } from '../trace-machine/machine-model';
import {
  projectConversationMachine,
  type MachineMomentProjection,
  type MachineTurnProjection,
} from '../trace-machine/machine-projection';
import { previousArtifactBaseline } from './forensic-selectors';

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

function latestTurn<T extends { turnId: string }>(turns: T[]) {
  return turns[turns.length - 1] || null;
}

function latestMoment(turn: MachineTurnProjection | null) {
  return turn?.moments[turn.moments.length - 1] || null;
}

function primaryRecordForMoment(
  records: ForensicRecordProjection[],
  moment: MachineMomentProjection | null
) {
  if (!moment) {
    return records[records.length - 1] || null;
  }
  const linkedRecordIds = new Set(moment.raw.forensicRecordIds);
  return (
    records.find((record) => record.record.record_id === moment.raw.primaryForensicRecordId) ||
    records.find((record) => linkedRecordIds.has(record.record.record_id)) ||
    null
  );
}

export function useInspectorSelection(projection: ConversationProjectionSnapshot | null) {
  const machine = useMemo(
    () => (projection ? projectConversationMachine(projection) : null),
    [projection]
  );
  const turns = projection?.forensics.turns || [];
  const machineTurns = machine?.turns || [];
  const [selection, setSelection] = useState(DEFAULT_MACHINE_SELECTION);

  const currentMachineTurn =
    machineTurns.find((turn) => turn.turnId === selection.selectedTurnId) || latestTurn(machineTurns);
  const currentTurn =
    turns.find((turn) => turn.turn_id === currentMachineTurn?.turnId) || turns[turns.length - 1] || null;
  const currentMoment =
    currentMachineTurn?.moments.find((moment) => moment.momentId === selection.selectedMomentId) ||
    latestMoment(currentMachineTurn);
  const currentRecord = primaryRecordForMoment(currentTurn?.records || [], currentMoment);
  const linkedRecordIds = new Set(currentMoment?.raw.forensicRecordIds || []);
  const linkedRecords =
    currentTurn?.records.filter((record) => linkedRecordIds.has(record.record.record_id)) || [];
  const signalRecords = linkedRecords.filter((record) => kindEntry(record).key === 'SignalSnapshot');
  const strongestSignal = strongestSignalSnapshot(signalRecords);
  const strongestSignalValue = strongestSignal ? kindEntry(strongestSignal).value : null;
  const contributions = aggregateSignalContributions(signalRecords);
  const baseline = previousArtifactBaseline(currentTurn, currentRecord);
  const currentArtifact = currentRecord ? primaryArtifact(currentRecord) : null;
  const currentPayload = currentRecord
    ? {
        raw: rawRecordBody(currentRecord),
        rendered: renderedRecordBody(currentRecord),
      }
    : null;

  useEffect(() => {
    if (!machineTurns.length) {
      setSelection(DEFAULT_MACHINE_SELECTION);
      return;
    }

    const nextTurn = currentMachineTurn || latestTurn(machineTurns);
    const nextMoment = currentMoment || latestMoment(nextTurn);

    if (
      selection.selectedTurnId !== nextTurn?.turnId ||
      selection.selectedMomentId !== nextMoment?.momentId
    ) {
      setSelection((previous) => ({
        ...previous,
        selectedTurnId: nextTurn?.turnId || null,
        selectedMomentId: nextMoment?.momentId || null,
      }));
    }
  }, [
    currentMachineTurn,
    currentMoment,
    machineTurns,
    selection.selectedMomentId,
    selection.selectedTurnId,
  ]);

  function selectTurn(turnId: string) {
    const turn = machineTurns.find((candidate) => candidate.turnId === turnId) || null;
    setSelection((previous) => ({
      ...previous,
      selectedTurnId: turn?.turnId || null,
      selectedMomentId: latestMoment(turn)?.momentId || null,
    }));
  }

  function selectMoment(momentId: string) {
    setSelection((previous) => ({
      ...previous,
      selectedMomentId: momentId,
    }));
  }

  function toggleInternals() {
    setSelection((previous) => ({
      ...previous,
      showInternals: !previous.showInternals,
    }));
  }

  return {
    baseline,
    contributions,
    currentArtifact,
    currentMachineTurn,
    currentMoment,
    currentPayload,
    currentRecord,
    currentTurn,
    linkedRecords,
    machineTurns,
    showInternals: selection.showInternals,
    strongestSignalValue,
    taskId: machine?.taskId || projection?.task_id || null,
    toggleInternals,
    turns,
    selectMoment,
    selectTurn,
  };
}

export type InspectorSelectionState = ReturnType<typeof useInspectorSelection>;

export function linkedRecordHeadline(record: ForensicRecordProjection | null) {
  return record ? recordSummary(record) : 'Awaiting a linked forensic record';
}
