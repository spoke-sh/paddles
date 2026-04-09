import { useRuntimeStore } from '../runtime-store';
import { useManifoldTurnSelection } from '../chat/manifold-turn-selection-context';
import { ManifoldStage } from './manifold-stage';
import { useManifoldCamera } from './use-manifold-camera';
import { useManifoldPlayback } from './use-manifold-playback';

export function ManifoldRoute() {
  const { projection } = useRuntimeStore();
  const turns = projection?.manifold.turns || [];
  const { selectedTurnId } = useManifoldTurnSelection();
  const playback = useManifoldPlayback(turns, selectedTurnId);
  const camera = useManifoldCamera();

  return (
    <div className="trace-view trace-view--active trace-view--manifold manifold-view" id="manifold-view">
      <div className="manifold-shell" id="manifold-shell">
        <ManifoldStage
          camera={camera.camera}
          currentFrame={playback.currentFrame}
          currentTurn={playback.currentTurn}
          dragMode={camera.dragMode}
          effectiveFrameIndex={playback.effectiveFrameIndex}
          gateField={playback.gateField}
          playing={playback.playing}
          selectedGate={playback.selectedGate}
          selectedPointKey={playback.selectedPointKey}
          selectedResolverOutcome={playback.selectedResolverOutcome}
          selectedSignal={playback.selectedSignal}
          selectedSourceRecordId={playback.selectedSourceRecordId}
          taskId={projection?.manifold.task_id || null}
          totalFrames={playback.totalFrames}
          turnsCount={turns.length}
          onBeginCameraDrag={camera.beginCameraDrag}
          onClearSelection={playback.clearSelection}
          onFrameChange={playback.selectFrame}
          onPointSelect={playback.selectPoint}
          onResetView={camera.resetCamera}
          onTogglePlay={() => playback.setPlaying((current) => !current)}
          onViewportWheel={camera.zoomFromWheel}
        />
      </div>
    </div>
  );
}
