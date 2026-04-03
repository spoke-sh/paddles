function normalizePath(pathname: string) {
  if (pathname.length > 1 && pathname.endsWith('/')) {
    return pathname.slice(0, -1);
  }
  return pathname || '/';
}

function legacyPathForPrimaryRoute(pathname: string) {
  const path = normalizePath(pathname);
  if (path === '/transit') {
    return '/legacy/transit';
  }
  if (path === '/manifold') {
    return '/legacy/manifold';
  }
  return '/legacy';
}

export function RuntimeShell({ pathname }: { pathname?: string }) {
  const currentPath =
    pathname ?? (typeof window !== 'undefined' ? window.location.pathname : '/');
  const legacyPath = legacyPathForPrimaryRoute(currentPath);

  return (
    <div className="runtime-root" data-testid="runtime-root">
      <iframe
        className="runtime-iframe"
        src={legacyPath}
        title="Paddles Runtime"
        loading="lazy"
        referrerPolicy="same-origin"
      />
    </div>
  );
}

export function RuntimeApp() {
  return <RuntimeShell />;
}
