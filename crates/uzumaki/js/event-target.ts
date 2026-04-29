export interface ListenerOptions {
  capture?: boolean;
}

export interface EmitterEntry<F extends Function = Function> {
  handler: F;
  capture: boolean;
}

export class UzEventTarget<M extends Record<string, any>> {
  private _entries: Map<keyof M, EmitterEntry[]> = new Map();

  on<K extends keyof M>(
    name: K,
    handler: (event: M[K]) => void,
    options?: ListenerOptions,
  ): void {
    const capture = options?.capture ?? false;
    let list = this._entries.get(name);
    if (!list) {
      list = [];
      this._entries.set(name, list);
    }
    list.push({ handler, capture });
  }

  off<K extends keyof M>(
    name: K,
    handler: (event: M[K]) => void,
    options?: ListenerOptions,
  ): void {
    const capture = options?.capture ?? false;
    const list = this._entries.get(name);
    if (!list) return;
    const idx = list.findIndex(
      (e) => e.handler === handler && e.capture === capture,
    );
    if (idx !== -1) list.splice(idx, 1);
    if (list.length === 0) this._entries.delete(name);
  }

  emit<K extends keyof M>(name: K, event: M[K]): void {
    const list = this._entries.get(name);
    if (!list || list.length === 0) return;
    // snapshot: a handler may call off() during dispatch
    // eslint-disable-next-line unicorn/no-useless-spread
    for (const entry of [...list]) {
      entry.handler(event);
    }
  }

  /** @internal Used by the dispatcher to filter handlers by phase. */
  _listeners<K extends keyof M>(name: K): readonly EmitterEntry[] | undefined {
    return this._entries.get(name);
  }

  /** @internal */
  _hasAny(): boolean {
    return this._entries.size > 0;
  }

  _listenerCount<K extends keyof M>(name: K): number {
    return this._entries.get(name)?.length ?? 0;
  }

  /** @internal */
  _clear(): void {
    this._entries.clear();
  }
}
