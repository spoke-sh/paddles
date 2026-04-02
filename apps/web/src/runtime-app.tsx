import { BrowserRouter, Link, Route, Routes } from 'react-router-dom';

function RouteFrame({
  title,
  summary,
  testId,
}: {
  title: string;
  summary: string;
  testId: string;
}) {
  return (
    <section className="route-panel" data-testid={testId}>
      <p className="eyebrow">React Runtime Shell</p>
      <h2>{title}</h2>
      <p>{summary}</p>
      <div className="status-callout">
        <strong>Migration state:</strong> The Rust-embedded web shell still owns live runtime
        behavior while this React app grows route-by-route toward cutover.
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
  },
  {
    path: '/transit',
    navLabel: 'Transit',
    testId: 'route-transit',
    title: 'Transit Route Shell',
    summary: 'This React surface will absorb the turn-step map, zoom controls, and lineage filters.',
  },
  {
    path: '/manifold',
    navLabel: 'Manifold',
    testId: 'route-manifold',
    title: 'Manifold Route Shell',
    summary:
      'This React surface will absorb the steering-signal manifold and its route-linked forensic drilldown.',
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
