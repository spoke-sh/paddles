import { lifecycleLabel, truncate } from '../runtime-helpers';
import type { ForensicTurnProjection } from '../runtime-types';
import type {
  FocusState,
  InspectorModelCall,
  InspectorPlannerStep,
  InspectorSelectionMode,
} from './use-inspector-selection';

interface InspectorNavProps {
  currentTurn: ForensicTurnProjection | null;
  focus: FocusState;
  modelCalls: InspectorModelCall[];
  plannerSteps: InspectorPlannerStep[];
  selectionMode: InspectorSelectionMode;
  taskId: string | null;
  turns: ForensicTurnProjection[];
  onFocusAllRecords: () => void;
  onFocusModelCall: (modelCallId: string) => void;
  onFocusPlannerStep: (plannerStepId: string) => void;
  onSelectConversation: () => void;
  onSelectTurn: (turnId: string) => void;
}

export function InspectorNav({
  currentTurn,
  focus,
  modelCalls,
  plannerSteps,
  selectionMode,
  taskId,
  turns,
  onFocusAllRecords,
  onFocusModelCall,
  onFocusPlannerStep,
  onSelectConversation,
  onSelectTurn,
}: InspectorNavProps) {
  return (
    <aside className="forensic-nav" id="forensic-nav">
      {!turns.length ? (
        <div className="forensic-empty-state">No forensic replay is available yet.</div>
      ) : (
        <>
          <div className="forensic-nav-group">
            <div className="forensic-nav-group-title">Conversation</div>
            <button
              className={`forensic-nav-button${selectionMode === 'conversation' ? ' is-active' : ''}`}
              id="forensic-conversation-button"
              onClick={onSelectConversation}
              type="button"
            >
              <div className="forensic-nav-title">
                <span>{taskId || 'conversation'}</span>
                <span>{turns.length}</span>
              </div>
              <div className="forensic-nav-meta">turns · replay-backed lineage surface</div>
            </button>
          </div>

          <div className="forensic-nav-group">
            <div className="forensic-nav-group-title">Turns</div>
            {turns.map((turn) => (
              <button
                className={`forensic-nav-button${
                  currentTurn?.turn_id === turn.turn_id && focus.kind === 'all'
                    ? ' is-active'
                    : ''
                } is-${turn.lifecycle}`}
                data-turn-id={turn.turn_id}
                key={turn.turn_id}
                onClick={() => onSelectTurn(turn.turn_id)}
                type="button"
              >
                <div className="forensic-nav-title">
                  <span>{truncate(turn.turn_id, 28)}</span>
                  <span className={`forensic-lifecycle is-${turn.lifecycle}`}>
                    {lifecycleLabel(turn.lifecycle)}
                  </span>
                </div>
                <div className="forensic-nav-meta">{turn.records.length} records</div>
              </button>
            ))}
          </div>

          <div className="forensic-nav-group">
            <div className="forensic-nav-group-title">Focus</div>
            <button
              className={`forensic-nav-button${focus.kind === 'all' ? ' is-active' : ''}`}
              onClick={onFocusAllRecords}
              type="button"
            >
              <div className="forensic-nav-title">
                <span>All records</span>
                <span>{currentTurn?.records.length || 0}</span>
              </div>
              <div className="forensic-nav-meta">full turn sequence</div>
            </button>
            {modelCalls.map((modelCall) => (
              <button
                className={`forensic-nav-button${
                  focus.kind === 'model_call' && focus.id === modelCall.id ? ' is-active' : ''
                }`}
                key={modelCall.id}
                onClick={() => onFocusModelCall(modelCall.id)}
                type="button"
              >
                <div className="forensic-nav-title">
                  <span>{truncate(modelCall.summary, 34)}</span>
                  <span>{modelCall.lane}</span>
                </div>
                <div className="forensic-nav-meta">
                  {modelCall.provider}:{modelCall.model} · {modelCall.category}
                </div>
              </button>
            ))}
            {plannerSteps.map((step) => (
              <button
                className={`forensic-nav-button${
                  focus.kind === 'planner_step' && focus.id === step.id ? ' is-active' : ''
                }`}
                key={step.id}
                onClick={() => onFocusPlannerStep(step.id)}
                type="button"
              >
                <div className="forensic-nav-title">
                  <span>{step.label}</span>
                  <span>step</span>
                </div>
                <div className="forensic-nav-meta">{step.recordId}</div>
              </button>
            ))}
          </div>
        </>
      )}
    </aside>
  );
}
