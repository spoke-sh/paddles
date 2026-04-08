import { useRef, useState } from 'react';
import type { ClipboardEvent, FormEvent, KeyboardEvent } from 'react';

import { truncate } from '../runtime-helpers';

export type ComposerPart =
  | { id: string; kind: 'text'; text: string }
  | { id: string; kind: 'paste'; text: string; lines: number; preview: string };

function normalizeComposerText(text: string) {
  return text.replace(/\r\n/g, '\n');
}

function pastedLineCount(text: string) {
  const normalized = normalizeComposerText(text).replace(/\n+$/, '');
  if (!normalized) {
    return 0;
  }
  return normalized.split('\n').length;
}

function shouldCompressPastedText(text: string) {
  return pastedLineCount(text) > 1;
}

export function useChatComposer({
  promptHistory,
  onSubmitPrompt,
}: {
  promptHistory: string[];
  onSubmitPrompt: (prompt: string) => Promise<void>;
}) {
  const [prompt, setPrompt] = useState('');
  const [composerParts, setComposerParts] = useState<ComposerPart[]>([]);
  const [historyCursor, setHistoryCursor] = useState<number | null>(null);
  const [historyDraft, setHistoryDraft] = useState('');
  const composerPartId = useRef(0);

  function nextComposerPartId() {
    composerPartId.current += 1;
    return `composer-part-${composerPartId.current}`;
  }

  function createTextPart(text: string): ComposerPart {
    return {
      id: nextComposerPartId(),
      kind: 'text',
      text,
    };
  }

  function createPastePart(text: string): ComposerPart {
    const normalized = normalizeComposerText(text);
    const trimmed = normalized.trimEnd();
    const lines = pastedLineCount(normalized);
    const previewSource =
      trimmed
        .split('\n')
        .map((line) => line.trim())
        .find((line) => line.length > 0) ||
      trimmed.split('\n')[0] ||
      '';
    return {
      id: nextComposerPartId(),
      kind: 'paste',
      text: normalized,
      lines,
      preview: truncate(previewSource, 48),
    };
  }

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const text = [...composerParts.map((part) => part.text), prompt].join('');
    if (!text.trim()) {
      return;
    }
    setComposerParts([]);
    setHistoryCursor(null);
    setHistoryDraft('');
    setPrompt('');
    await onSubmitPrompt(text);
  }

  function historyBack() {
    if (composerParts.length > 0 || promptHistory.length === 0) {
      return;
    }
    if (historyCursor === null) {
      setHistoryDraft(prompt);
      const nextCursor = promptHistory.length - 1;
      setHistoryCursor(nextCursor);
      setPrompt(promptHistory[nextCursor]);
      return;
    }
    if (historyCursor === 0) {
      return;
    }
    const nextCursor = historyCursor - 1;
    setHistoryCursor(nextCursor);
    setPrompt(promptHistory[nextCursor]);
  }

  function historyForward() {
    if (composerParts.length > 0 || historyCursor === null) {
      return;
    }
    if (historyCursor + 1 < promptHistory.length) {
      const nextCursor = historyCursor + 1;
      setHistoryCursor(nextCursor);
      setPrompt(promptHistory[nextCursor]);
      return;
    }
    setHistoryCursor(null);
    setPrompt(historyDraft);
  }

  function popComposerPart() {
    if (composerParts.length === 0) {
      return;
    }
    const next = [...composerParts];
    const removed = next.pop();
    setComposerParts(next);
    if (removed?.kind === 'text') {
      setPrompt(removed.text);
    }
  }

  function onPromptKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === 'ArrowUp') {
      event.preventDefault();
      historyBack();
      return;
    }
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      historyForward();
      return;
    }
    if ((event.key === 'Backspace' || event.key === 'Delete') && prompt.length === 0) {
      if (composerParts.length > 0) {
        event.preventDefault();
        popComposerPart();
      }
    }
  }

  function onPromptPaste(event: ClipboardEvent<HTMLInputElement>) {
    const text = event.clipboardData.getData('text');
    if (!shouldCompressPastedText(text)) {
      return;
    }
    event.preventDefault();
    setHistoryCursor(null);
    setHistoryDraft('');
    setComposerParts((current) => {
      const next = [...current];
      if (prompt.length > 0) {
        next.push(createTextPart(prompt));
      }
      next.push(createPastePart(text));
      return next;
    });
    setPrompt('');
  }

  return {
    composerParts,
    onPromptKeyDown,
    onPromptPaste,
    onSubmit,
    prompt,
    setPrompt,
  };
}
