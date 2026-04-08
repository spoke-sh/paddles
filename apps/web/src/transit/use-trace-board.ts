import { useEffect, useMemo, useRef, useState } from 'react';

import {
  TRACE_ZOOM_MAX,
  TRACE_ZOOM_MIN,
  traceDetailLevelForZoom,
  traceLayoutForZoom,
  traceNodeVisible,
} from '../runtime-helpers';
import type { ConversationTraceGraph, ConversationTraceGraphNode } from '../runtime-types';

export function useTraceBoard(graph: ConversationTraceGraph | null) {
  const boardRef = useRef<HTMLDivElement | null>(null);
  const canvasRef = useRef<HTMLDivElement | null>(null);
  const [boardWidth, setBoardWidth] = useState(960);
  const [zoom, setZoom] = useState(1);
  const [scope, setScope] = useState<'significant' | 'full'>('significant');
  const [familyVisibility, setFamilyVisibility] = useState<Record<string, boolean>>({
    model_io: false,
    lineage: false,
    signals: false,
    threads: false,
    tool_results: false,
  });
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [pathD, setPathD] = useState('');
  const panRef = useRef<{
    dragging: boolean;
    startX: number;
    startY: number;
    originX: number;
    originY: number;
  } | null>(null);

  useEffect(() => {
    const board = boardRef.current;
    if (!board || typeof ResizeObserver === 'undefined') {
      return;
    }
    const observer = new ResizeObserver(() => {
      setBoardWidth(board.clientWidth || 960);
    });
    observer.observe(board);
    setBoardWidth(board.clientWidth || 960);
    return () => observer.disconnect();
  }, []);

  const sortedNodes = useMemo(
    () => [...(graph?.nodes || [])].sort((left, right) => left.sequence - right.sequence),
    [graph?.nodes]
  );

  const visibleNodes = useMemo(() => {
    let nodes = sortedNodes.filter((node) => traceNodeVisible(node, scope, familyVisibility));
    if (!nodes.length && scope === 'significant') {
      nodes = sortedNodes.slice();
    }
    return nodes;
  }, [familyVisibility, scope, sortedNodes]);

  const detailLevel = traceDetailLevelForZoom(zoom);
  const layout = traceLayoutForZoom(zoom);
  const effectiveScope =
    scope === 'significant' &&
    !sortedNodes.filter((node) => traceNodeVisible(node, scope, familyVisibility)).length
      ? 'full'
      : scope;
  const columns = Math.max(
    2,
    Math.floor((Math.max(boardWidth, 280) + layout.columnGap) / (layout.tileMin + layout.columnGap))
  );
  const rows: ConversationTraceGraphNode[][] = [];
  for (let index = 0; index < visibleNodes.length; index += columns) {
    rows.push(visibleNodes.slice(index, index + columns));
  }
  const branchLabels = Object.fromEntries(
    (graph?.branches || []).map((branch) => [branch.id, branch.label])
  );

  useEffect(() => {
    const board = boardRef.current;
    const canvas = canvasRef.current;
    if (!board || !canvas) {
      return;
    }
    const nodes = Array.from(canvas.querySelectorAll<HTMLElement>('.trace-node'));
    if (nodes.length < 2) {
      setPathD('');
      return;
    }
    const canvasRect = canvas.getBoundingClientRect();
    const points = nodes.map((node) => {
      const rect = node.getBoundingClientRect();
      return {
        x: rect.left - canvasRect.left + rect.width / 2,
        y: rect.top - canvasRect.top + rect.height / 2,
      };
    });
    let nextPath = `M ${points[0].x} ${points[0].y}`;
    for (let index = 1; index < points.length; index += 1) {
      const prev = points[index - 1];
      const next = points[index];
      const midX = (prev.x + next.x) / 2;
      nextPath += ` C ${midX} ${prev.y}, ${midX} ${next.y}, ${next.x} ${next.y}`;
    }
    setPathD(nextPath);
  }, [detailLevel, rows, zoom]);

  useEffect(() => {
    function handleMouseMove(event: MouseEvent) {
      if (!panRef.current?.dragging) {
        return;
      }
      setPan({
        x: panRef.current.originX + (event.clientX - panRef.current.startX),
        y: panRef.current.originY + (event.clientY - panRef.current.startY),
      });
    }
    function handleMouseUp() {
      panRef.current = null;
    }
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    window.addEventListener('blur', handleMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
      window.removeEventListener('blur', handleMouseUp);
    };
  }, []);

  function onBoardWheel(event: React.WheelEvent<HTMLDivElement>) {
    if (!visibleNodes.length) {
      return;
    }
    event.preventDefault();
    const nextZoom = Math.max(
      TRACE_ZOOM_MIN,
      Math.min(TRACE_ZOOM_MAX, zoom * Math.exp(-event.deltaY * 0.0015))
    );
    setZoom(nextZoom);
  }

  function onBoardMouseDown(event: React.MouseEvent<HTMLDivElement>) {
    if (event.button !== 0) {
      return;
    }
    panRef.current = {
      dragging: true,
      startX: event.clientX,
      startY: event.clientY,
      originX: pan.x,
      originY: pan.y,
    };
  }

  function toggleFamily(family: string) {
    setFamilyVisibility((current) => ({
      ...current,
      [family]: !current[family],
    }));
  }

  return {
    boardRef,
    branchLabels,
    canvasRef,
    detailLevel,
    effectiveScope,
    familyVisibility,
    layout,
    pan,
    pathD,
    rows,
    scope,
    sortedNodes,
    visibleNodes,
    zoom,
    onBoardMouseDown,
    onBoardWheel,
    setScope,
    toggleFamily,
  };
}
