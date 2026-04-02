import { BrowserRouter, Link, Route, Routes } from 'react-router-dom';

function RouteFrame({
  title,
  summary,
  testId,
  legacyPath,
}: {
  title: string;
  summary: string;
  testId: string;
  legacyPath: string;
}) {
  return (
    <section className="route-panel" data-testid={testId}>
      <p className="eyebrow">React Runtime Shell</p>
      <h2>{title}</h2>
      <p>{summary}</p>
      <div className="status-callout">
        <strong>Runtime composition:</strong> The Rust server now serves this React shell on the
        primary routes while the legacy live runtime remains mounted under <code>{legacyPath}</code>{' '}
        until each surface is replaced route-by-route.
      </div>
      <div className="runtime-frame-shell">
        <iframe
          className="runtime-frame"
          src={legacyPath}
          title={title}
          loading="lazy"
          referrerPolicy="same-origin"
        />
      </div>
    </section>
  );
}

const runtimeRoutes = [
  {
    path: '/',
    navLabel: 'Chat',
    testId: 'route-chat',
    title: 'Conversation Route Shell',
    summary:
      'This React surface will absorb the transcript and operator controls while preserving the Rust session APIs.',
    legacyPath: '/legacy',
  },
  {
    path: '/transit',
    navLabel: 'Transit',
    testId: 'route-transit',
    title: 'Transit Route Shell',
    summary: 'This React surface will absorb the turn-step map, zoom controls, and lineage filters.',
    legacyPath: '/legacy/transit',
  },
  {
    path: '/manifold',
    navLabel: 'Manifold',
    testId: 'route-manifold',
    title: 'Manifold Route Shell',
    summary:
      'This React surface will absorb the steering-signal manifold and its route-linked forensic drilldown.',
    legacyPath: '/legacy/manifold',
  },
] as const;

export function RuntimeShell() {
  return (
    <div className="app-shell">
      <header className="app-header">
        <div>
          <p className="eyebrow">Paddles</p>
          <h1>Turborepo Runtime Web App</h1>
        </div>
        <nav className="app-nav" aria-label="Primary">
          {runtimeRoutes.map((route) => (
            <Link key={route.path} to={route.path}>
              {route.navLabel}
            </Link>
          ))}
        </nav>
      </header>
      <main>
        <Routes>
          {runtimeRoutes.map((route) => (
            <Route
              key={route.path}
              path={route.path}
              element={
                <RouteFrame
                  testId={route.testId}
                  title={route.title}
                  summary={route.summary}
                  legacyPath={route.legacyPath}
                />
              }
            />
          ))}
        </Routes>
      </main>
    </div>
  );
}

export function RuntimeApp() {
  return (
    <BrowserRouter>
      <RuntimeShell />
    </BrowserRouter>
  );
}
