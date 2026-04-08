import { useEffect, useRef } from 'react';
import type { UIEvent } from 'react';

const CHAT_TAIL_THRESHOLD_PX = 24;

function chatViewportNearTail(container: HTMLElement) {
  return (
    container.scrollHeight - (container.scrollTop + container.clientHeight) <=
    CHAT_TAIL_THRESHOLD_PX
  );
}

export function useStickyTailScroll({
  eventCount,
  transcriptEntryCount,
}: {
  eventCount: number;
  transcriptEntryCount: number;
}) {
  const messagesRef = useRef<HTMLDivElement | null>(null);
  const shouldStickMessagesToTailRef = useRef(true);

  useEffect(() => {
    const container = messagesRef.current;
    if (!container || !shouldStickMessagesToTailRef.current) {
      return;
    }
    container.scrollTop = container.scrollHeight;
  }, [eventCount, transcriptEntryCount]);

  function onMessagesScroll(event: UIEvent<HTMLDivElement>) {
    shouldStickMessagesToTailRef.current = chatViewportNearTail(event.currentTarget);
  }

  return { messagesRef, onMessagesScroll };
}
