import type { Window } from '../window';
import { UzElement } from './base';

export interface CheckboxChangeEvent {
  readonly value: boolean;
}

export class UzCheckboxElement extends UzElement {
  constructor(window: Window) {
    super('checkbox', window);
    this.on('input', (event) => {
      this._emitter.emit('change', event);
    });
  }

  get checked(): boolean {
    const checked = this.getAttribute('checked');
    return checked === true || checked === 'true';
  }
}
