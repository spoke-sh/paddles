import { lifecycleLabel, recordMeta, signalKindLabel, sourceColor } from '../runtime-helpers';
import type { ForensicRecordProjection } from '../runtime-types';
import type {
  MachineMomentProjection,
  MachineTurnProjection,
} from '../trace-machine/machine-projection';
import { comparisonSnippet, comparisonTitle } from './forensic-selectors';

interface ForensicMachineDrawerProps {
  baseline: ForensicRecordProjection | null;
  contributions: Array<{ label: string; percent: number; rationale?: string }>;
  currentMoment: MachineMomentProjection | null;
  currentRecord: ForensicRecordProjection | null;
  currentTurn: MachineTurnProjection | null;
  showInternals: boolean;
  strongestSignalValue: Record<string, unknown> | null;
}

function forceSummary(
  strongestSignalValue: Record<string, unknown> | null,
  contributions: Array<{ label: string; percent: number; rationale?: string }>
) {
  if (!strongestSignalValue) {
    return 'No steering forces were recorded for the selected machine moment.';
  }

  const kind = signalKindLabel(String(strongestSignalValue.kind || 'signal'));
  const level = String(strongestSignalValue.level || 'unknown');
  const magnitude = Number(strongestSignalValue.magnitude_percent || 0);
  const dominantContribution = contributions[0];

  if (!dominantContribution) {
    return `${kind} peaked at ${magnitude}% and held the machine in a ${level} steering state.`;
  }

  return `${kind} peaked at ${magnitude}% and was driven mostly by ${dominantContribution.label}.`;
}

export function ForensicMachineDrawer({
  baseline,
  contributions,
  currentMoment,
  currentRecord,
  currentTurn,
  showInternals,
  strongestSignalValue,
}: ForensicMachineDrawerProps) {
  const title = currentMoment?.headline || 'Awaiting a selected machine moment';
  const narrative =
    currentMoment?.narrative ||
    'Select a machine moment from the atlas to inspect how the turn moved.';
  const linkedRecordId = currentMoment?.raw.primaryForensicRecordId || currentRecord?.record.record_id || null;
  const strongestKind = strongestSignalValue
    ? signalKindLabel(String(strongestSignalValue.kind || 'signal'))
    : null;
  const strongestMagnitude = Number(strongestSignalValue?.magnitude_percent || 0);

  return (
    <section className="forensic-machine-drawer" id="forensic-machine-drawer">
      <div className="forensic-detail-chip-row">
        <span className="forensic-chip">{currentMoment?.label || 'No moment'}</span>
        <span className="forensic-chip">{currentTurn?.turnId || 'task'}</span>
        <span className="forensic-chip">{lifecycleLabel(currentTurn?.lifecycle || null)}</span>
        {linkedRecordId ? <span className="forensic-chip">{linkedRecordId}</span> : null}
      </div>

      <div className="forensic-machine-drawer__intro">
        <div className="forensic-machine-drawer__eyebrow">Why this moment mattered</div>
        <div className="forensic-machine-drawer__title">{title}</div>
        <div className="forensic-machine-drawer__body">{narrative}</div>
      </div>

      <div className="forensic-machine-drawer__grid">
        <section className="forensic-machine-drawer-card" id="forensic-machine-drawer-forces">
          <div className="forensic-card-title">Steering forces</div>
          <div className="forensic-machine-drawer-card__body">
            {strongestSignalValue ? (
              <>
                <div className="forensic-force-hero">
                  <div className="forensic-force-hero__value">{strongestMagnitude}%</div>
                  <div className="forensic-force-hero__meta">
                    <strong>{strongestKind}</strong>
                    <span>{String(strongestSignalValue.level || 'unknown')}</span>
                  </div>
                </div>
                <div className="forensic-machine-drawer__body">
                  {forceSummary(strongestSignalValue, contributions)}
                </div>
                <div className="forensic-contribs forensic-contribs--stacked">
                  {contributions.slice(0, 4).map((contribution) => (
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
              </>
            ) : (
              <div className="forensic-empty-state">
                No steering forces were linked to this machine moment.
              </div>
            )}
          </div>
        </section>

        <section className="forensic-machine-drawer-card" id="forensic-machine-drawer-artifacts">
          <div className="forensic-card-title">Artifact context</div>
          <div className="forensic-machine-drawer-card__body forensic-machine-drawer-card__body--split">
            <div className="forensic-shadow-pane">
              <div className="forensic-shadow-pane-label">Current</div>
              <div className="forensic-shadow-pane-title">
                {comparisonTitle(currentRecord)}
              </div>
              <div className="forensic-shadow-pane-meta">
                {currentRecord ? recordMeta(currentRecord) : 'No linked forensic record yet.'}
              </div>
              <div className="forensic-shadow-pane-snippet">
                {comparisonSnippet(currentRecord, 'rendered')}
              </div>
            </div>
            <div className="forensic-shadow-pane is-baseline">
              <div className="forensic-shadow-pane-label">Baseline</div>
              <div className="forensic-shadow-pane-title">{comparisonTitle(baseline)}</div>
              <div className="forensic-shadow-pane-meta">
                {baseline ? recordMeta(baseline) : 'No previous artifact in lineage.'}
              </div>
              <div className="forensic-shadow-pane-snippet">
                {comparisonSnippet(baseline, 'rendered')}
              </div>
            </div>
          </div>
        </section>

        <section className="forensic-machine-drawer-card" id="forensic-machine-drawer-internals-path">
          <div className="forensic-card-title">Internals path</div>
          <div className="forensic-machine-drawer-card__body">
            <div className="forensic-machine-drawer__body">
              Show internals to inspect raw payloads, record ids, and lineage anchors.
            </div>
            <div className="forensic-machine-summary__note">
              {showInternals
                ? 'Internals are open below, but the default drawer remains the primary explanation surface.'
                : 'Internals stay behind an explicit drill-down so the default route stays narrative-first.'}
            </div>
          </div>
        </section>
      </div>
    </section>
  );
}
