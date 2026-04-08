import type { ConversationTraceGraphNode } from '../runtime-types';
import { TraceNode } from './trace-node';

interface TraceBoardProps {
  boardRef: React.RefObject<HTMLDivElement | null>;
  branchLabels: Record<string, string>;
  canvasRef: React.RefObject<HTMLDivElement | null>;
  detailLevel: string;
  layout: { columnGap: number; rowGap: number };
  pan: { x: number; y: number };
  pathD: string;
  rows: ConversationTraceGraphNode[][];
  visibleNodeCount: number;
  zoom: number;
  onMouseDown: (event: React.MouseEvent<HTMLDivElement>) => void;
  onWheel: (event: React.WheelEvent<HTMLDivElement>) => void;
}

export function TraceBoard({
  boardRef,
  branchLabels,
  canvasRef,
  detailLevel,
  layout,
  pan,
  pathD,
  rows,
  visibleNodeCount,
  zoom,
  onMouseDown,
  onWheel,
}: TraceBoardProps) {
  return (
    <div
      className="trace-board"
      data-detail-level={detailLevel}
      id="trace-board"
      onMouseDown={onMouseDown}
      onWheel={onWheel}
      ref={boardRef}
      style={
        {
          ['--trace-scale' as string]: zoom.toFixed(3),
          ['--trace-column-gap' as string]: `${layout.columnGap.toFixed(2)}px`,
          ['--trace-row-gap' as string]: `${layout.rowGap.toFixed(2)}px`,
          ['--trace-pan-x' as string]: `${pan.x.toFixed(2)}px`,
          ['--trace-pan-y' as string]: `${pan.y.toFixed(2)}px`,
        } as React.CSSProperties
      }
    >
      {visibleNodeCount ? (
        <div className="trace-canvas" ref={canvasRef}>
          <svg aria-hidden="true" className="trace-overlay" id="trace-overlay">
            <path className="trace-overlay__glow" d={pathD} id="trace-overlay-glow" />
            <path className="trace-overlay__trench" d={pathD} id="trace-overlay-trench" />
            <path className="trace-overlay__line" d={pathD} id="trace-overlay-line" />
          </svg>
          {rows.map((row, rowIndex) => (
            <div className={`trace-row${rowIndex % 2 === 1 ? ' reverse' : ''}`} key={`row-${rowIndex}`}>
              {row.map((node, nodeIndex) => (
                <TraceNode
                  branchLabel={node.branch_id ? branchLabels[node.branch_id] || null : null}
                  detailLevel={detailLevel}
                  key={node.id}
                  node={node}
                  nodeIndex={nodeIndex}
                  rowIndex={rowIndex}
                  rowLength={row.length}
                  rowsLength={rows.length}
                  visibleNodeCount={visibleNodeCount}
                />
              ))}
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}
