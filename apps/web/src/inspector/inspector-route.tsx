import { useRuntimeStore } from '../runtime-store';
import { InspectorDetailPane } from './inspector-detail-pane';
import { InspectorNav } from './inspector-nav';
import { InspectorOverview } from './inspector-overview';
import { InspectorRecordList } from './inspector-record-list';
import { previousArtifactBaseline } from './forensic-selectors';
import { useInspectorSelection } from './use-inspector-selection';

export function InspectorRoute() {
  const { projection } = useRuntimeStore();
  const turns = projection?.forensics.turns || [];
  const {
    comparisonRecord,
    contributions,
    currentRecord,
    currentTurn,
    detailMode,
    focus,
    modelCalls,
    plannerSteps,
    records,
    selectionMode,
    signalRecords,
    strongestSignalValue,
    setDetailMode,
    focusAllRecords,
    focusModelCall,
    focusPlannerStep,
    selectConversation,
    selectRecord,
    selectTurn,
  } = useInspectorSelection(turns);
  const baseline = previousArtifactBaseline(currentTurn, comparisonRecord);

  return (
    <div className="trace-view trace-view--active forensic-view" id="forensic-view">
      <InspectorOverview
        baseline={baseline}
        comparisonRecord={comparisonRecord}
        contributions={contributions}
        currentTurn={currentTurn}
        detailMode={detailMode}
        focus={focus}
        signalRecords={signalRecords}
        strongestSignalValue={strongestSignalValue}
        turns={turns}
        onSelectRecord={selectRecord}
      />

      <div className="forensic-shell">
        <InspectorNav
          currentTurn={currentTurn}
          focus={focus}
          modelCalls={modelCalls}
          plannerSteps={plannerSteps}
          selectionMode={selectionMode}
          taskId={projection?.forensics.task_id || null}
          turns={turns}
          onFocusAllRecords={focusAllRecords}
          onFocusModelCall={focusModelCall}
          onFocusPlannerStep={focusPlannerStep}
          onSelectConversation={selectConversation}
          onSelectTurn={selectTurn}
        />

        <div className="forensic-main">
          <InspectorRecordList
            currentRecord={currentRecord}
            currentTurn={currentTurn}
            records={records}
            onSelectRecord={selectRecord}
          />
          <InspectorDetailPane
            baseline={baseline}
            currentRecord={currentRecord}
            currentTurn={currentTurn}
            detailMode={detailMode}
            focus={focus}
            modelCalls={modelCalls}
            plannerSteps={plannerSteps}
            selectionMode={selectionMode}
            taskId={projection?.forensics.task_id || null}
            turns={turns}
            onSelectDetailMode={setDetailMode}
          />
        </div>
      </div>
    </div>
  );
}
