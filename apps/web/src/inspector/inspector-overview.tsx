import {
  FORCE_KIND_COLORS,
  FORCE_LEVEL_COLORS,
  recordMeta,
  signalKindLabel,
  sourceColor,
} from '../runtime-helpers';
import type { ForensicRecordProjection } from '../runtime-types';
import type {
  MachineMomentProjection,
  MachineTurnProjection,
} from '../trace-machine/machine-projection';
import { InspectorAtlas } from './inspector-atlas';
import { comparisonSnippet, comparisonTitle } from './forensic-selectors';

interface InspectorOverviewProps {
  baseline: ForensicRecordProjection | null;
  comparisonRecord: ForensicRecordProjection | null;
  contributions: Array<{ label: string; percent: number; rationale?: string }>;
  currentMoment: MachineMomentProjection | null;
  currentTurn: MachineTurnProjection | null;
  strongestSignalValue: Record<string, unknown> | null;
  turns: MachineTurnProjection[];
  onSelectMoment: (momentId: string) => void;
}

export function InspectorOverview({
  baseline,
  comparisonRecord,
  contributions,
  currentMoment,
  currentTurn,
  strongestSignalValue,
  turns,
  onSelectMoment,
}: InspectorOverviewProps) {
  return (
    <div className="forensic-overview" id="forensic-overview">
      <InspectorAtlas
        currentMoment={currentMoment}
        currentTurn={currentTurn}
        turns={turns}
        onSelectMoment={onSelectMoment}
      />

      <div className="forensic-overview-sidebar">
        <section className="forensic-overview-card" id="forensic-signal-overview">
          {!strongestSignalValue ? (
            <>
              <div className="forensic-card-title">Steering Signals</div>
              <div className="forensic-empty">
                No steering signals were recorded for the selected machine moment.
              </div>
            </>
          ) : (
            <>
              <div className="forensic-card-title">Steering Signals</div>
              <div className="forensic-signal-hero">
                <div
                  className="forensic-signal-gauge"
                  style={{
                    ['--signal-color' as string]:
                      FORCE_LEVEL_COLORS[String(strongestSignalValue.level)] ||
                      FORCE_KIND_COLORS[String(strongestSignalValue.kind)] ||
                      '#2d90c8',
                    ['--signal-sweep' as string]: `${Number(
                      strongestSignalValue.magnitude_percent || 0
                    )}%`,
                  }}
                >
                  <div className="forensic-signal-gauge-content">
                    <div className="forensic-signal-gauge-value">
                      {Number(strongestSignalValue.magnitude_percent || 0)}%
                    </div>
                    <div className="forensic-signal-gauge-label">
                      {String(strongestSignalValue.level || 'unknown')}
                    </div>
                  </div>
                </div>
                <div className="forensic-signal-summary">
                  <div className="forensic-signal-summary-row">
                    <strong>{signalKindLabel(String(strongestSignalValue.kind || 'signal'))}</strong>
                    <span>{currentMoment?.label || 'moment'}</span>
                  </div>
                  <div className="forensic-signal-summary-row">
                    <strong>Moment</strong>
                    <span>{currentMoment?.headline || 'Awaiting a selected machine part'}</span>
                  </div>
                  <div className="forensic-contribs forensic-contribs--stacked">
                    {contributions.slice(0, 5).map((contribution) => (
                      <span
                        className="forensic-chip"
                        key={`${contribution.label}-${contribution.percent}`}
                        title={contribution.rationale}
                      >
                        <span
                          className="forensic-chip-swatch"
                          style={{ ['--chip-color' as string]: sourceColor(contribution.label) }}
                        />
                        {contribution.label} {contribution.percent}%
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            </>
          )}
        </section>

        <section className="forensic-overview-card" id="forensic-shadow-overview">
          <div className="forensic-card-title">Shadow Baseline</div>
          {!comparisonRecord ? (
            <div className="forensic-empty">
              Select a machine moment with linked forensic evidence to compare it with the prior
              artifact in lineage.
            </div>
          ) : !baseline ? (
            <div className="forensic-empty">
              No previous artifact in lineage was available for this moment yet.
            </div>
          ) : (
            <div className="forensic-shadow-compare">
              <div className="forensic-shadow-pane">
                <div className="forensic-shadow-pane-label">Current</div>
                <div className="forensic-shadow-pane-title">
                  {comparisonTitle(comparisonRecord)}
                </div>
                <div className="forensic-shadow-pane-meta">{recordMeta(comparisonRecord)}</div>
                <div className="forensic-shadow-pane-snippet">
                  {comparisonSnippet(comparisonRecord, 'rendered')}
                </div>
              </div>
              <div className="forensic-shadow-pane is-baseline">
                <div className="forensic-shadow-pane-label">Baseline</div>
                <div className="forensic-shadow-pane-title">{comparisonTitle(baseline)}</div>
                <div className="forensic-shadow-pane-meta">{recordMeta(baseline)}</div>
                <div className="forensic-shadow-pane-snippet">
                  {comparisonSnippet(baseline, 'rendered')}
                </div>
              </div>
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
