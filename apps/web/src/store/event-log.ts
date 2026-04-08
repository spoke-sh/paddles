import { eventRow } from '../runtime-helpers';
import type { ProjectionTurnEvent, TurnEvent } from '../runtime-types';

export interface RuntimeEventRow {
  id: string;
  badge: string;
  badgeClass: string;
  text: string;
  diff?: string;
  output?: string;
  streamKey?: string;
}

type RuntimeEventRowDraft = Omit<RuntimeEventRow, 'id'>;

function toRuntimeEventRowDraft(
  payload: ProjectionTurnEvent | TurnEvent
): RuntimeEventRowDraft | null {
  const nextRow = eventRow(payload);
  if (!nextRow) {
    return null;
  }
  return {
    badge: nextRow.badge,
    badgeClass: nextRow.badgeClass,
    text: nextRow.text,
    diff: 'diff' in nextRow ? nextRow.diff : undefined,
    output: 'output' in nextRow ? nextRow.output : undefined,
    streamKey: 'streamKey' in nextRow ? nextRow.streamKey : undefined,
  };
}

export function appendRuntimeEventRow(
  current: RuntimeEventRow[],
  row: RuntimeEventRowDraft
) {
  if (row.streamKey) {
    const existingIndex = current.findIndex(
      (item) => item.streamKey === row.streamKey
    );
    if (existingIndex >= 0) {
      const next = [...current];
      const existing = next[existingIndex];
      next[existingIndex] = {
        ...existing,
        badge: row.badge,
        badgeClass: row.badgeClass,
        text: row.text,
        output: `${existing.output || ''}${row.output || ''}`,
      };
      return next;
    }
  }

  return [
    ...current,
    { id: row.streamKey || `${Date.now()}-${current.length}`, ...row },
  ].slice(-64);
}

export function reduceRuntimeTurnEvent(
  current: RuntimeEventRow[],
  payload: ProjectionTurnEvent | TurnEvent
) {
  const row = toRuntimeEventRowDraft(payload);
  if (!row) {
    return current;
  }
  return appendRuntimeEventRow(current, row);
}

export function sanitizePromptHistory(promptHistory: string[]) {
  return promptHistory.filter((prompt) => prompt.trim().length > 0);
}
