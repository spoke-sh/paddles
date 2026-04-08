import {
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import {
  Link,
  Outlet,
  RouterProvider,
  createRootRoute,
  createRoute,
  createRouter,
  useRouterState,
} from '@tanstack/react-router';

import {
  FORCE_KIND_COLORS,
  FORCE_LEVEL_COLORS,
  KIND_COLORS,
  STEERING_GATE_COLORS,
  STEERING_GATE_ORDER,
  TRACE_DETAIL_LEVEL_LABELS,
  TRACE_VIEW_LABELS,
  TRACE_ZOOM_MAX,
  TRACE_ZOOM_MIN,
  formatTraceKind,
  manifoldAnchorLabel,
  resolverOutcomeMeta,
  resolverOutcomeNarrative,
  resolverOutcomeTitle,
  resolverSignalDetails,
  steeringGateClass,
  traceDetailLevelForZoom,
  traceLayoutForZoom,
  traceNodeDirection,
  traceNodeFamily,
  traceNodeVisible,
  truncate,
} from './runtime-helpers';
import { ChatComposer } from './chat/composer';
import {
  ManifoldTurnSelectionProvider,
  useManifoldTurnSelection,
} from './chat/manifold-turn-selection-context';
import { InspectorRoute } from './inspector/inspector-route';
import { ManifoldRoute } from './manifold/manifold-route';
import { TranscriptPane } from './chat/transcript-pane';
import { useChatComposer } from './chat/use-chat-composer';
import { useStickyTailScroll } from './chat/use-sticky-tail-scroll';
import { RuntimeStoreProvider, useRuntimeStore } from './runtime-store';
import type {
  ConversationTraceGraphNode,
  ManifoldFrame,
} from './runtime-types';

function activeViewForPath(pathname: string) {
  if (pathname === '/manifold') {
    return 'manifold';
  }
  if (pathname === '/transit') {
    return 'transit';
  }
  return 'inspector';
}

function RuntimeShellLayout() {
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const activeView = activeViewForPath(pathname);
  const { connected, error, events, projection, promptHistory, sending, sendTurn } =
    useRuntimeStore();
  const manifoldTurns = projection?.manifold.turns || [];
  const manifoldTurnIds = useMemo(
    () => new Set(manifoldTurns.map((turn) => turn.turn_id)),
    [manifoldTurns]
  );
  const [selectedManifoldTurnId, setSelectedManifoldTurnId] = useState<string | null>(null);
  const transcriptEntryCount = projection?.transcript.entries.length || 0;
  const { messagesRef, onMessagesScroll } = useStickyTailScroll({
    eventCount: events.length,
    transcriptEntryCount,
  });
  const { composerParts, onPromptKeyDown, onPromptPaste, onSubmit, prompt, setPrompt } =
    useChatComposer({
      promptHistory,
      onSubmitPrompt: sendTurn,
    });

  useEffect(() => {
    if (!manifoldTurns.length) {
      setSelectedManifoldTurnId(null);
      return;
    }
    if (
      !selectedManifoldTurnId ||
      !manifoldTurns.some((turn) => turn.turn_id === selectedManifoldTurnId)
    ) {
      setSelectedManifoldTurnId(manifoldTurns[manifoldTurns.length - 1].turn_id);
    }
  }, [manifoldTurns, selectedManifoldTurnId]);

  function selectManifoldTurnFromTranscript(turnId: string) {
    if (!manifoldTurnIds.has(turnId)) {
      return;
    }
    setSelectedManifoldTurnId(turnId);
  }

  return (
    <ManifoldTurnSelectionProvider
      value={{
        selectedTurnId: selectedManifoldTurnId,
        setSelectedTurnId: setSelectedManifoldTurnId,
      }}
    >
      <>
        <div className="chat-panel">
          <div className="chat-header">Paddles</div>
          <TranscriptPane
            activeView={activeView}
            connected={connected}
            error={error}
            events={events}
            manifoldTurnIds={manifoldTurnIds}
            messagesRef={messagesRef}
            onMessagesScroll={onMessagesScroll}
            onSelectManifoldTurn={selectManifoldTurnFromTranscript}
            projection={projection}
            selectedManifoldTurnId={selectedManifoldTurnId}
          />
          <ChatComposer
            composerParts={composerParts}
            onPromptChange={(event) => setPrompt(event.target.value)}
            onPromptKeyDown={onPromptKeyDown}
            onPromptPaste={onPromptPaste}
            onSubmit={onSubmit}
            prompt={prompt}
            sending={sending}
          />
        </div>

        <div className="trace-panel">
          <div className="trace-header-wrap">
            <div>
              <div className="trace-header">Transit Trace</div>
              <div className="trace-subhead" id="trace-subhead">
                {TRACE_VIEW_LABELS[activeView]}
              </div>
            </div>
            <div className="trace-tabs">
              <Link className={`trace-tab${activeView === 'inspector' ? ' is-active' : ''}`} to="/">
                Inspector
              </Link>
              <Link
                className={`trace-tab${activeView === 'manifold' ? ' is-active' : ''}`}
                to="/manifold"
              >
                Manifold
              </Link>
              <Link
                className={`trace-tab${activeView === 'transit' ? ' is-active' : ''}`}
                to="/transit"
              >
                Transit
              </Link>
            </div>
          </div>
          <Outlet />
        </div>
      </>
    </ManifoldTurnSelectionProvider>
  );
}

function TransitRoute() {
  const { projection } = useRuntimeStore();
  const graph = projection?.trace_graph || null;
  const boardRef = useRef<HTMLDivElement | null>(null);
  const canvasRef = useRef<HTMLDivElement | null>(null);
  const [boardWidth, setBoardWidth] = useState(960);
  const [zoom, setZoom] = useState(1);
  const [scope, setScope] = useState<'significant' | 'full'>('significant');
  const [familyVisibility, setFamilyVisibility] = useState<Record<string, boolean>>({
    model_io: false,
    lineage: false,
    signals: false,
    threads: false,
    tool_results: false,
  });
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [pathD, setPathD] = useState('');
  const panRef = useRef<{
    dragging: boolean;
    startX: number;
    startY: number;
    originX: number;
    originY: number;
  } | null>(null);

  useEffect(() => {
    const board = boardRef.current;
    if (!board || typeof ResizeObserver === 'undefined') {
      return;
    }
    const observer = new ResizeObserver(() => {
      setBoardWidth(board.clientWidth || 960);
    });
    observer.observe(board);
    setBoardWidth(board.clientWidth || 960);
    return () => observer.disconnect();
  }, []);

  const sortedNodes = useMemo(
    () => [...(graph?.nodes || [])].sort((left, right) => left.sequence - right.sequence),
    [graph?.nodes]
  );

  const visibleNodes = useMemo(() => {
    let nodes = sortedNodes.filter((node) => traceNodeVisible(node, scope, familyVisibility));
    if (!nodes.length && scope === 'significant') {
      nodes = sortedNodes.slice();
    }
    return nodes;
  }, [familyVisibility, scope, sortedNodes]);

  const detailLevel = traceDetailLevelForZoom(zoom);
  const layout = traceLayoutForZoom(zoom);
  const effectiveScope =
    scope === 'significant' && !sortedNodes.filter((node) => traceNodeVisible(node, scope, familyVisibility)).length
      ? 'full'
      : scope;
  const columns = Math.max(
    2,
    Math.floor((Math.max(boardWidth, 280) + layout.columnGap) / (layout.tileMin + layout.columnGap))
  );
  const rows: ConversationTraceGraphNode[][] = [];
  for (let index = 0; index < visibleNodes.length; index += columns) {
    rows.push(visibleNodes.slice(index, index + columns));
  }
  const branchLabels = Object.fromEntries((graph?.branches || []).map((branch) => [branch.id, branch.label]));

  useEffect(() => {
    const board = boardRef.current;
    const canvas = canvasRef.current;
    if (!board || !canvas) {
      return;
    }
    const nodes = Array.from(canvas.querySelectorAll<HTMLElement>('.trace-node'));
    if (nodes.length < 2) {
      setPathD('');
      return;
    }
    const canvasRect = canvas.getBoundingClientRect();
    const points = nodes.map((node) => {
      const rect = node.getBoundingClientRect();
      return {
        x: rect.left - canvasRect.left + rect.width / 2,
        y: rect.top - canvasRect.top + rect.height / 2,
      };
    });
    let nextPath = `M ${points[0].x} ${points[0].y}`;
    for (let index = 1; index < points.length; index += 1) {
      const prev = points[index - 1];
      const next = points[index];
      const midX = (prev.x + next.x) / 2;
      nextPath += ` C ${midX} ${prev.y}, ${midX} ${next.y}, ${next.x} ${next.y}`;
    }
    setPathD(nextPath);
  }, [detailLevel, rows, zoom]);

  useEffect(() => {
    function handleMouseMove(event: MouseEvent) {
      if (!panRef.current?.dragging) {
        return;
      }
      setPan({
        x: panRef.current.originX + (event.clientX - panRef.current.startX),
        y: panRef.current.originY + (event.clientY - panRef.current.startY),
      });
    }
    function handleMouseUp() {
      panRef.current = null;
    }
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    window.addEventListener('blur', handleMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
      window.removeEventListener('blur', handleMouseUp);
    };
  }, []);

  function onWheel(event: React.WheelEvent<HTMLDivElement>) {
    if (!visibleNodes.length) {
      return;
    }
    event.preventDefault();
    const nextZoom = Math.max(
      TRACE_ZOOM_MIN,
      Math.min(TRACE_ZOOM_MAX, zoom * Math.exp(-event.deltaY * 0.0015))
    );
    setZoom(nextZoom);
  }

  return (
    <div className="trace-view trace-view--active trace-view--transit" id="transit-view">
      <div className="trace-transit-toolbar" id="trace-transit-toolbar">
        <div className="trace-transit-toggle-row">
          <div className="trace-transit-toggle-group">
            <button
              className={`trace-transit-toggle${scope === 'significant' ? ' is-active' : ''}`}
              data-trace-scope="significant"
              onClick={() => setScope('significant')}
              type="button"
            >
              Significant
            </button>
            <button
              className={`trace-transit-toggle${effectiveScope === 'full' ? ' is-active' : ''}`}
              data-trace-scope="full"
              onClick={() => setScope('full')}
              type="button"
            >
              Full Trace
            </button>
          </div>
          <div className="trace-transit-toggle-group">
            {[
              ['model_io', 'Model I/O'],
              ['lineage', 'Lineage'],
              ['signals', 'Signals'],
              ['threads', 'Threads'],
              ['tool_results', 'Tool Done'],
            ].map(([family, label]) => (
              <button
                className={`trace-transit-toggle${familyVisibility[family] ? ' is-active' : ''}`}
                data-trace-family={family}
                key={family}
                onClick={() =>
                  setFamilyVisibility((current) => ({
                    ...current,
                    [family]: !current[family],
                  }))
                }
                type="button"
              >
                {label}
              </button>
            ))}
          </div>
        </div>
        <div className="trace-transit-meta" id="trace-transit-meta">
          {visibleNodes.length
            ? `Showing ${visibleNodes.length} of ${sortedNodes.length} steps · ${
                effectiveScope === 'full' ? 'full trace' : 'significant steps'
              } · ${TRACE_DETAIL_LEVEL_LABELS[detailLevel] || detailLevel}`
            : `Showing ${effectiveScope === 'full' ? 'full trace' : 'significant steps'} · ${
                TRACE_DETAIL_LEVEL_LABELS[detailLevel] || detailLevel
              }`}
        </div>
      </div>
      <div
        className="trace-board"
        data-detail-level={detailLevel}
        id="trace-board"
        onMouseDown={(event) => {
          if (event.button !== 0) {
            return;
          }
          panRef.current = {
            dragging: true,
            startX: event.clientX,
            startY: event.clientY,
            originX: pan.x,
            originY: pan.y,
          };
        }}
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
        {visibleNodes.length ? (
          <div className="trace-canvas" ref={canvasRef}>
            <svg aria-hidden="true" className="trace-overlay" id="trace-overlay">
              <path className="trace-overlay__glow" d={pathD} id="trace-overlay-glow" />
              <path className="trace-overlay__trench" d={pathD} id="trace-overlay-trench" />
              <path className="trace-overlay__line" d={pathD} id="trace-overlay-line" />
            </svg>
            {rows.map((row, rowIndex) => (
              <div className={`trace-row${rowIndex % 2 === 1 ? ' reverse' : ''}`} key={`row-${rowIndex}`}>
                {row.map((node, nodeIndex) => {
                  const recencyIndex =
                    visibleNodes.length - (rowIndex * columns + nodeIndex) - 1;
                  const recencyRatio =
                    visibleNodes.length <= 1
                      ? 1
                      : 1 - recencyIndex / Math.max(visibleNodes.length - 1, 1);
                  const depth = Math.max(0, Math.min(1, recencyRatio));
                  const sunTrailDepth = Math.max(0, 1 - recencyIndex / 3);
                  const direction = traceNodeDirection(
                    rowIndex,
                    nodeIndex,
                    row.length,
                    rows.length
                  );
                  const branchLabel = node.branch_id ? branchLabels[node.branch_id] : null;
                  const summary =
                    detailLevel === 'overview'
                      ? node.id
                      : branchLabel || formatTraceKind(node.kind);

                  return (
                    <div
                      className={`trace-node${
                        recencyIndex === 0 ? ' trace-node--latest' : ''
                      } trace-node--${direction}`}
                      key={node.id}
                      style={
                        {
                          ['--node-color' as string]: KIND_COLORS[node.kind] || '#8b949e',
                          ['--node-raise' as string]: `${(-(2 + depth * 6)).toFixed(2)}px`,
                          ['--node-shadow-x' as string]: `${(5 + depth * 2.5).toFixed(2)}px`,
                          ['--node-shadow-y' as string]: `${(6 + depth * 4).toFixed(2)}px`,
                          ['--node-shadow-blur' as string]: `${(12 + depth * 7).toFixed(2)}px`,
                          ['--node-shadow-alpha' as string]: `${(0.1 + depth * 0.12).toFixed(3)}`,
                          ['--node-shadow-warm-alpha' as string]: `${(
                            0.02 +
                            sunTrailDepth * 0.1
                          ).toFixed(3)}`,
                          ['--node-shadow-warm-x' as string]: `${(
                            1 +
                            sunTrailDepth * 2
                          ).toFixed(2)}px`,
                          ['--node-shadow-warm-y' as string]: `${(
                            2 +
                            sunTrailDepth * 4
                          ).toFixed(2)}px`,
                          ['--node-shadow-warm-blur' as string]: `${(
                            4 +
                            sunTrailDepth * 10
                          ).toFixed(2)}px`,
                          ['--node-tilt-x' as string]: `${direction === 'down' ? 8 : 5}deg`,
                          ['--node-tilt-y' as string]: `${
                            direction === 'ltr' ? -4 : direction === 'rtl' ? 4 : 0
                          }deg`,
                          ['--node-specular-alpha' as string]: `${(0.68 + depth * 0.2).toFixed(3)}`,
                        } as React.CSSProperties
                      }
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
                })}
              </div>
            ))}
          </div>
        ) : null}
      </div>
      <div className="trace-empty" id="trace-empty" style={{ display: visibleNodes.length ? 'none' : 'block' }}>
        {sortedNodes.length
          ? 'Current transit toggles hide every step. Re-enable a family or switch to full trace.'
          : 'Submit a prompt to see the trace railroad.'}
      </div>
    </div>
  );
}

const rootRoute = createRootRoute({
  component: RuntimeShellLayout,
});

const inspectorRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: InspectorRoute,
});

const transitRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/transit',
  component: TransitRoute,
});

const manifoldRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/manifold',
  component: ManifoldRoute,
});

const routeTree = rootRoute.addChildren([inspectorRoute, transitRoute, manifoldRoute]);

export function buildRuntimeRouter() {
  return createRouter({
    routeTree,
  });
}

declare module '@tanstack/react-router' {
  interface Register {
    router: ReturnType<typeof buildRuntimeRouter>;
  }
}

export function RuntimeApp() {
  const [router] = useState(() => buildRuntimeRouter());

  return (
    <RuntimeStoreProvider>
      <div className="runtime-shell-host">
        <RouterProvider router={router} />
      </div>
    </RuntimeStoreProvider>
  );
}
