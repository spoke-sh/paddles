import { TRACE_DETAIL_LEVEL_LABELS } from '../runtime-helpers';

interface TransitToolbarProps {
  detailLevel: string;
  effectiveScope: 'significant' | 'full';
  familyVisibility: Record<string, boolean>;
  sortedNodeCount: number;
  visibleNodeCount: number;
  onSetScope: (scope: 'significant' | 'full') => void;
  onToggleFamily: (family: string) => void;
  scope: 'significant' | 'full';
}

export function TransitToolbar({
  detailLevel,
  effectiveScope,
  familyVisibility,
  sortedNodeCount,
  visibleNodeCount,
  onSetScope,
  onToggleFamily,
  scope,
}: TransitToolbarProps) {
  return (
    <div className="trace-transit-toolbar" id="trace-transit-toolbar">
      <div className="trace-transit-toggle-row">
        <div className="trace-transit-toggle-group">
          <button
            className={`trace-transit-toggle${scope === 'significant' ? ' is-active' : ''}`}
            data-trace-scope="significant"
            onClick={() => onSetScope('significant')}
            type="button"
          >
            Significant
          </button>
          <button
            className={`trace-transit-toggle${effectiveScope === 'full' ? ' is-active' : ''}`}
            data-trace-scope="full"
            onClick={() => onSetScope('full')}
            type="button"
          >
            Full Trace
          </button>
        </div>
        <div className="trace-transit-toggle-group">
          {[
            ['model_io', 'Model I/O'],
            ['lineage', 'Lineage'],
            ['signals', 'Signals'],
            ['threads', 'Threads'],
            ['tool_results', 'Tool Done'],
          ].map(([family, label]) => (
            <button
              className={`trace-transit-toggle${familyVisibility[family] ? ' is-active' : ''}`}
              data-trace-family={family}
              key={family}
              onClick={() => onToggleFamily(family)}
              type="button"
            >
              {label}
            </button>
          ))}
        </div>
      </div>
      <div className="trace-transit-meta" id="trace-transit-meta">
        {visibleNodeCount
          ? `Showing ${visibleNodeCount} of ${sortedNodeCount} steps · ${
              effectiveScope === 'full' ? 'full trace' : 'significant steps'
            } · ${TRACE_DETAIL_LEVEL_LABELS[detailLevel] || detailLevel}`
          : `Showing ${effectiveScope === 'full' ? 'full trace' : 'significant steps'} · ${
              TRACE_DETAIL_LEVEL_LABELS[detailLevel] || detailLevel
            }`}
      </div>
    </div>
  );
}
