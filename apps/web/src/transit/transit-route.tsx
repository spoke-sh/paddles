import { useRuntimeStore } from '../runtime-store';
import { TransitToolbar } from './transit-toolbar';
import { TraceBoard } from './trace-board';
import { useTraceBoard } from './use-trace-board';

export function TransitRoute() {
  const { projection } = useRuntimeStore();
  const graph = projection?.trace_graph || null;
  const traceBoard = useTraceBoard(graph);

  return (
    <div className="trace-view trace-view--active trace-view--transit" id="transit-view">
      <TransitToolbar
        detailLevel={traceBoard.detailLevel}
        effectiveScope={traceBoard.effectiveScope}
        familyVisibility={traceBoard.familyVisibility}
        sortedNodeCount={traceBoard.sortedNodes.length}
        visibleNodeCount={traceBoard.visibleNodes.length}
        onSetScope={traceBoard.setScope}
        onToggleFamily={traceBoard.toggleFamily}
        scope={traceBoard.scope}
      />
      <TraceBoard
        boardRef={traceBoard.boardRef}
        branchLabels={traceBoard.branchLabels}
        canvasRef={traceBoard.canvasRef}
        detailLevel={traceBoard.detailLevel}
        layout={traceBoard.layout}
        pan={traceBoard.pan}
        pathD={traceBoard.pathD}
        rows={traceBoard.rows}
        visibleNodeCount={traceBoard.visibleNodes.length}
        zoom={traceBoard.zoom}
        onMouseDown={traceBoard.onBoardMouseDown}
        onWheel={traceBoard.onBoardWheel}
      />
      <div
        className="trace-empty"
        id="trace-empty"
        style={{ display: traceBoard.visibleNodes.length ? 'none' : 'block' }}
      >
        {traceBoard.sortedNodes.length
          ? 'Current transit toggles hide every step. Re-enable a family or switch to full trace.'
          : 'Submit a prompt to see the trace railroad.'}
      </div>
    </div>
  );
}
