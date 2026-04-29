import type { EventHandlerMap, UzumakiInputEvent } from '../events';
import type { Window } from '../window';
import { UzElement } from './base';

export interface InputChangeEvent {
  readonly value: string;
}

export interface InputEventMap extends EventHandlerMap {
  /**
   * Fires before a pending text change is applied. Call `preventDefault()` to
   * cancel the change (e.g. for input filtering). Native may not emit this for
   * every modification source yet.
   */
  beforeinput: UzumakiInputEvent;
}

export class UzInputElement extends UzElement<InputEventMap> {
  constructor(window: Window) {
    super('input', window);
    this.on('input', (event) => {
      this._emitter.emit('change', event);
    });
  }

  get value(): string {
    return String(this.getAttribute('value') ?? '');
  }
}
