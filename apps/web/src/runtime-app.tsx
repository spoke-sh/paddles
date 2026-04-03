import { useEffect, useRef, useState } from 'react';
import {
  RouterProvider,
  createRootRoute,
  createRoute,
  createRouter,
} from '@tanstack/react-router';

import runtimeShellHtml from '../../../src/infrastructure/web/index.html?raw';

declare global {
  interface Window {
    __PADDLES_DISABLE_RUNTIME_BOOTSTRAP__?: boolean;
  }
}

const STYLE_ID = 'paddles-runtime-shell-style';

function extractHtmlSegment(pattern: RegExp, label: string) {
  const match = runtimeShellHtml.match(pattern);
  if (!match) {
    throw new Error(`Unable to extract ${label} from runtime shell source`);
  }
  return match[1];
}

const runtimeShellCss = extractHtmlSegment(/<style>([\s\S]*?)<\/style>/i, 'style block').replace(
  /\bbody\b/g,
  '.runtime-shell-host'
);
const runtimeShellBody = extractHtmlSegment(/<body[^>]*>([\s\S]*?)<\/body>/i, 'body markup');
const runtimeShellScript = extractHtmlSegment(
  /<script>([\s\S]*?)<\/script>\s*<\/body>/i,
  'runtime bootstrap script'
);

function ensureRuntimeShellStyles() {
  if (document.getElementById(STYLE_ID)) {
    return;
  }

  const style = document.createElement('style');
  style.id = STYLE_ID;
  style.textContent = runtimeShellCss;
  document.head.appendChild(style);
}

function RuntimeShellBridge() {
  const hostRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const host = hostRef.current;
    if (!host || host.dataset.runtimeMounted === 'true') {
      return;
    }

    ensureRuntimeShellStyles();
    host.innerHTML = runtimeShellBody;
    host.dataset.runtimeMounted = 'true';

    if (window.__PADDLES_DISABLE_RUNTIME_BOOTSTRAP__) {
      return;
    }

    const script = document.createElement('script');
    script.type = 'text/javascript';
    script.textContent = runtimeShellScript;
    host.appendChild(script);

    return () => {
      script.remove();
      host.innerHTML = '';
      delete host.dataset.runtimeMounted;
    };
  }, []);

  return <div className="runtime-shell-host" data-testid="runtime-root" ref={hostRef} />;
}

const rootRoute = createRootRoute({
  component: RuntimeShellBridge,
});

const conversationRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: () => null,
});

const transitRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/transit',
  component: () => null,
});

const manifoldRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/manifold',
  component: () => null,
});

const routeTree = rootRoute.addChildren([conversationRoute, transitRoute, manifoldRoute]);

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
  return <RouterProvider router={router} />;
}
