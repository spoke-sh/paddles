import {useEffect, useRef, useState} from 'react';
import type {CSSProperties, PointerEvent} from 'react';

import styles from './styles.module.css';

type TurnDividerProps = {
  arcSide?: 'left' | 'right';
  compact?: boolean;
  turns?: 2 | 3 | 4;
};

type StepPosition = {
  rotate: number;
  x: number;
  y: number;
};

type PointerState = {
  height: number;
  width: number;
  x: number;
  y: number;
};

const GRAVITY_OUTER_RADIUS_FACTOR = 0.62;
const GRAVITY_INNER_RADIUS_FACTOR = 0.26;
const GRAVITY_OUTER_ENTRY_THRESHOLD = 0.24;
const GRAVITY_OUTER_SCALE_GAIN = 0.14;
const GRAVITY_INNER_SCALE_GAIN = 0.68;
const FULL_SCROLL_BASE_VIEWPORT = 0.58;
const FULL_SCROLL_STEP_VIEWPORT = 0.22;
const COMPACT_SCROLL_BASE_VIEWPORT = 0.44;
const COMPACT_SCROLL_STEP_VIEWPORT = 0.16;
const SCROLL_DURATION_BASE_MS = 360;
const SCROLL_DURATION_STEP_MS = 90;

const STEP_PATTERNS: Record<2 | 3 | 4, StepPosition[]> = {
  2: [
    {x: 28, y: 22, rotate: 10},
    {x: 58, y: 68, rotate: 26},
  ],
  3: [
    {x: 24, y: 16, rotate: 8},
    {x: 56, y: 48, rotate: 20},
    {x: 36, y: 82, rotate: 30},
  ],
  4: [
    {x: 18, y: 12, rotate: 8},
    {x: 46, y: 32, rotate: 16},
    {x: 60, y: 58, rotate: 24},
    {x: 34, y: 84, rotate: 32},
  ],
};

export default function TurnDivider({
  arcSide = 'right',
  compact = false,
  turns = 3,
}: TurnDividerProps) {
  const scrollAnimationRef = useRef<number | null>(null);
  const [pointer, setPointer] = useState<PointerState | null>(null);
  const steps = STEP_PATTERNS[turns];
  const wrapClass = [
    styles.wrap,
    arcSide === 'left' ? styles.left : styles.right,
    compact ? styles.compact : '',
  ]
    .filter(Boolean)
    .join(' ');

  useEffect(() => {
    return () => {
      if (scrollAnimationRef.current !== null) {
        cancelAnimationFrame(scrollAnimationRef.current);
      }
    };
  }, []);

  const prefersReducedMotion = () =>
    typeof window !== 'undefined' &&
    window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  const stopScrollAnimation = () => {
    if (scrollAnimationRef.current !== null) {
      cancelAnimationFrame(scrollAnimationRef.current);
      scrollAnimationRef.current = null;
    }
  };

  const handlePointerLeave = () => {
    setPointer(null);
  };

  const handlePointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (event.pointerType === 'touch' || prefersReducedMotion()) {
      return;
    }

    const rect = event.currentTarget.getBoundingClientRect();

    setPointer({
      height: rect.height,
      width: rect.width,
      x: event.clientX - rect.left,
      y: event.clientY - rect.top,
    });
  };

  const animateScrollTo = (targetY: number, durationMs: number) => {
    if (typeof window === 'undefined' || typeof document === 'undefined') {
      return;
    }

    const maxScrollY = Math.max(
      0,
      document.documentElement.scrollHeight - window.innerHeight,
    );
    const nextY = Math.max(window.scrollY, Math.min(targetY, maxScrollY));
    const delta = nextY - window.scrollY;

    if (Math.abs(delta) < 2) {
      return;
    }

    if (prefersReducedMotion()) {
      stopScrollAnimation();
      window.scrollTo({top: nextY});
      return;
    }

    const startY = window.scrollY;
    const startTime = performance.now();

    stopScrollAnimation();

    const tick = (now: number) => {
      const elapsed = now - startTime;
      const progress = Math.min(1, elapsed / durationMs);
      const eased = 1 - Math.pow(1 - progress, 3);

      window.scrollTo({top: startY + delta * eased});

      if (progress < 1) {
        scrollAnimationRef.current = requestAnimationFrame(tick);
      } else {
        scrollAnimationRef.current = null;
      }
    };

    scrollAnimationRef.current = requestAnimationFrame(tick);
  };

  const handleStepClick = (stepIndex: number) => {
    if (typeof window === 'undefined') {
      return;
    }

    const scrollViewportUnits = compact
      ? COMPACT_SCROLL_BASE_VIEWPORT + stepIndex * COMPACT_SCROLL_STEP_VIEWPORT
      : FULL_SCROLL_BASE_VIEWPORT + stepIndex * FULL_SCROLL_STEP_VIEWPORT;
    const targetY = window.scrollY + window.innerHeight * scrollViewportUnits;
    const durationMs = SCROLL_DURATION_BASE_MS + stepIndex * SCROLL_DURATION_STEP_MS;

    animateScrollTo(targetY, durationMs);
  };

  return (
    <div aria-label="Turnstep navigator" className={wrapClass} role="group">
      <div
        className={styles.trail}
        onPointerLeave={handlePointerLeave}
        onPointerMove={handlePointerMove}>
        {steps.map((step, index) => {
          const x = arcSide === 'left' ? 100 - step.x : step.x;
          let cursorScale = 1;

          if (pointer) {
            const stepCenterX = (x / 100) * pointer.width;
            const stepCenterY = (step.y / 100) * pointer.height;
            const gravityExtent = Math.max(pointer.width, pointer.height);
            const outerRadius = gravityExtent * GRAVITY_OUTER_RADIUS_FACTOR;
            const innerRadius = gravityExtent * GRAVITY_INNER_RADIUS_FACTOR;
            const distance = Math.hypot(
              pointer.x - stepCenterX,
              pointer.y - stepCenterY,
            );
            const outerProximity = Math.max(0, 1 - distance / outerRadius);
            const innerProximity = Math.max(0, 1 - distance / innerRadius);
            const outerInfluence =
              outerProximity <= GRAVITY_OUTER_ENTRY_THRESHOLD
                ? 0
                : Math.pow(
                    (outerProximity - GRAVITY_OUTER_ENTRY_THRESHOLD) /
                      (1 - GRAVITY_OUTER_ENTRY_THRESHOLD),
                    2.35,
                  );
            const innerInfluence = Math.pow(innerProximity, 1.4);

            cursorScale =
              1 +
              outerInfluence * GRAVITY_OUTER_SCALE_GAIN +
              innerInfluence * GRAVITY_INNER_SCALE_GAIN;
          }

          const style = {
            '--step-cursor-scale': cursorScale.toFixed(3),
            '--step-delay': `calc(${index} * var(--keel-motion-delay-turnstep))`,
            '--step-rotate': `${arcSide === 'left' ? -step.rotate : step.rotate}deg`,
            '--step-x': `${x}%`,
            '--step-y': `${step.y}%`,
          } as CSSProperties;

          return (
            <button
              aria-label={`Scroll ahead ${index + 1} ${
                index === 0 ? 'turn' : 'turns'
              }`}
              type="button"
              className={styles.step}
              data-step={index + 1}
              key={`${step.x}-${step.y}-${step.rotate}`}
              onClick={() => handleStepClick(index)}
              style={style}
            />
          );
        })}
      </div>
    </div>
  );
}
