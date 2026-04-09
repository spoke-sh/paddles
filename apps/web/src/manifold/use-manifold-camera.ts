import { useEffect, useRef, useState } from 'react';

export type ManifoldCameraState = {
  pitch: number;
  yaw: number;
  roll: number;
  panX: number;
  panY: number;
  zoom: number;
};

type ManifoldDragState = {
  mode: 'tilt' | 'pan' | 'rotate';
  startX: number;
  startY: number;
  origin: ManifoldCameraState;
};

export const DEFAULT_MANIFOLD_CAMERA: ManifoldCameraState = {
  pitch: 21,
  yaw: -4,
  roll: 0,
  panX: 0,
  panY: 0,
  zoom: 1,
};

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

export function useManifoldCamera() {
  const [camera, setCamera] = useState<ManifoldCameraState>(DEFAULT_MANIFOLD_CAMERA);
  const [dragMode, setDragMode] = useState<'tilt' | 'pan' | 'rotate' | null>(null);
  const dragStateRef = useRef<ManifoldDragState | null>(null);

  useEffect(() => {
    function stopDragging() {
      dragStateRef.current = null;
      setDragMode(null);
    }

    function handleMouseMove(event: MouseEvent) {
      const dragState = dragStateRef.current;
      if (!dragState) {
        return;
      }

      const dx = event.clientX - dragState.startX;
      const dy = event.clientY - dragState.startY;

      if (dragState.mode === 'pan') {
        setCamera({
          ...dragState.origin,
          panX: clamp(dragState.origin.panX + dx, -320, 320),
          panY: clamp(dragState.origin.panY + dy, -220, 220),
        });
        return;
      }

      if (dragState.mode === 'rotate') {
        setCamera({
          ...dragState.origin,
          roll: clamp(dragState.origin.roll + dx * 0.28, -85, 85),
        });
        return;
      }

      setCamera({
        ...dragState.origin,
        pitch: clamp(dragState.origin.pitch - dy * 0.28, 6, 96),
        yaw: clamp(dragState.origin.yaw + dx * 0.32, -88, 88),
      });
    }

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', stopDragging);
    window.addEventListener('blur', stopDragging);

    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', stopDragging);
      window.removeEventListener('blur', stopDragging);
    };
  }, []);

  function resetCamera() {
    dragStateRef.current = null;
    setDragMode(null);
    setCamera(DEFAULT_MANIFOLD_CAMERA);
  }

  function beginCameraDrag(event: React.MouseEvent<HTMLDivElement>) {
    if (event.button !== 0 && event.button !== 1) {
      return;
    }
    event.preventDefault();
    const mode =
      event.altKey ? 'rotate' : event.shiftKey || event.button === 1 ? 'pan' : 'tilt';
    dragStateRef.current = {
      mode,
      startX: event.clientX,
      startY: event.clientY,
      origin: camera,
    };
    setDragMode(mode);
  }

  function zoomFromWheel(event: React.WheelEvent<HTMLDivElement>) {
    event.preventDefault();
    event.stopPropagation();
    setCamera((current) => ({
      ...current,
      zoom: clamp(current.zoom * Math.exp(-event.deltaY * 0.0012), 0.68, 2.2),
    }));
  }

  return {
    beginCameraDrag,
    camera,
    dragMode,
    resetCamera,
    zoomFromWheel,
  };
}
