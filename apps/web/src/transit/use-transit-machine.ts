import { useEffect, useMemo, useState } from 'react';

import type { ConversationProjectionSnapshot } from '../runtime-types';
import {
  projectConversationMachine,
  type MachineMomentProjection,
  type MachineTurnProjection,
} from '../trace-machine/machine-projection';

function latestTurn(turns: MachineTurnProjection[]) {
  return turns[turns.length - 1] || null;
}

function latestMoment(turn: MachineTurnProjection | null) {
  return turn?.moments[turn.moments.length - 1] || null;
}

export function useTransitMachine(projection: ConversationProjectionSnapshot | null) {
  const machine = useMemo(
    () => (projection ? projectConversationMachine(projection) : null),
    [projection]
  );
  const turns = machine?.turns || [];
  const [selectedTurnId, setSelectedTurnId] = useState<string | null>(null);
  const [selectedMomentId, setSelectedMomentId] = useState<string | null>(null);

  const currentTurn =
    turns.find((turn) => turn.turnId === selectedTurnId) || latestTurn(turns);
  const currentMoment =
    currentTurn?.moments.find((moment) => moment.momentId === selectedMomentId) ||
    latestMoment(currentTurn);

  useEffect(() => {
    if (!turns.length) {
      setSelectedTurnId(null);
      setSelectedMomentId(null);
      return;
    }

    if (!currentTurn) {
      const nextTurn = latestTurn(turns);
      setSelectedTurnId(nextTurn?.turnId || null);
      setSelectedMomentId(latestMoment(nextTurn)?.momentId || null);
      return;
    }

    if (currentTurn.turnId !== selectedTurnId) {
      setSelectedTurnId(currentTurn.turnId);
    }

    if (!currentMoment) {
      setSelectedMomentId(latestMoment(currentTurn)?.momentId || null);
    }
  }, [currentMoment, currentTurn, selectedTurnId, turns]);

  function selectMoment(momentId: string) {
    setSelectedMomentId(momentId);
  }

  return {
    taskId: machine?.taskId || projection?.task_id || null,
    turns,
    currentTurn,
    currentMoment,
    selectMoment,
  };
}

export type TransitMachineSelection = {
  currentMoment: MachineMomentProjection | null;
  currentTurn: MachineTurnProjection | null;
  taskId: string | null;
  turns: MachineTurnProjection[];
  selectMoment: (momentId: string) => void;
};
