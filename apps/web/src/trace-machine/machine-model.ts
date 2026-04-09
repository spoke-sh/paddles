export type MachineMomentKind =
  | 'input'
  | 'planner'
  | 'evidence_probe'
  | 'diverter'
  | 'jam'
  | 'spring_return'
  | 'tool_run'
  | 'force'
  | 'output'
  | 'unknown';

export interface MachineMomentLexiconEntry {
  label: string;
  narrative: string;
  replaces: string[];
}

export interface MachineSelectionState {
  selectedTurnId: string | null;
  selectedMomentId: string | null;
  showInternals: boolean;
}

export const MACHINE_NARRATIVE_KINDS: MachineMomentKind[] = [
  'input',
  'planner',
  'evidence_probe',
  'diverter',
  'jam',
  'spring_return',
  'tool_run',
  'force',
  'output',
  'unknown',
];

export const MACHINE_SELECTION_FIELDS = [
  'selectedTurnId',
  'selectedMomentId',
  'showInternals',
] as const;

export const MACHINE_MOMENT_LEXICON: Record<MachineMomentKind, MachineMomentLexiconEntry> = {
  input: {
    label: 'Input',
    narrative: 'Where the turn enters the machine and the request starts rolling.',
    replaces: ['task root', 'turn start', 'prompt artifact'],
  },
  planner: {
    label: 'Planner',
    narrative: 'Where the machine chooses the next bounded move.',
    replaces: ['planner step', 'planner action'],
  },
  evidence_probe: {
    label: 'Evidence probe',
    narrative: 'Where the machine inspects the workspace or gathers proof before it commits.',
    replaces: ['search result', 'read step', 'inspect record'],
  },
  diverter: {
    label: 'Diverter',
    narrative: 'Where the machine changes direction because the current line needs a different path.',
    replaces: ['branch', 'route change', 'action bias redirect'],
  },
  jam: {
    label: 'Jam',
    narrative: 'Where progress catches and the machine has to explain or recover before continuing.',
    replaces: ['fallback', 'stop reason', 'blocked step'],
  },
  spring_return: {
    label: 'Spring return',
    narrative: 'Where the machine replans and snaps back into a better path without restarting the turn.',
    replaces: ['planner retry', 'replan', 'recursive continuation'],
  },
  tool_run: {
    label: 'Tool run',
    narrative: 'Where the machine acts on the workspace and produces concrete local output.',
    replaces: ['tool call', 'shell result', 'workspace edit action'],
  },
  force: {
    label: 'Force',
    narrative: 'Where steering pressure pushes the machine toward evidence, convergence, or containment.',
    replaces: ['signal snapshot', 'steering signal', 'gate contribution'],
  },
  output: {
    label: 'Output',
    narrative: 'Where the machine lands the answer, edit, commit, or explicit block at the end of the run.',
    replaces: ['completion checkpoint', 'assistant response', 'final artifact'],
  },
  unknown: {
    label: 'Unknown part',
    narrative: 'Where the machine keeps the source ids visible while the narrative mapping catches up.',
    replaces: ['unmapped trace data'],
  },
};

export const DEFAULT_MACHINE_SELECTION: MachineSelectionState = {
  selectedTurnId: null,
  selectedMomentId: null,
  showInternals: false,
};

export function machineMomentKinds() {
  return MACHINE_NARRATIVE_KINDS.slice();
}

export function machineMomentEntry(kind: MachineMomentKind) {
  return MACHINE_MOMENT_LEXICON[kind];
}

export function machineMomentLabel(kind: MachineMomentKind) {
  return machineMomentEntry(kind).label;
}
