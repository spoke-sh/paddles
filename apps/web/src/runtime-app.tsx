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
  formatTraceKind,
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
import { TransitRoute } from './transit/transit-route';
import { useChatComposer } from './chat/use-chat-composer';
import { useStickyTailScroll } from './chat/use-sticky-tail-scroll';
import { RuntimeStoreProvider, useRuntimeStore } from './runtime-store';
import type { ManifoldFrame } from './runtime-types';

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
