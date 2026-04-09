import { useRuntimeStore } from '../runtime-store';
import { TransitMachineStage } from './transit-machine-stage';
import { useTransitMachine } from './use-transit-machine';

export function TransitRoute() {
  const { projection } = useRuntimeStore();
  const transitMachine = useTransitMachine(projection);

  return (
    <div className="trace-view trace-view--active trace-view--transit" id="transit-view">
      <TransitMachineStage
        currentMoment={transitMachine.currentMoment}
        currentTurn={transitMachine.currentTurn}
        taskId={transitMachine.taskId}
        turns={transitMachine.turns}
        onSelectMoment={transitMachine.selectMoment}
      />
    </div>
  );
}
