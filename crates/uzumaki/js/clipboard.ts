import core from './core';

export const Clipboard = {
  readText(): string | null {
    return core.readClipboardText();
  },
  writeText(text: string): boolean {
    return core.writeClipboardText(text);
  },
};
