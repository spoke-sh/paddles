import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { RuntimeApp } from './runtime-app';

afterEach(() => {
  cleanup();
});

function renderAtPath(pathname: string) {
  window.history.pushState({}, '', pathname);
  return render(<RuntimeApp />);
}

describe('RuntimeApp', () => {
  it('renders the primary conversation route without an iframe proxy', async () => {
    renderAtPath('/');

    expect(await screen.findByTestId('runtime-root')).toBeInTheDocument();
    expect(document.getElementById('prompt')).toBeInTheDocument();
    expect(document.getElementById('trace-board')).toBeInTheDocument();
    expect(screen.queryByTitle('Paddles Runtime')).not.toBeInTheDocument();
  });

  it('renders the primary transit route through the client router', async () => {
    renderAtPath('/transit');

    expect(await screen.findByTestId('runtime-root')).toBeInTheDocument();
    expect(document.getElementById('prompt')).toBeInTheDocument();
    expect(document.getElementById('trace-board')).toBeInTheDocument();
    expect(screen.queryByTitle('Paddles Runtime')).not.toBeInTheDocument();
  });

  it('renders the primary manifold route through the client router', async () => {
    renderAtPath('/manifold');

    expect(await screen.findByTestId('runtime-root')).toBeInTheDocument();
    expect(document.getElementById('prompt')).toBeInTheDocument();
    expect(document.getElementById('manifold-canvas')).toBeInTheDocument();
    expect(screen.queryByTitle('Paddles Runtime')).not.toBeInTheDocument();
  });
});
