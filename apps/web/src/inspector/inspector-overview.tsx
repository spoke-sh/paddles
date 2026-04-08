import {
  FORCE_KIND_COLORS,
  FORCE_LEVEL_COLORS,
  KIND_COLORS,
  recordMeta,
  signalKindLabel,
  sourceColor,
} from '../runtime-helpers';
import type { ForensicRecordProjection, ForensicTurnProjection } from '../runtime-types';
import { comparisonSnippet, comparisonTitle } from './forensic-selectors';
import type { FocusState, InspectorDetailMode } from './use-inspector-selection';

interface InspectorOverviewProps {
  baseline: ForensicRecordProjection | null;
  comparisonRecord: ForensicRecordProjection | null;
  contributions: Array<{ label: string; percent: number; rationale?: string }>;
  currentTurn: ForensicTurnProjection | null;
  detailMode: InspectorDetailMode;
  focus: FocusState;
  signalRecords: ForensicRecordProjection[];
  strongestSignalValue: Record<string, unknown> | null;
  turns: ForensicTurnProjection[];
}

export function InspectorOverview({
  baseline,
  comparisonRecord,
  contributions,
  currentTurn,
  detailMode,
  focus,
  signalRecords,
  strongestSignalValue,
  turns,
}: InspectorOverviewProps) {
  return (
    <div className="forensic-overview" id="forensic-overview">
      <section className="forensic-overview-card" id="forensic-topology-overview">
        {!turns.length ? (
          <div className="forensic-empty">Forensic replay appears here when transit records exist.</div>
        ) : (
          <>
            <div className="forensic-card-title">Topology</div>
            <dl className="forensic-topology-metrics">
              <div className="forensic-topology-metric">
                <dt>Turns</dt>
                <dd>{turns.length}</dd>
              </div>
              <div className="forensic-topology-metric">
                <dt>Records</dt>
                <dd>{currentTurn ? currentTurn.records.length : 0}</dd>
              </div>
              <div className="forensic-topology-metric">
                <dt>Scope</dt>
                <dd>{focus.kind === 'all' ? 'all records' : focus.kind.replace('_', ' ')}</dd>
              </div>
              <div className="forensic-topology-metric">
                <dt>Selection</dt>
                <dd>{comparisonRecord ? comparisonTitle(comparisonRecord) : 'turn'}</dd>
              </div>
            </dl>
            <div className="forensic-topology-legend">
              <span className="forensic-chip">
                <span
                  className="forensic-chip-swatch"
                  style={{ ['--chip-color' as string]: KIND_COLORS.forensic }}
                />
                lineage path
              </span>
              <span className="forensic-chip">
                <span
                  className="forensic-chip-swatch"
                  style={{ ['--chip-color' as string]: KIND_COLORS.action }}
                />
                model/tool state
              </span>
              <span className="forensic-chip">
                <span
                  className="forensic-chip-swatch"
                  style={{ ['--chip-color' as string]: KIND_COLORS.signal }}
                />
                steering signals
              </span>
            </div>
          </>
        )}
      </section>

      <section className="forensic-overview-card" id="forensic-signal-overview">
        {!signalRecords.length || !strongestSignalValue ? (
          <>
            <div className="forensic-card-title">Steering Signals</div>
            <div className="forensic-empty">
              No steering signals were recorded for the current lineage selection.
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
                    KIND_COLORS.signal,
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
                  <span>{signalRecords.length} snapshots</span>
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
            Select a lineage artifact to compare it with the previous artifact in lineage.
          </div>
        ) : !baseline ? (
          <div className="forensic-empty">
            No previous artifact in lineage was available for this selection yet.
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
                {comparisonSnippet(comparisonRecord, detailMode)}
              </div>
            </div>
            <div className="forensic-shadow-pane is-baseline">
              <div className="forensic-shadow-pane-label">Baseline</div>
              <div className="forensic-shadow-pane-title">{comparisonTitle(baseline)}</div>
              <div className="forensic-shadow-pane-meta">{recordMeta(baseline)}</div>
              <div className="forensic-shadow-pane-snippet">
                {comparisonSnippet(baseline, detailMode)}
              </div>
            </div>
          </div>
        )}
      </section>
    </div>
  );
}
