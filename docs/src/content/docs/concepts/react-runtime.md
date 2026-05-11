---
title: React in Uzumaki
description: How React fits on top of Uzumaki elements.
---

`uzumaki-react` is the adapter that lets React manage Uzumaki elements. You write components and hooks the way you already do — `uzumaki-react` keeps the live element tree in sync with your render output, and Uzumaki paints the result.

If you have used React Native, this is the same arrangement: React is the programming model, the building blocks are not HTML.

## JSX Setup

Use React's JSX runtime with Uzumaki's JSX source:

```json
{
  "compilerOptions": {
    "jsx": "react-jsx",
    "jsxImportSource": "uzumaki-react",
    "types": ["uzumaki-types"]
  }
}
```

`uzumaki-types` teaches TypeScript about the built-in `uzumaki` module and the JSX elements Uzumaki understands.

## The Elements You Can Use

| Element      | For                                                |
| ------------ | -------------------------------------------------- |
| `<view>`     | Layout, grouping, backgrounds, borders, scrolling. |
| `<text>`     | Inline text run (other elements are block-level).  |
| `<button>`   | Pressable content.                                 |
| `<input>`    | Text input.                                        |
| `<checkbox>` | Boolean input.                                     |
| `<image>`    | Local, bundled, or remote image.                   |

DOM tags like `<div>`, `<span>`, and `<img>` are not recognized. If a tag is not in this list (or in your editor's autocomplete), it is not supported.

## Props Instead of CSS

Style is set on the element directly:

```tsx
<view display="flex" flexDir="row" items="center" gap={10} p={12}>
  <text fontWeight={700}>Inbox</text>
</view>
```

No class names, no stylesheet, no DOM attributes. The full prop list is in [Props](/reference/props/).

## Refs Point to Uzumaki Elements

Refs give you the underlying element instance, with imperative methods like `focus()`:

```tsx
import { useRef } from 'react';
import type { UzInputElement } from 'uzumaki';

function SearchBox() {
  const inputRef = useRef<UzInputElement>(null);

  return (
    <view>
      <input ref={inputRef} placeholder="Search" />
      <button onClick={() => inputRef.current?.focus()}>
        <text>Focus search</text>
      </button>
    </view>
  );
}
```

## Finding What's Supported

If you are unsure whether an element or prop exists, hover it in your editor — `uzumaki-types` is the source of truth. The same set is documented in [Elements](/reference/elements/) and [Props](/reference/props/).
