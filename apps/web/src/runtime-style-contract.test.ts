import { readFileSync } from 'node:fs';

import { describe, expect, it } from 'vitest';

function readRelative(path: string) {
  return readFileSync(new URL(path, import.meta.url), 'utf8');
}

describe('runtime stylesheet partition', () => {
  it('composes runtime-shell.css from feature-aligned stylesheet imports', () => {
    const runtimeShellCss = readRelative('./runtime-shell.css');

    expect(runtimeShellCss).toContain("@import './styles/runtime-shell-base.css';");
    expect(runtimeShellCss).toContain("@import './styles/chat.css';");
    expect(runtimeShellCss).toContain("@import './styles/inspector.css';");
    expect(runtimeShellCss).toContain("@import './styles/manifold.css';");
    expect(runtimeShellCss).toContain("@import './styles/transit.css';");
  });

  it('keeps shared tokens explicit while parking feature rules in domain stylesheets', () => {
    const shellBaseCss = readRelative('./styles/runtime-shell-base.css');
    const chatCss = readRelative('./styles/chat.css');
    const inspectorCss = readRelative('./styles/inspector.css');
    const manifoldCss = readRelative('./styles/manifold.css');
    const transitCss = readRelative('./styles/transit.css');

    expect(shellBaseCss).toContain(':root');
    expect(shellBaseCss).toContain('.runtime-shell-host');
    expect(chatCss).toContain('.chat-header');
    expect(inspectorCss).toContain('.forensic-view');
    expect(manifoldCss).toContain('.manifold-stage');
    expect(transitCss).toContain('.trace-transit-toolbar');
  });

  it('lets the manifold stage claim the full trace panel height for the temporal gate field', () => {
    const manifoldCss = readRelative('./styles/manifold.css');

    expect(manifoldCss).toContain('.manifold-shell {');
    expect(manifoldCss).toContain('.manifold-stage {');
    expect(manifoldCss).toContain('.manifold-stage {\n  flex: 1;');
    expect(manifoldCss).toContain('.manifold-canvas {\n  flex: 1;');
    expect(manifoldCss).toContain('.manifold-spacefield__viewport {\n  position: relative;\n  min-height: 0;\n  height: 100%;');
  });
});
