import {
  KIND_COLORS,
  formatTraceKind,
  traceNodeDirection,
} from '../runtime-helpers';
import type { ConversationTraceGraphNode } from '../runtime-types';

interface TraceNodeProps {
  branchLabel: string | null;
  detailLevel: string;
  node: ConversationTraceGraphNode;
  nodeIndex: number;
  rowIndex: number;
  rowLength: number;
  rowsLength: number;
  selected: boolean;
  visibleNodeCount: number;
  onSelectNode: (nodeId: string) => void;
}

export function TraceNode({
  branchLabel,
  detailLevel,
  node,
  nodeIndex,
  rowIndex,
  rowLength,
  rowsLength,
  selected,
  visibleNodeCount,
  onSelectNode,
}: TraceNodeProps) {
  const recencyIndex = visibleNodeCount - (rowIndex * rowLength + nodeIndex) - 1;
  const recencyRatio =
    visibleNodeCount <= 1 ? 1 : 1 - recencyIndex / Math.max(visibleNodeCount - 1, 1);
  const depth = Math.max(0, Math.min(1, recencyRatio));
  const sunTrailDepth = Math.max(0, 1 - recencyIndex / 3);
  const direction = traceNodeDirection(rowIndex, nodeIndex, rowLength, rowsLength);
  const summary =
    detailLevel === 'overview' ? node.id : branchLabel || formatTraceKind(node.kind);

  return (
    <div
      aria-label={`step ${node.sequence} ${node.label}`}
      className={`trace-node${recencyIndex === 0 ? ' trace-node--latest' : ''}${
        selected ? ' is-selected' : ''
      } trace-node--${direction}`}
      data-trace-node-id={node.id}
      key={node.id}
      onClick={(event) => {
        event.stopPropagation();
        onSelectNode(node.id);
      }}
      onMouseDown={(event) => {
        event.stopPropagation();
      }}
      onKeyDown={(event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          onSelectNode(node.id);
        }
      }}
      role="button"
      style={
        {
          ['--node-color' as string]: KIND_COLORS[node.kind] || '#8b949e',
          ['--node-raise' as string]: `${(-(2 + depth * 6)).toFixed(2)}px`,
          ['--node-shadow-x' as string]: `${(5 + depth * 2.5).toFixed(2)}px`,
          ['--node-shadow-y' as string]: `${(6 + depth * 4).toFixed(2)}px`,
          ['--node-shadow-blur' as string]: `${(12 + depth * 7).toFixed(2)}px`,
          ['--node-shadow-alpha' as string]: `${(0.1 + depth * 0.12).toFixed(3)}`,
          ['--node-shadow-warm-alpha' as string]: `${(0.02 + sunTrailDepth * 0.1).toFixed(3)}`,
          ['--node-shadow-warm-x' as string]: `${(1 + sunTrailDepth * 2).toFixed(2)}px`,
          ['--node-shadow-warm-y' as string]: `${(2 + sunTrailDepth * 4).toFixed(2)}px`,
          ['--node-shadow-warm-blur' as string]: `${(4 + sunTrailDepth * 10).toFixed(2)}px`,
          ['--node-tilt-x' as string]: `${direction === 'down' ? 8 : 5}deg`,
          ['--node-tilt-y' as string]: `${direction === 'ltr' ? -4 : direction === 'rtl' ? 4 : 0}deg`,
          ['--node-specular-alpha' as string]: `${(0.68 + depth * 0.2).toFixed(3)}`,
        } as React.CSSProperties
      }
      tabIndex={0}
    >
      <div className="trace-node__hex">
        <div className="trace-node__sequence">step {node.sequence}</div>
        <div className="trace-node__kind">{formatTraceKind(node.kind)}</div>
        <div className="trace-node__label">{node.label}</div>
        <div className="trace-node__summary">{summary}</div>
        <div className="trace-node__branch">{branchLabel || 'mainline'}</div>
      </div>
      <div className="trace-node__detail">
        <div className="trace-node__detail-title">{node.label}</div>
        <div className="trace-node__detail-meta">
          {`kind: ${formatTraceKind(node.kind)}\n${
            branchLabel ? `branch: ${branchLabel}\n` : ''
          }record: ${node.id}`}
        </div>
      </div>
    </div>
  );
}
