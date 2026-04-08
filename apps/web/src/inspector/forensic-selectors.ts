import {
  primaryArtifact,
  rawRecordBody,
  recordSummary,
  renderedRecordBody,
  truncate,
} from '../runtime-helpers';
import type { ForensicRecordProjection, ForensicTurnProjection } from '../runtime-types';

export function previousArtifactBaseline(
  turn: ForensicTurnProjection | null,
  recordProjection: ForensicRecordProjection | null
) {
  if (!turn || !recordProjection) {
    return null;
  }
  for (let index = turn.records.length - 1; index >= 0; index -= 1) {
    const candidate = turn.records[index];
    if (candidate.record.sequence >= recordProjection.record.sequence) {
      continue;
    }
    if (primaryArtifact(candidate)) {
      return candidate;
    }
  }
  return null;
}

export function comparisonSnippet(
  recordProjection: ForensicRecordProjection | null,
  detailMode: 'rendered' | 'raw'
) {
  if (!recordProjection) {
    return 'No lineage artifact available.';
  }
  const body =
    detailMode === 'raw' ? rawRecordBody(recordProjection) : renderedRecordBody(recordProjection);
  return truncate(body.replace(/\s+/g, ' ').trim(), 180);
}

export function comparisonTitle(recordProjection: ForensicRecordProjection | null) {
  return recordProjection ? recordSummary(recordProjection) : 'turn';
}
