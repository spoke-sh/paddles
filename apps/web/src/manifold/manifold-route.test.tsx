import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { RuntimeApp } from '../runtime-app';
import type { ConversationProjectionSnapshot } from '../runtime-types';
import {
  bootstrapProjection,
  installRuntimeHarness,
  renderAtPath,
  resetRuntimeHarness,
  stubRuntimeFetch,
} from '../test-support/runtime-harness';

beforeEach(() => {
  installRuntimeHarness();
});

afterEach(() => {
  resetRuntimeHarness();
});

describe('ManifoldRoute', () => {
  it('renders the primary manifold route through the client router', async () => {
    renderAtPath('/manifold');

    expect(
      await screen.findByText('Steering Gate Manifold', { selector: '#trace-subhead' })
    ).toBeInTheDocument();
    expect(document.getElementById('manifold-canvas')).toBeInTheDocument();
    expect(document.querySelectorAll('.manifold-force-point').length).toBeGreaterThan(0);
    expect(screen.getByText('Evidence gate')).toBeInTheDocument();
    expect(screen.getByText('Convergence gate')).toBeInTheDocument();
    expect(screen.getByText('Containment gate')).toBeInTheDocument();
    expect(document.querySelector('.manifold-readout')).toBeNull();
    expect(screen.queryByText('Timeline')).not.toBeInTheDocument();
    expect(screen.queryByText('Gate Sources')).not.toBeInTheDocument();
    expect(screen.queryByText('Gate family')).not.toBeInTheDocument();
    expect(screen.queryByTitle('Paddles Runtime')).not.toBeInTheDocument();
  });

  it('renders the temporal gate field instead of the empty-state filler once frames exist', async () => {
    renderAtPath('/manifold');

    expect(screen.queryByText('Temporal gate playback is active.')).not.toBeInTheDocument();
    expect(document.querySelector('.manifold-playback-banner')).toBeNull();
    expect(document.querySelector('.manifold-empty-state')).toBeNull();
    expect(await screen.findByText('Temporal gate field')).toBeInTheDocument();
    expect(
      screen.queryByText(/task-123\.turn-0001 · anchor replace runtime shell padding/i)
    ).not.toBeInTheDocument();
    expect(screen.getByText(/Drag to tilt · Alt\+drag to rotate/i)).toBeInTheDocument();
    expect(document.querySelector('.manifold-spacefield__hint')).toBeNull();
    expect(document.querySelector('.manifold-stage-timeline')).toBeNull();
    expect(document.querySelector('.manifold-spacefield__scrubber')).not.toBeNull();
    expect(document.getElementById('manifold-time-scrubber')).not.toBeNull();
    expect(document.querySelectorAll('.manifold-spacefield__scrubber-frame')).toHaveLength(1);
    expect(document.querySelector('.manifold-force-point__label')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Replay' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Reset View' })).toBeNull();
    const scrubber = document.querySelector('.manifold-spacefield__scrubber');
    expect(scrubber?.querySelector('#manifold-play-toggle')).not.toBeNull();
    expect(document.querySelector('.manifold-stage-head #manifold-play-toggle')).toBeNull();
  });

  it('scrubs manifold frames from the bottom filmstrip slider', async () => {
    const multiFrameProjection: ConversationProjectionSnapshot = {
      ...bootstrapProjection,
      manifold: {
        ...bootstrapProjection.manifold,
        turns: [
          {
            ...bootstrapProjection.manifold.turns[0],
            frames: [
              ...bootstrapProjection.manifold.turns[0].frames,
              {
                ...bootstrapProjection.manifold.turns[0].frames[0],
                record_id: 'record-3',
                sequence: 3,
                anchor: {
                  id: 'planner-step:record-3',
                  kind: 'planner_step',
                  label: 'apply focused manifold edit',
                },
                active_signals: [
                  {
                    ...bootstrapProjection.manifold.turns[0].frames[0].active_signals[0],
                    snapshot_record_id: 'record-3',
                    summary: 'Containment held the selected manifold target steady.',
                    kind: 'containment_pressure',
                    gate: 'containment',
                    phase: 'holding',
                    magnitude_percent: 74,
                    level: 'high',
                    anchor: {
                      id: 'planner-step:record-3',
                      kind: 'planner_step',
                      label: 'apply focused manifold edit',
                    },
                  },
                ],
                gates: [
                  {
                    ...bootstrapProjection.manifold.turns[0].frames[0].gates[0],
                    gate: 'containment',
                    label: 'containment gate',
                    dominant_signal_kind: 'containment_pressure',
                    signal_kinds: ['containment_pressure'],
                    dominant_record_id: 'record-3',
                    phase: 'holding',
                    magnitude_percent: 74,
                    level: 'high',
                    anchor: {
                      id: 'planner-step:record-3',
                      kind: 'planner_step',
                      label: 'apply focused manifold edit',
                    },
                  },
                ],
              },
            ],
          },
        ],
      },
    };

    stubRuntimeFetch({ projection: multiFrameProjection });
    renderAtPath('/manifold');

    expect(await screen.findByText('Temporal gate field')).toBeInTheDocument();
    expect(document.querySelectorAll('.manifold-spacefield__scrubber-frame')).toHaveLength(2);

    const secondFrame = await screen.findByRole('button', {
      name: /Frame 2: apply focused manifold edit/i,
    });
    fireEvent.click(secondFrame);

    expect(await screen.findByText('Containment held the selected manifold target steady.')).toBeInTheDocument();
    expect(document.getElementById('manifold-frame-meta')).toHaveTextContent('Frame 2 / 2');
  });

  it('selects manifold turns from transcript messages instead of an in-stage dropdown', async () => {
    const multiTurnProjection: ConversationProjectionSnapshot = {
      ...bootstrapProjection,
      transcript: {
        ...bootstrapProjection.transcript,
        entries: [
          ...bootstrapProjection.transcript.entries,
          {
            record_id: 'record-3',
            turn_id: 'task-123.turn-0002',
            speaker: 'user',
            content: 'Please tune the steering gate manifold copy.',
          },
          {
            record_id: 'record-4',
            turn_id: 'task-123.turn-0002',
            speaker: 'assistant',
            content: '**Update**\n\nThe manifold now carries a second turn for selection tests.',
            response_mode: 'grounded_answer',
            render: {
              blocks: [
                { type: 'heading', text: 'Update' },
                {
                  type: 'paragraph',
                  text: 'The manifold now carries a second turn for selection tests.',
                },
              ],
            },
          },
        ],
      },
      forensics: {
        ...bootstrapProjection.forensics,
        turns: [
          ...bootstrapProjection.forensics.turns,
          {
            turn_id: 'task-123.turn-0002',
            lifecycle: 'final',
            records: [],
          },
        ],
      },
      manifold: {
        ...bootstrapProjection.manifold,
        turns: [
          ...bootstrapProjection.manifold.turns,
          {
            turn_id: 'task-123.turn-0002',
            lifecycle: 'final',
            frames: [
              {
                record_id: 'record-4',
                sequence: 4,
                lifecycle: 'final',
                anchor: {
                  id: 'planner-step:record-4',
                  kind: 'planner_step',
                  label: 'retune manifold narration',
                },
                active_signals: [
                  {
                    snapshot_record_id: 'record-4',
                    lifecycle: 'final',
                    kind: 'containment_pressure',
                    gate: 'containment',
                    phase: 'holding',
                    summary: 'Containment held the authored target steady.',
                    level: 'high',
                    magnitude_percent: 71,
                    anchor: {
                      id: 'planner-step:record-4',
                      kind: 'planner_step',
                      label: 'retune manifold narration',
                    },
                    contributions: [
                      {
                        source: 'authored_path_boundary',
                        share_percent: 71,
                        rationale: 'The authored workspace boundary kept the turn on target.',
                      },
                    ],
                    artifact: {
                      summary: 'signal snapshot',
                      inline_content: '{"kind":"containment_pressure"}',
                      mime_type: 'application/json',
                    },
                  },
                ],
                gates: [
                  {
                    gate: 'containment',
                    label: 'containment gate',
                    phase: 'holding',
                    level: 'high',
                    magnitude_percent: 71,
                    anchor: {
                      id: 'planner-step:record-4',
                      kind: 'planner_step',
                      label: 'retune manifold narration',
                    },
                    dominant_signal_kind: 'containment_pressure',
                    signal_kinds: ['containment_pressure'],
                    dominant_record_id: 'record-4',
                  },
                ],
                primitives: [
                  {
                    primitive_id: 'gate:containment',
                    kind: 'shield',
                    label: 'Containment gate',
                    basis: { kind: 'steering_gate', gate: 'containment' },
                    evidence_record_id: 'record-4',
                    anchor: {
                      id: 'planner-step:record-4',
                      kind: 'planner_step',
                      label: 'retune manifold narration',
                    },
                    level: 'high',
                    magnitude_percent: 71,
                  },
                ],
                conduits: [],
              },
            ],
          },
        ],
      },
    };

    stubRuntimeFetch({ projection: multiTurnProjection });
    renderAtPath('/manifold');

    expect(document.getElementById('manifold-turn-select')).toBeNull();
    expect(
      await screen.findByText('Containment held the authored target steady.')
    ).toBeInTheDocument();
    await waitFor(() =>
      expect(document.querySelectorAll('.manifold-force-point')).toHaveLength(2)
    );

    const olderTurnMessage = screen.getByText('CI is failing. Can you debug it?').closest('.msg');
    expect(olderTurnMessage).not.toBeNull();
    fireEvent.click(olderTurnMessage as HTMLElement);

    await waitFor(() => expect(olderTurnMessage).toHaveClass('is-selected-turn'));
    expect(await screen.findByText('Action bias strengthened after local evidence.')).toBeInTheDocument();
    expect(document.querySelectorAll('.manifold-force-point')).toHaveLength(2);
  });

  it('shows steering point details in a popup instead of lower readout cards', async () => {
    const resolverProjection: ConversationProjectionSnapshot = {
      ...bootstrapProjection,
      forensics: {
        ...bootstrapProjection.forensics,
        turns: [
          {
            turn_id: 'task-123.turn-0001',
            lifecycle: 'final',
            records: [
              {
                lifecycle: 'final',
                superseded_by_record_id: null,
                record: {
                  record_id: 'record-resolver',
                  sequence: 3,
                  lineage: {
                    task_id: 'task-123',
                    turn_id: 'task-123.turn-0001',
                    branch_id: null,
                    parent_record_id: 'record-2',
                  },
                  kind: {
                    SignalSnapshot: {
                      kind: 'action_bias',
                      gate: 'convergence',
                      phase: 'narrowing',
                      summary: 'deterministic resolver resolved apps/web/src/runtime-shell.css',
                      level: 'high',
                      magnitude_percent: 79,
                      applies_to: {
                        id: 'planner-step:record-resolver',
                        kind: 'planner_step',
                        label: 'replace runtime shell padding',
                      },
                      contributions: [
                        {
                          source: 'candidate_file_evidence',
                          share_percent: 60,
                          rationale: 'Authored candidates converged on the runtime shell.',
                        },
                      ],
                      artifact: {
                        summary: 'entity resolution',
                        inline_content: JSON.stringify({
                          stage: 'entity-resolution',
                          status: 'resolved',
                          source: 'bootstrap',
                          path: 'apps/web/src/runtime-shell.css',
                          candidates: [
                            'apps/web/src/runtime-shell.css',
                            'apps/web/src/runtime-app.tsx',
                          ],
                          explanation: 'deterministic ranking selected a single authored target',
                        }),
                        mime_type: 'application/json',
                      },
                    },
                  },
                },
              },
            ],
          },
        ],
      },
      manifold: {
        ...bootstrapProjection.manifold,
        turns: [
          {
            turn_id: 'task-123.turn-0001',
            lifecycle: 'final',
            frames: [
              {
                record_id: 'record-resolver',
                sequence: 3,
                lifecycle: 'final',
                anchor: {
                  id: 'planner-step:record-resolver',
                  kind: 'planner_step',
                  label: 'replace runtime shell padding',
                },
                active_signals: [
                  {
                    snapshot_record_id: 'record-resolver',
                    lifecycle: 'final',
                    kind: 'action_bias',
                    gate: 'convergence',
                    phase: 'narrowing',
                    summary: 'deterministic resolver resolved apps/web/src/runtime-shell.css',
                    level: 'high',
                    magnitude_percent: 79,
                    anchor: {
                      id: 'planner-step:record-resolver',
                      kind: 'planner_step',
                      label: 'replace runtime shell padding',
                    },
                    contributions: [
                      {
                        source: 'candidate_file_evidence',
                        share_percent: 60,
                        rationale: 'Authored candidates converged on the runtime shell.',
                      },
                    ],
                    artifact: {
                      summary: 'entity resolution',
                      inline_content: JSON.stringify({
                        stage: 'entity-resolution',
                        status: 'resolved',
                        source: 'bootstrap',
                        path: 'apps/web/src/runtime-shell.css',
                        candidates: [
                          'apps/web/src/runtime-shell.css',
                          'apps/web/src/runtime-app.tsx',
                        ],
                        explanation: 'deterministic ranking selected a single authored target',
                      }),
                      mime_type: 'application/json',
                    },
                  },
                ],
                gates: [
                  {
                    gate: 'convergence',
                    label: 'convergence gate',
                    phase: 'narrowing',
                    level: 'high',
                    magnitude_percent: 79,
                    anchor: {
                      id: 'planner-step:record-resolver',
                      kind: 'planner_step',
                      label: 'replace runtime shell padding',
                    },
                    dominant_signal_kind: 'action_bias',
                    signal_kinds: ['action_bias'],
                    dominant_record_id: 'record-resolver',
                  },
                ],
                primitives: [
                  {
                    primitive_id: 'gate:convergence',
                    kind: 'valve',
                    label: 'Convergence gate',
                    basis: { kind: 'steering_gate', gate: 'convergence' },
                    evidence_record_id: 'record-resolver',
                    anchor: {
                      id: 'planner-step:record-resolver',
                      kind: 'planner_step',
                      label: 'replace runtime shell padding',
                    },
                    level: 'high',
                    magnitude_percent: 79,
                  },
                ],
                conduits: [],
              },
            ],
          },
        ],
      },
    };

    stubRuntimeFetch({ projection: resolverProjection });
    renderAtPath('/manifold');

    expect(document.querySelector('.manifold-readout')).toBeNull();
    expect(await screen.findByText('Resolved target')).toBeInTheDocument();
    expect(await screen.findByText('apps/web/src/runtime-shell.css')).toBeInTheDocument();
    expect(
      await screen.findByText('deterministic ranking selected a single authored target')
    ).toBeInTheDocument();

    const viewport = await screen.findByTestId('manifold-spacefield-viewport');
    fireEvent.mouseDown(viewport, { button: 0, clientX: 80, clientY: 80 });
    fireEvent.mouseUp(viewport, { button: 0, clientX: 80, clientY: 80 });
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Selected steering point details' })).toBeNull()
    );
    expect(screen.queryByText('Resolved target')).toBeNull();

    const convergencePoint = await screen.findByRole('button', {
      name: /Convergence gate, frame 1, 79%/i,
    });
    fireEvent.click(convergencePoint);
    expect(await screen.findByText('Resolved target')).toBeInTheDocument();
  });

  it('groups same-frame steering points into a piano key slice', async () => {
    const multiPointProjection: ConversationProjectionSnapshot = {
      ...bootstrapProjection,
      manifold: {
        ...bootstrapProjection.manifold,
        turns: [
          {
            ...bootstrapProjection.manifold.turns[0],
            frames: [
              {
                ...bootstrapProjection.manifold.turns[0].frames[0],
                active_signals: [
                  {
                    ...bootstrapProjection.manifold.turns[0].frames[0].active_signals[0],
                    snapshot_record_id: 'record-2-convergence',
                  },
                  {
                    ...bootstrapProjection.manifold.turns[0].frames[0].active_signals[0],
                    snapshot_record_id: 'record-2-evidence',
                    kind: 'candidate_file_evidence',
                    gate: 'evidence',
                    summary: 'Candidate file evidence accumulated around the authored target.',
                    magnitude_percent: 48,
                  },
                ],
                gates: [
                  {
                    ...bootstrapProjection.manifold.turns[0].frames[0].gates[0],
                    dominant_record_id: 'record-2-convergence',
                  },
                  {
                    ...bootstrapProjection.manifold.turns[0].frames[0].gates[0],
                    gate: 'evidence',
                    label: 'evidence gate',
                    dominant_signal_kind: 'candidate_file_evidence',
                    signal_kinds: ['candidate_file_evidence'],
                    dominant_record_id: 'record-2-evidence',
                    magnitude_percent: 48,
                    level: 'medium',
                  },
                ],
                primitives: [
                  ...bootstrapProjection.manifold.turns[0].frames[0].primitives,
                  {
                    ...bootstrapProjection.manifold.turns[0].frames[0].primitives[0],
                    primitive_id: 'gate:evidence',
                    label: 'Evidence gate',
                    basis: { kind: 'steering_gate', gate: 'evidence' },
                    magnitude_percent: 48,
                  },
                ],
              },
            ],
          },
        ],
      },
    };

    stubRuntimeFetch({ projection: multiPointProjection });
    renderAtPath('/manifold');

    await screen.findByText('Temporal gate field');
    const slices = document.querySelectorAll('.manifold-force-slice');
    expect(slices).toHaveLength(1);
    expect(slices[0]?.getAttribute('data-point-count')).toBe('2');

    const evidencePoint = await screen.findByRole('button', { name: /Evidence gate, frame 1, 48%/i });
    fireEvent.click(evidencePoint);
    expect(
      await screen.findByText('Candidate file evidence accumulated around the authored target.')
    ).toBeInTheDocument();
  });

  it('selects steering points even when the gate does not expose a dominant record id', async () => {
    const nullRecordProjection: ConversationProjectionSnapshot = {
      ...bootstrapProjection,
      manifold: {
        ...bootstrapProjection.manifold,
        turns: [
          {
            ...bootstrapProjection.manifold.turns[0],
            frames: [
              {
                ...bootstrapProjection.manifold.turns[0].frames[0],
                active_signals: [
                  {
                    ...bootstrapProjection.manifold.turns[0].frames[0].active_signals[0],
                    snapshot_record_id: 'record-2-convergence',
                    summary: 'Action bias strengthened after local evidence.',
                  },
                  {
                    ...bootstrapProjection.manifold.turns[0].frames[0].active_signals[0],
                    snapshot_record_id: 'record-2-evidence',
                    kind: 'candidate_file_evidence',
                    gate: 'evidence',
                    summary: 'Candidate file evidence accumulated around the authored target.',
                    magnitude_percent: 48,
                  },
                ],
                gates: [
                  {
                    ...bootstrapProjection.manifold.turns[0].frames[0].gates[0],
                    dominant_record_id: 'record-2-convergence',
                  },
                  {
                    ...bootstrapProjection.manifold.turns[0].frames[0].gates[0],
                    gate: 'evidence',
                    label: 'evidence gate',
                    dominant_signal_kind: 'candidate_file_evidence',
                    signal_kinds: ['candidate_file_evidence'],
                    dominant_record_id: null,
                    magnitude_percent: 48,
                    level: 'medium',
                  },
                ],
              },
            ],
          },
        ],
      },
    };

    stubRuntimeFetch({ projection: nullRecordProjection });
    renderAtPath('/manifold');

    const evidencePoint = await screen.findByRole('button', {
      name: /Evidence gate, frame 1, 48%/i,
    });
    fireEvent.click(evidencePoint);

    expect(
      await screen.findByText('Candidate file evidence accumulated around the authored target.')
    ).toBeInTheDocument();
    expect(await screen.findByText('Evidence gate')).toBeInTheDocument();
  });

  it('supports mouse pan tilt and zoom on the manifold camera', async () => {
    const outerWheel = vi.fn();
    renderAtPath(
      '/manifold',
      <div onWheel={outerWheel}>
        <RuntimeApp />
      </div>
    );

    const viewport = await screen.findByTestId('manifold-spacefield-viewport');
    const deck = await screen.findByTestId('manifold-spacefield-deck');

    expect(deck.getAttribute('data-pan-x')).toBe('0');
    expect(deck.getAttribute('data-pan-y')).toBe('0');
    expect(deck.getAttribute('data-pitch')).toBe('21');
    expect(deck.getAttribute('data-yaw')).toBe('-4');
    expect(deck.getAttribute('data-roll')).toBe('0');
    expect(deck.getAttribute('data-zoom')).toBe('1.00');

    fireEvent.mouseDown(viewport, { button: 0, clientX: 120, clientY: 120 });
    fireEvent.mouseMove(window, { clientX: 260, clientY: 330 });
    fireEvent.mouseUp(window);

    expect(deck.getAttribute('data-pitch')).not.toBe('21');
    expect(deck.getAttribute('data-yaw')).not.toBe('-4');

    fireEvent.mouseDown(viewport, { button: 0, shiftKey: true, clientX: 160, clientY: 90 });
    fireEvent.mouseMove(window, { clientX: 190, clientY: 130 });
    fireEvent.mouseUp(window);

    expect(deck.getAttribute('data-pan-x')).not.toBe('0');
    expect(deck.getAttribute('data-pan-y')).not.toBe('0');

    fireEvent.mouseDown(viewport, { button: 0, altKey: true, clientX: 160, clientY: 90 });
    fireEvent.mouseMove(window, { clientX: 260, clientY: 90 });
    fireEvent.mouseUp(window);

    expect(deck.getAttribute('data-roll')).not.toBe('0');

    const zoomEvent = new WheelEvent('wheel', {
      deltaY: -120,
      bubbles: true,
      cancelable: true,
    });
    fireEvent(viewport, zoomEvent);

    expect(deck.getAttribute('data-zoom')).not.toBe('1.00');
    expect(outerWheel).not.toHaveBeenCalled();
  });
});
