import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { RuntimeShell } from './runtime-app';

afterEach(() => {
  cleanup();
});

describe('RuntimeShell', () => {
  it('renders no outer chrome and points the root route at the legacy runtime', () => {
    render(<RuntimeShell pathname="/" />);

    expect(screen.queryByRole('heading', { name: 'Turborepo Runtime Web App' })).not.toBeInTheDocument();
    expect(screen.queryByRole('navigation')).not.toBeInTheDocument();
    expect(screen.getByTestId('runtime-root')).toBeInTheDocument();
    expect(screen.getByTitle('Paddles Runtime')).toHaveAttribute('src', '/legacy');
  });

  it('routes the transit path to the legacy transit surface', () => {
    render(<RuntimeShell pathname="/transit" />);

    expect(screen.getByTitle('Paddles Runtime')).toHaveAttribute('src', '/legacy/transit');
  });

  it('routes the manifold path to the legacy manifold surface', () => {
    render(<RuntimeShell pathname="/manifold" />);

    expect(screen.getByTitle('Paddles Runtime')).toHaveAttribute('src', '/legacy/manifold');
  });
});
