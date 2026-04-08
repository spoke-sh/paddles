import { createContext, useContext } from 'react';
import type { ReactNode } from 'react';

interface ManifoldTurnSelectionValue {
  selectedTurnId: string | null;
  setSelectedTurnId: (turnId: string | null) => void;
}

const ManifoldTurnSelectionContext = createContext<ManifoldTurnSelectionValue | null>(null);

export function ManifoldTurnSelectionProvider({
  children,
  value,
}: {
  children: ReactNode;
  value: ManifoldTurnSelectionValue;
}) {
  return (
    <ManifoldTurnSelectionContext.Provider value={value}>
      {children}
    </ManifoldTurnSelectionContext.Provider>
  );
}

export function useManifoldTurnSelection() {
  const value = useContext(ManifoldTurnSelectionContext);
  if (!value) {
    throw new Error('useManifoldTurnSelection must be used inside RuntimeShellLayout');
  }
  return value;
}
