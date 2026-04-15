import type {PointerEventHandler} from 'react';

const MAGNETIC_MAX_PULL_PX = 11;
const MAGNETIC_PERIMETER_EXPONENT = 0.72;
const MAGNETIC_CORE_DAMPING = 0.42;
const MAGNETIC_CORE_DAMPING_EXPONENT = 1.8;
const RECESSED_MAX_PUSH_PX = 13;
const RECESSED_OUTER_FIELD_FACTOR = 1.18;
const RECESSED_OUTER_EXPONENT = 1.42;
const RECESSED_CORE_EXPONENT = 1.1;
const RECESSED_OUTER_GAIN = 0.3;
const RECESSED_CORE_GAIN = 0.98;
const RECESSED_LOCK_THRESHOLD = 0.08;

type RecessedFieldState = {
  lockX: number;
  lockY: number;
  peakX: number;
  peakY: number;
  peakDepth: number;
};

const recessedFieldState = new WeakMap<HTMLElement, RecessedFieldState>();

function prefersReducedMotion() {
  return (
    typeof window !== 'undefined' &&
    window.matchMedia('(prefers-reduced-motion: reduce)').matches
  );
}

function resetMagneticField(element: HTMLElement) {
  element.style.setProperty('--magnetic-x', '0px');
  element.style.setProperty('--magnetic-y', '0px');
  element.style.setProperty('--magnetic-field', '0');
}

function updateMagneticField(
  element: HTMLElement,
  clientX: number,
  clientY: number,
) {
  const rect = element.getBoundingClientRect();

  if (rect.width === 0 || rect.height === 0) {
    resetMagneticField(element);
    return;
  }

  const localX = clientX - rect.left;
  const localY = clientY - rect.top;
  const centerX = rect.width / 2;
  const centerY = rect.height / 2;
  const offsetX = localX - centerX;
  const offsetY = localY - centerY;
  const normalizedX = offsetX / centerX;
  const normalizedY = offsetY / centerY;
  const radialDistance = Math.min(1, Math.hypot(normalizedX, normalizedY));
  const proximity = Math.max(0, 1 - radialDistance);
  const perimeterField = Math.pow(proximity, MAGNETIC_PERIMETER_EXPONENT);
  const coreDamping =
    1 -
    MAGNETIC_CORE_DAMPING * Math.pow(proximity, MAGNETIC_CORE_DAMPING_EXPONENT);
  const magneticField = perimeterField * coreDamping;
  const pullX = normalizedX * MAGNETIC_MAX_PULL_PX * magneticField;
  const pullY = normalizedY * MAGNETIC_MAX_PULL_PX * magneticField;

  element.style.setProperty('--magnetic-x', `${pullX.toFixed(2)}px`);
  element.style.setProperty('--magnetic-y', `${pullY.toFixed(2)}px`);
  element.style.setProperty('--magnetic-field', magneticField.toFixed(3));
}

function resetRecessedField(element: HTMLElement) {
  recessedFieldState.delete(element);
  element.style.setProperty('--button-repel-x', '0px');
  element.style.setProperty('--button-repel-y', '0px');
  element.style.setProperty('--button-depth', '0');
}

function getRecessedFieldState(element: HTMLElement) {
  let state = recessedFieldState.get(element);

  if (!state) {
    state = {
      lockX: 0,
      lockY: 0,
      peakX: 0,
      peakY: 0,
      peakDepth: 0,
    };
    recessedFieldState.set(element, state);
  }

  return state;
}

function updateRecessedField(
  element: HTMLElement,
  clientX: number,
  clientY: number,
) {
  const rect = element.getBoundingClientRect();

  if (rect.width === 0 || rect.height === 0) {
    resetRecessedField(element);
    return;
  }

  const localX = clientX - rect.left;
  const localY = clientY - rect.top;
  const centerX = rect.width / 2;
  const centerY = rect.height / 2;
  const offsetX = localX - centerX;
  const offsetY = localY - centerY;
  const normalizedX = offsetX / centerX;
  const normalizedY = offsetY / centerY;
  const outerNormalizedX = offsetX / (centerX * RECESSED_OUTER_FIELD_FACTOR);
  const outerNormalizedY = offsetY / (centerY * RECESSED_OUTER_FIELD_FACTOR);
  const outerDistance = Math.min(
    1,
    Math.hypot(outerNormalizedX, outerNormalizedY),
  );
  const coreDistance = Math.min(1, Math.hypot(normalizedX, normalizedY));
  const outerProximity = Math.max(0, 1 - outerDistance);
  const coreProximity = Math.max(0, 1 - coreDistance);
  const outerField = Math.pow(outerProximity, RECESSED_OUTER_EXPONENT);
  const coreField = Math.pow(coreProximity, RECESSED_CORE_EXPONENT);
  const repelField = Math.min(
    1,
    outerField * RECESSED_OUTER_GAIN + coreField * RECESSED_CORE_GAIN,
  );
  const state = getRecessedFieldState(element);

  if (state.lockX === 0 && Math.abs(normalizedX) >= RECESSED_LOCK_THRESHOLD) {
    state.lockX = -Math.sign(normalizedX);
  }

  if (state.lockY === 0 && Math.abs(normalizedY) >= RECESSED_LOCK_THRESHOLD) {
    state.lockY = -Math.sign(normalizedY);
  }

  state.peakX = Math.max(
    state.peakX,
    Math.abs(normalizedX) * RECESSED_MAX_PUSH_PX * repelField,
  );
  state.peakY = Math.max(
    state.peakY,
    Math.abs(normalizedY) * RECESSED_MAX_PUSH_PX * repelField,
  );
  state.peakDepth = Math.max(state.peakDepth, repelField);

  const pushX = state.lockX * state.peakX;
  const pushY = state.lockY * state.peakY;

  element.style.setProperty('--button-repel-x', `${pushX.toFixed(2)}px`);
  element.style.setProperty('--button-repel-y', `${pushY.toFixed(2)}px`);
  element.style.setProperty('--button-depth', state.peakDepth.toFixed(3));
}

export function magneticFieldEvents<T extends HTMLElement>() {
  const handlePointerLeave: PointerEventHandler<T> = (event) => {
    resetMagneticField(event.currentTarget);
  };

  const handlePointerMove: PointerEventHandler<T> = (event) => {
    if (event.pointerType === 'touch' || prefersReducedMotion()) {
      resetMagneticField(event.currentTarget);
      return;
    }

    updateMagneticField(event.currentTarget, event.clientX, event.clientY);
  };

  return {
    onPointerCancel: handlePointerLeave,
    onPointerLeave: handlePointerLeave,
    onPointerMove: handlePointerMove,
  };
}

export function recessedFieldEvents<T extends HTMLElement>() {
  const handlePointerLeave: PointerEventHandler<T> = (event) => {
    resetRecessedField(event.currentTarget);
  };

  const handlePointerMove: PointerEventHandler<T> = (event) => {
    if (event.pointerType === 'touch' || prefersReducedMotion()) {
      resetRecessedField(event.currentTarget);
      return;
    }

    updateRecessedField(event.currentTarget, event.clientX, event.clientY);
  };

  return {
    onPointerCancel: handlePointerLeave,
    onPointerLeave: handlePointerLeave,
    onPointerMove: handlePointerMove,
  };
}
