import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it } from 'vitest';

import { RuntimeShell } from './runtime-app';

describe('RuntimeShell', () => {
  it('renders the conversation route shell by default', () => {
    render(
      <MemoryRouter initialEntries={['/']}>
        <RuntimeShell />
      </MemoryRouter>
    );

    expect(screen.getByTestId('route-chat')).toHaveTextContent('Conversation Route Shell');
    expect(screen.getByText('Migration state:')).toBeInTheDocument();
  });

  it('renders the transit route shell', () => {
    render(
      <MemoryRouter initialEntries={['/transit']}>
        <RuntimeShell />
      </MemoryRouter>
    );

    expect(screen.getByTestId('route-transit')).toHaveTextContent('Transit Route Shell');
  });

  it('renders the manifold route shell', () => {
    render(
      <MemoryRouter initialEntries={['/manifold']}>
        <RuntimeShell />
      </MemoryRouter>
    );

    expect(screen.getByTestId('route-manifold')).toHaveTextContent('Manifold Route Shell');
  });
});
