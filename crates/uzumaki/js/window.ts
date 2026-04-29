import core, { type CoreWindow } from './core';
import { UzTextNode } from './node';
import { Element } from './elements/element';
import { UzElement } from './elements/base';
import { UzRootElement } from './elements/root';
import { UzImageElement } from './elements/image';
import { UzInputElement } from './elements/input';
import { UzCheckboxElement } from './elements/checkbox';
import { EventEmitter, type ListenerOptions } from './event-emitter';
import {
  buildLifecycleEvent,
  type WindowEventMap,
  type WindowEventName,
  type WindowEventHandler,
} from './events';
import { clearWindowNodes } from './registry';

const windowsByLabel = new Map<string, Window>();
const windowsById = new Map<number, Window>();

export interface WindowAttributes {
  width: number;
  height: number;
  title: string;
  rootStyles: Record<string, unknown>;
}

export class Window {
  private _id: number;
  private _native: CoreWindow;
  private _label: string;
  private _title: string;
  private _width: number;
  private _height: number;
  private _remBase: number = 16;
  private _disposed: boolean = false;
  private _disposables: (() => void)[] = [];
  private _root: UzRootElement | null = null;
  /** @internal Used by the dispatcher and runtime glue. */
  readonly _emitter: EventEmitter<WindowEventMap> = new EventEmitter();

  constructor(
    label: string,
    {
      width = 800,
      height = 600,
      title = 'uzumaki',
      rootStyles,
    }: Partial<WindowAttributes> = {},
  ) {
    const existing = windowsByLabel.get(label);
    if (existing) {
      throw new Error(`Window with label ${label} already exists`);
    }

    this._width = width;
    this._height = height;
    this._label = label;
    this._title = title;
    this._native = core.createWindow({ width, height, title });
    this._id = this._native.id;
    if (rootStyles) {
      const root = this.root;
      for (const [key, value] of Object.entries(rootStyles)) {
        if (value != null) root.setAttribute(key, value);
      }
    }
    windowsByLabel.set(label, this);
    windowsById.set(this._id, this);
  }

  close() {
    this._emitter._clear();
    windowsByLabel.delete(this._label);
    windowsById.delete(this._id);
    this._native.close();
  }

  addDisposable(cb: () => void): void {
    this._disposables.push(cb);
  }

  static _getById(id: number): Window | undefined {
    return windowsById.get(id);
  }

  setSize(width: number, height: number) {
    this._width = width;
    this._height = height;
  }

  get scaleFactor(): number {
    return this._native.scaleFactor ?? 1;
  }

  get innerWidth(): number {
    return this._native.innerWidth ?? this._width;
  }

  get innerHeight(): number {
    return this._native.innerHeight ?? this._height;
  }

  get title(): string {
    return this._native.title ?? this._title;
  }

  get label(): string {
    return this._label;
  }

  get id(): number {
    return this._id;
  }

  get root(): UzRootElement {
    if (!this._root) {
      this._root = new UzRootElement(this);
    }
    return this._root;
  }

  createElement(type: string): Element<any> {
    if (type === 'image') return new UzImageElement(this);
    if (type === 'input') return new UzInputElement(this);
    if (type === 'checkbox') return new UzCheckboxElement(this);
    return new UzElement(type, this);
  }

  createTextNode(text: string): UzTextNode {
    return new UzTextNode(this, text);
  }

  get isDisposed(): boolean {
    return this._disposed;
  }

  get remBase(): number {
    return this._native.remBase ?? this._remBase;
  }

  set remBase(value: number) {
    this._remBase = value;
    this._native.remBase = value;
  }

  on<K extends WindowEventName>(
    eventName: K,
    handler: WindowEventHandler<K>,
    options?: ListenerOptions,
  ): void {
    this._emitter.on(eventName, handler, options);
  }

  off<K extends WindowEventName>(
    eventName: K,
    handler: WindowEventHandler<K>,
    options?: ListenerOptions,
  ): void {
    this._emitter.off(eventName, handler, options);
  }

  /** @internal Fire a lifecycle event (load/close/resize). */
  _dispatchLifecycle(
    name: 'load' | 'close' | 'resize',
    payload?: any,
  ): boolean {
    const event = buildLifecycleEvent(name, payload);
    this._emitter.emit(name, event as any);
    return event.defaultPrevented;
  }
}

/** @internal Called when the native window is destroyed. */
export function disposeWindow(_window: Window) {
  const window = _window as never as {
    id: number;
    label: string;
    _disposed: boolean;
    _disposables: (() => void)[];
    _emitter: { _clear(): void };
  };

  window._disposed = true;
  for (const cb of window._disposables) {
    cb();
  }
  window._disposables = [];
  window._emitter._clear();
  clearWindowNodes(window.id);
  windowsByLabel.delete(window.label);
  windowsById.delete(window.id);
}
