import type {
  ConversationProjectionSnapshot,
  ConversationProjectionUpdate,
} from '../runtime-types';

export function projectionVersion(
  snapshot: ConversationProjectionSnapshot | null | undefined
) {
  if (!snapshot) {
    return 0;
  }

  return snapshot.trace_graph.nodes.reduce(
    (latest, node) => Math.max(latest, node.sequence),
    0
  );
}

export function reduceProjectionUpdate(
  current: ConversationProjectionSnapshot | null,
  update: ConversationProjectionUpdate
) {
  if (current && current.task_id !== update.task_id) {
    return current;
  }

  if (update.reducer !== 'replace_snapshot') {
    return current;
  }

  if (update.version <= projectionVersion(current)) {
    return current;
  }

  return update.snapshot;
}

export function reduceProjectionSnapshot(
  current: ConversationProjectionSnapshot | null,
  nextSnapshot: ConversationProjectionSnapshot
) {
  if (current && current.task_id !== nextSnapshot.task_id) {
    return current;
  }

  if (projectionVersion(nextSnapshot) <= projectionVersion(current)) {
    return current;
  }

  return nextSnapshot;
}
