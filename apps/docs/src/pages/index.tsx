import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';

import SignalGrid, {type SignalItem} from '@site/src/components/SignalGrid';
import TurnDivider from '@site/src/components/TurnDivider';
import TurnCycle, {type TurnStep} from '@site/src/components/TurnCycle';
import TermRamp, {type TermRampItem} from '@site/src/components/TermRamp';
import {recessedFieldEvents} from '@site/src/components/magneticField';

import styles from './index.module.css';

const capabilityItems: SignalItem[] = [
  {
    eyebrow: 'Operator Context',
    title: 'Load repo-specific guidance before the first token',
    body: 'Paddles reads AGENTS.md memory, foundational docs, and project conventions into the turn so the model starts from operator reality instead of generic priors.',
    href: '/docs/intro',
    cta: 'Read the intro',
  },
  {
    eyebrow: 'Recursive Investigation',
    title: 'Search, read, and refine until evidence is sufficient',
    body: 'The planner can branch through the workspace in bounded steps, which lets small local models inspect code instead of bluffing through uncertainty.',
    href: '/docs/concepts/recursive-planning',
    cta: 'See the planner loop',
  },
  {
    eyebrow: 'Structured Synthesis',
    title: 'Write the answer from evidence, not memory',
    body: 'A separate synthesis pass turns the gathered evidence bundle into the operator-facing response, keeping the final output grounded and legible.',
    href: '/docs/concepts/turn-loop',
    cta: 'See the turn loop',
  },
];

const firstTurnItems = [
  {
    label: 'Enter Shell',
    command: 'nix develop',
    href: '/docs/start-here/installation#enter-the-dev-shell',
  },
  {
    label: 'Verify Binary',
    command: 'paddles --help',
    href: '/docs/start-here/installation#verify-the-binary',
  },
  {
    label: 'Start Session',
    command: 'paddles --cuda',
    href: '/docs/start-here/first-turn#start-interactive-mode',
  },
  {
    label: 'Run One Shot',
    command: 'paddles --prompt "Summarize the turn loop"',
    href: '/docs/start-here/first-turn#one-shot-mode',
  },
];

const turnLoopItems: TurnStep[] = [
  {
    label: 'Interpret',
    command: 'Load operator memory',
    body: 'Paddles starts by reading operator guidance, workspace conventions, and runtime hints before it asks the model to act.',
    href: '/docs/reference/foundational-docs',
  },
  {
    label: 'Classify',
    command: 'Choose the response path',
    body: 'The runtime decides whether the prompt is casual, direct, deterministic, or worth a planned investigation loop.',
    href: '/docs/concepts/turn-loop',
  },
  {
    label: 'Plan',
    command: 'Search, read, refine',
    body: 'Planned turns recurse through bounded evidence gathering instead of spending the entire budget on a single completion.',
    href: '/docs/concepts/recursive-planning',
  },
  {
    label: 'Synthesize',
    command: 'Write from the evidence bundle',
    body: 'The synthesizer turns the gathered trace into a grounded answer that stays readable for the operator.',
    href: '/docs/concepts/model-routing',
  },
  {
    label: 'Record',
    command: 'Persist typed traces',
    body: 'Each step is captured in stable runtime records so the TUI and web shell can replay the same turn history.',
    href: '/docs/start-here/first-turn#read-the-trace',
  },
];

const vocabularyItems: TermRampItem[] = [
  {
    plain: 'Project conventions',
    term: 'Operator memory',
    body: 'The repo-authored guidance Paddles loads before planning so the model begins with local rules and priorities.',
    href: '/docs/reference/foundational-docs',
  },
  {
    plain: 'Workspace investigation',
    term: 'Recursive planning',
    body: 'A bounded loop of search, read, and refinement that gathers evidence until the planner decides the turn is grounded enough.',
    href: '/docs/concepts/recursive-planning',
  },
  {
    plain: 'Context budget',
    term: 'Context pressure',
    body: 'The signals Paddles uses to track how much of the available context window is already occupied and what needs compression.',
    href: '/docs/concepts/context-pressure',
  },
  {
    plain: 'Different input surfaces',
    term: 'Context tiers',
    body: 'Paddles distinguishes inline, transit, and filesystem context so the operator can see how evidence was gathered.',
    href: '/docs/concepts/context-tiers',
  },
  {
    plain: 'Model split',
    term: 'Model routing',
    body: 'Planner and synthesizer roles can point at different models so deeper investigation does not force a heavier answer writer.',
    href: '/docs/concepts/model-routing',
  },
  {
    plain: 'Grounded answer',
    term: 'Synthesis step',
    body: 'The final pass that assembles the response strictly from the evidence bundle accumulated during the turn.',
    href: '/docs/concepts/turn-loop',
  },
];

const surfaceItems: SignalItem[] = [
  {
    eyebrow: 'Context Tiers',
    title: 'Keep inline, transit, and filesystem spans distinct',
    body: 'Paddles separates short prompt context from fetched evidence and workspace reads so operators can inspect where each piece came from.',
    href: '/docs/concepts/context-tiers',
    cta: 'Read the context model',
  },
  {
    eyebrow: 'Model Routing',
    title: 'Route investigation and answer writing independently',
    body: 'A light synthesizer can pair with a heavier planner, which gives you deeper workspace inspection without paying the same cost for every final response.',
    href: '/docs/concepts/model-routing',
    cta: 'See routing strategy',
  },
];

const readingPathItems: SignalItem[] = [
  {
    eyebrow: 'Start Here',
    title: 'Install the runtime and run a first turn',
    body: 'Use the installation and first-turn guides when you want the shortest path from clone to a live local session.',
    href: '/docs/start-here/installation',
    cta: 'Open the getting-started path',
  },
  {
    eyebrow: 'Turn Mechanics',
    title: 'Understand the planner and synthesis loop',
    body: 'Read how Paddles classifies prompts, runs recursive investigation, and assembles grounded answers.',
    href: '/docs/concepts/turn-loop',
    cta: 'Open the loop docs',
  },
  {
    eyebrow: 'Retrieval And Context',
    title: 'See how evidence is gathered and bounded',
    body: 'The retrieval docs explain query construction, traceability, context tiers, and how pressure is surfaced back to the operator.',
    href: '/docs/concepts/search-retrieval',
    cta: 'Open retrieval docs',
  },
  {
    eyebrow: 'Reference Surface',
    title: 'Inspect the foundational contracts',
    body: 'Use the reference pages when you need the owning source documents and transport/runtime constraints in one place.',
    href: '/docs/reference/foundational-docs',
    cta: 'Open reference docs',
  },
];

export default function Home(): ReactNode {
  return (
    <Layout
      title="Paddles"
      description="Paddles is a recursive in-context planning harness for local-first coding agents.">
      <main className={styles.page}>
        <section className={styles.hero}>
          <div className="container">
            <div className={styles.heroGrid}>
              <div className={styles.heroCopy}>
                <p className={styles.eyebrow}>Recursive Planning Harness</p>
                <h1>Make small local models behave like grounded coding agents.</h1>
                <p className={styles.lede}>
                  Paddles is a local-first harness that gives small models
                  operator context, bounded recursive investigation, and a
                  synthesis pass that writes from evidence instead of guesswork.
                </p>
                <div className={styles.actions}>
                  <Link
                    className={`${styles.primaryAction} keel-recessed-button`}
                    to="/docs/intro"
                    {...recessedFieldEvents<HTMLAnchorElement>()}>
                    Read The Story
                  </Link>
                  <Link
                    className={`${styles.secondaryAction} keel-recessed-button`}
                    to="/docs/start-here/first-turn"
                    {...recessedFieldEvents<HTMLAnchorElement>()}>
                    Run A First Turn
                  </Link>
                </div>
                <ul className={styles.heroPoints}>
                  <li>Load operator memory and repo docs before the model answers.</li>
                  <li>Let the planner search, read, and refine instead of improvising from memory.</li>
                  <li>Keep the same turn trace visible in both the TUI and the web runtime.</li>
                </ul>
              </div>
              <div className={styles.scenePanel}>
                <div className={styles.sceneFrame}>
                  <div className={styles.sceneChrome} aria-hidden="true">
                    <span />
                    <span />
                    <span />
                  </div>
                  <p className={styles.sceneLabel}>Typical First Turn</p>
                  <ol className={styles.sceneSteps}>
                    {firstTurnItems.map((item) => (
                      <li key={item.command}>
                        <Link
                          className={styles.sceneStepLink}
                          to={item.href}
                          {...recessedFieldEvents<HTMLAnchorElement>()}>
                          <span>{item.label}</span>
                          <code>{item.command}</code>
                        </Link>
                      </li>
                    ))}
                  </ol>
                </div>
              </div>
            </div>
            <TurnDivider arcSide="right" turns={3} />
          </div>
        </section>

        <section className={styles.section}>
          <div className="container">
            <div className={styles.sectionHeader}>
              <p className={styles.sectionEyebrow}>Why Teams Reach For Paddles</p>
              <h2>Paddles changes what a small local model can actually do.</h2>
              <p>
                The harness is opinionated on purpose. It adds repo-grounded
                context, recursive evidence gathering, and a visible answer path
                so local models can behave more like careful operators than
                autocomplete with better marketing.
              </p>
            </div>
            <SignalGrid items={capabilityItems} tone="hero" />
            <TurnDivider arcSide="left" turns={2} />
          </div>
        </section>

        <section className={styles.sectionAlt}>
          <div className="container">
            <div className={styles.sectionHeader}>
              <p className={styles.sectionEyebrow}>The Turn Loop</p>
              <h2>A single prompt can be vague. A traced turn is inspectable.</h2>
              <p>
                Paddles keeps the runtime legible: load guidance, choose the
                response path, investigate when necessary, and synthesize from
                evidence before the turn is recorded.
              </p>
            </div>
            <TurnCycle items={turnLoopItems} tone="hero" />
            <TurnDivider arcSide="right" turns={3} />
          </div>
        </section>

        <section className={styles.section}>
          <div className="container">
            <div className={styles.sectionHeader}>
              <p className={styles.sectionEyebrow}>Native Vocabulary</p>
              <h2>Start from the operator job, then learn the harness terms.</h2>
              <p>
                The docs introduce Paddles vocabulary by mapping it back to the
                everyday jobs you already do while steering a codebase and a
                local model.
              </p>
            </div>
            <TermRamp items={vocabularyItems} tone="hero" />
            <TurnDivider arcSide="left" turns={2} />
          </div>
        </section>

        <section className={styles.sectionAlt}>
          <div className="container">
            <div className={styles.sectionHeader}>
              <p className={styles.sectionEyebrow}>Runtime Surfaces</p>
              <h2>Context structure and model routing stay visible together.</h2>
              <p>
                Paddles exposes both how evidence enters the turn and which
                model is responsible for each phase, so operators can tune the
                runtime without losing debuggability.
              </p>
            </div>
            <SignalGrid items={surfaceItems} columns="two" tone="hero" />
            <TurnDivider arcSide="right" turns={3} />
          </div>
        </section>

        <section className={styles.section}>
          <div className="container">
            <div className={styles.sectionHeader}>
              <p className={styles.sectionEyebrow}>Reading Paths</p>
              <h2>Start with the docs path that matches the question you have.</h2>
              <p>
                After the shared intro, the docs branch into setup, loop
                mechanics, retrieval/context, and reference material so you can
                go straight to the depth you need.
              </p>
            </div>
            <SignalGrid items={readingPathItems} columns="two" tone="hero" />
            <TurnDivider arcSide="left" turns={2} />
          </div>
        </section>

        <section className={styles.ctaBand}>
          <div className="container">
            <div className={styles.ctaCard}>
              <div>
                <p className={styles.sectionEyebrow}>Start Here</p>
                <h2>Read the narrative, install the runtime, and take a traced turn.</h2>
              </div>
              <div className={styles.actions}>
                <Link
                  className={`${styles.primaryAction} keel-recessed-button`}
                  to="/docs/intro"
                  {...recessedFieldEvents<HTMLAnchorElement>()}>
                  Open The Docs
                </Link>
                <Link
                  className={`${styles.secondaryAction} keel-recessed-button`}
                  to="/docs/start-here/installation"
                  {...recessedFieldEvents<HTMLAnchorElement>()}>
                  Installation Guide
                </Link>
              </div>
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}
