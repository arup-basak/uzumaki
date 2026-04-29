import type { UzNode } from './node';

export const enum EventType {
  MouseMove = 0,
  MouseDown = 1,
  MouseUp = 2,
  Click = 3,
  KeyDown = 10,
  KeyUp = 11,
  Input = 20,
  Focus = 21,
  Blur = 22,
  Copy = 25,
  Cut = 26,
  Paste = 27,
}

export const enum EventPhase {
  None = 0,
  Capture = 1,
  Target = 2,
  Bubble = 3,
}

export interface UzumakiEvent {
  readonly type: EventType | string;
  readonly target: UzNode | null;
  currentTarget: UzNode | null;
  readonly eventPhase: EventPhase;
  readonly bubbles: boolean;
  readonly defaultPrevented: boolean;
  stopPropagation(): void;
  stopImmediatePropagation(): void;
  preventDefault(): void;
}

export interface UzumakiMouseEvent extends UzumakiEvent {
  readonly x: number;
  readonly y: number;
  readonly screenX: number;
  readonly screenY: number;
  readonly button: number;
  readonly buttons: number;
}

export interface UzumakiKeyboardEvent extends UzumakiEvent {
  readonly key: string;
  readonly code: string;
  readonly keyCode: number;
  readonly repeat: boolean;
  readonly ctrlKey: boolean;
  readonly altKey: boolean;
  readonly shiftKey: boolean;
  readonly metaKey: boolean;
}

export interface UzumakiInputEvent extends UzumakiEvent {
  readonly value: string;
  readonly inputType: string;
  readonly data: string | null;
}

export interface UzumakiFocusEvent extends UzumakiEvent {}

export interface UzumakiClipboardEvent extends UzumakiEvent {
  readonly selectionText: string | null;
  readonly clipboardText: string | null;
}

export interface UzumakiResizeEvent extends UzumakiEvent {
  readonly width: number;
  readonly height: number;
}

/** DOM-style events that can be attached to any element. */
export interface EventHandlerMap {
  mousemove: UzumakiMouseEvent;
  mousedown: UzumakiMouseEvent;
  mouseup: UzumakiMouseEvent;
  click: UzumakiMouseEvent;
  keydown: UzumakiKeyboardEvent;
  keyup: UzumakiKeyboardEvent;
  input: UzumakiInputEvent;
  focus: UzumakiFocusEvent;
  blur: UzumakiFocusEvent;
  copy: UzumakiClipboardEvent;
  cut: UzumakiClipboardEvent;
  paste: UzumakiClipboardEvent;
}

/** Window receives all DOM events (for bubble/capture) plus its lifecycle events. */
export interface WindowEventMap extends EventHandlerMap {
  load: UzumakiEvent;
  close: UzumakiEvent;
  resize: UzumakiResizeEvent;
}

export type EventName = keyof EventHandlerMap;
export type WindowEventName = keyof WindowEventMap;

export type EventHandler<K extends EventName = EventName> = (
  event: EventHandlerMap[K],
) => void;

export type WindowEventHandler<K extends WindowEventName = WindowEventName> = (
  event: WindowEventMap[K],
) => void;

export const EVENT_NAME_TO_TYPE: Record<string, EventType> = {
  mousemove: EventType.MouseMove,
  mousedown: EventType.MouseDown,
  mouseup: EventType.MouseUp,
  click: EventType.Click,
  keydown: EventType.KeyDown,
  keyup: EventType.KeyUp,
  input: EventType.Input,
  focus: EventType.Focus,
  blur: EventType.Blur,
  copy: EventType.Copy,
  cut: EventType.Cut,
  paste: EventType.Paste,
};

export const EVENT_TYPE_TO_NAME: Record<number, EventName> = {
  [EventType.MouseMove]: 'mousemove',
  [EventType.MouseDown]: 'mousedown',
  [EventType.MouseUp]: 'mouseup',
  [EventType.Click]: 'click',
  [EventType.KeyDown]: 'keydown',
  [EventType.KeyUp]: 'keyup',
  [EventType.Input]: 'input',
  [EventType.Focus]: 'focus',
  [EventType.Blur]: 'blur',
  [EventType.Copy]: 'copy',
  [EventType.Cut]: 'cut',
  [EventType.Paste]: 'paste',
};

export const NON_BUBBLING_TYPES: ReadonlySet<EventType> = new Set([
  EventType.Focus,
  EventType.Blur,
]);

function isMouseType(t: EventType): boolean {
  return t >= 0 && t <= 3;
}

function isKeyboardType(t: EventType): boolean {
  return t >= 10 && t <= 11;
}

function isInputType(t: EventType): boolean {
  return t === EventType.Input;
}

function isFocusType(t: EventType): boolean {
  return t === EventType.Focus || t === EventType.Blur;
}

function isClipboardType(t: EventType): boolean {
  return t === EventType.Copy || t === EventType.Cut || t === EventType.Paste;
}

interface InternalFlags {
  _stopped: boolean;
  _stoppedImmediate: boolean;
  _prevented: boolean;
  _phase: EventPhase;
}

export function buildDomEvent(
  type: EventType,
  target: UzNode | null,
  payload: any,
): UzumakiEvent {
  const flags: InternalFlags = {
    _stopped: false,
    _stoppedImmediate: false,
    _prevented: false,
    _phase: EventPhase.None,
  };
  const bubbles = !NON_BUBBLING_TYPES.has(type);

  const base: UzumakiEvent = {
    type,
    target,
    currentTarget: target,
    get eventPhase(): EventPhase {
      return flags._phase;
    },
    bubbles,
    get defaultPrevented(): boolean {
      return flags._prevented;
    },
    stopPropagation() {
      flags._stopped = true;
    },
    stopImmediatePropagation() {
      flags._stopped = true;
      flags._stoppedImmediate = true;
    },
    preventDefault() {
      flags._prevented = true;
    },
  };

  (base as any)._flags = flags;

  if (isMouseType(type)) {
    return Object.assign(base, {
      x: payload?.x ?? 0,
      y: payload?.y ?? 0,
      screenX: payload?.screenX ?? 0,
      screenY: payload?.screenY ?? 0,
      button: payload?.button ?? 0,
      buttons: payload?.buttons ?? 0,
    }) as UzumakiMouseEvent;
  }

  if (isKeyboardType(type)) {
    const mods: number = payload?.modifiers ?? 0;
    return Object.assign(base, {
      key: payload?.key ?? '',
      code: payload?.code ?? '',
      keyCode: payload?.keyCode ?? 0,
      repeat: payload?.repeat ?? false,
      ctrlKey: !!(mods & 1),
      altKey: !!(mods & 2),
      shiftKey: !!(mods & 4),
      metaKey: !!(mods & 8),
    }) as UzumakiKeyboardEvent;
  }

  if (isInputType(type)) {
    return Object.assign(base, {
      value: payload?.value ?? '',
      inputType: payload?.inputType ?? '',
      data: payload?.data ?? null,
    }) as UzumakiInputEvent;
  }

  if (isClipboardType(type)) {
    return Object.assign(base, {
      selectionText: payload?.selectionText ?? null,
      clipboardText: payload?.clipboardText ?? null,
    }) as UzumakiClipboardEvent;
  }

  if (isFocusType(type)) {
    return base as UzumakiFocusEvent;
  }

  return base;
}

export function buildLifecycleEvent(
  type: string,
  payload: any,
): UzumakiEvent | UzumakiResizeEvent {
  const flags: InternalFlags = {
    _stopped: false,
    _stoppedImmediate: false,
    _prevented: false,
    _phase: EventPhase.Target,
  };

  const base: UzumakiEvent = {
    type,
    target: null,
    currentTarget: null,
    eventPhase: EventPhase.Target,
    bubbles: false,
    get defaultPrevented(): boolean {
      return flags._prevented;
    },
    stopPropagation() {
      flags._stopped = true;
    },
    stopImmediatePropagation() {
      flags._stopped = true;
      flags._stoppedImmediate = true;
    },
    preventDefault() {
      flags._prevented = true;
    },
  };

  (base as any)._flags = flags;

  if (type === 'resize') {
    return Object.assign(base, {
      width: payload?.width ?? 0,
      height: payload?.height ?? 0,
    }) as UzumakiResizeEvent;
  }

  return base;
}

/** @internal Reads private flags set by buildDomEvent. */
export function _eventFlags(event: UzumakiEvent): InternalFlags {
  return (event as any)._flags as InternalFlags;
}

/** @internal Set the current phase on an event built by buildDomEvent. */
export function _setEventPhase(event: UzumakiEvent, phase: EventPhase): void {
  (event as any)._flags._phase = phase;
}
