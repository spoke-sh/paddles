import {
  KIND_COLORS,
  formatTraceKind,
  traceNodeFamily,
} from '../runtime-helpers';
import type { ConversationTraceGraphNode } from '../runtime-types';

interface TransitObservatoryProps {
  allNodes: ConversationTraceGraphNode[];
  branchLabels: Record<string, string>;
  selectedNode: ConversationTraceGraphNode | null;
  visibleNodes: ConversationTraceGraphNode[];
  onSelectNode: (nodeId: string) => void;
}

export function TransitObservatory({
  allNodes,
  branchLabels,
  selectedNode,
  visibleNodes,
  onSelectNode,
}: TransitObservatoryProps) {
  const selectedFamily = selectedNode ? traceNodeFamily(selectedNode.kind) : null;
  const visibleNodeIds = new Set(visibleNodes.map((node) => node.id));

  return (
    <>
      <div
        className="trace-observatory"
        id="trace-observatory"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="trace-observatory__eyebrow">Transit Observatory</div>
        <div className="trace-observatory__title">
          {selectedNode ? selectedNode.label : 'Select a step to inspect its branch orbit.'}
        </div>
        <div className="trace-observatory__meta">
          {selectedNode
            ? `step ${selectedNode.sequence} · ${formatTraceKind(selectedNode.kind)} · ${
                selectedNode.branch_id ? branchLabels[selectedNode.branch_id] || selectedNode.branch_id : 'mainline'
              }`
            : `${visibleNodes.length} visible steps`}
        </div>

        {selectedNode ? (
          <div className="trace-observatory__popup" id="trace-observatory-popup">
            <div className="trace-observatory__popup-chip-row">
              <span className="trace-observatory__chip">{selectedNode.id}</span>
              {selectedFamily ? <span className="trace-observatory__chip">{selectedFamily}</span> : null}
            </div>
            <div className="trace-observatory__popup-detail">{selectedNode.label}</div>
          </div>
        ) : null}
      </div>

      <div
        className="trace-step-scrubber"
        id="trace-step-scrubber"
        onMouseDown={(event) => event.stopPropagation()}
      >
        {allNodes.map((node) => (
          <button
            className={`trace-step-scrubber__chip${selectedNode?.id === node.id ? ' is-active' : ''}${
              visibleNodeIds.has(node.id) ? '' : ' is-muted'
            }`}
            data-trace-scrub-node-id={node.id}
            key={node.id}
            onClick={() => onSelectNode(node.id)}
            style={{ ['--trace-node-chip' as string]: KIND_COLORS[node.kind] || '#8b949e' }}
            type="button"
          >
            <span>{node.sequence}</span>
            <strong>{node.label}</strong>
          </button>
        ))}
      </div>
    </>
  );
}
