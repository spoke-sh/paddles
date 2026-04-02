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

export function RuntimeShell() {
  return (
    <div className="app-shell">
      <header className="app-header">
        <div>
          <p className="eyebrow">Paddles</p>
          <h1>Turborepo Runtime Web App</h1>
        </div>
        <nav className="app-nav" aria-label="Primary">
          <Link to="/">Chat</Link>
          <Link to="/transit">Transit</Link>
          <Link to="/manifold">Manifold</Link>
        </nav>
      </header>
      <main>
        <Routes>
          <Route
            path="/"
            element={
              <RouteFrame
                testId="route-chat"
                title="Conversation Route Shell"
                summary="This React surface will absorb the transcript and operator controls while preserving the Rust session APIs."
              />
            }
          />
          <Route
            path="/transit"
            element={
              <RouteFrame
                testId="route-transit"
                title="Transit Route Shell"
                summary="This React surface will absorb the turn-step map, zoom controls, and lineage filters."
              />
            }
          />
          <Route
            path="/manifold"
            element={
              <RouteFrame
                testId="route-manifold"
                title="Manifold Route Shell"
                summary="This React surface will absorb the steering-signal manifold and its route-linked forensic drilldown."
              />
            }
          />
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
