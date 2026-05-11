---
title: Props
description: Layout, style, interaction, and state props.
---

Style props are set directly on Uzumaki elements. Most props are shared across elements.

## Values

### Lengths

Numbers are logical pixels. Strings can carry a unit or a keyword.

```tsx
<view p={16} fontSize="1.25rem" w="100%" h="full" m="auto" />
```

| Form       | Example                   | Where it works                          |
| ---------- | ------------------------- | --------------------------------------- |
| Number     | `p={16}`, `fontSize={14}` | Any length prop. Unit is pixels.        |
| `"<n>"`    | `w="200"`                 | Same as a number.                       |
| `"<n>px"`  | `w="200px"`               | Same as a number.                       |
| `"<n>rem"` | `fontSize="1.25rem"`      | Scaled by `window.remBase`.             |
| `"<n>%"`   | `w="50%"`                 | Sizing and inset props.                 |
| `"full"`   | `h="full"`                | Sizing and inset props. Same as `100%`. |
| `"auto"`   | `m="auto"`, `w="auto"`    | Sizing, inset, and margin.              |

`gap` accepts the same forms as sizing except `auto`.

### Colors

Hex, named CSS colors, and `rgb()` / `rgba()` are all accepted.

```tsx
<text color="#f4f4f5" />
<view bg="slategray" border={1} borderColor="rgba(255, 255, 255, 0.1)" />
```

| Form     | Example                                                       |
| -------- | ------------------------------------------------------------- |
| Hex      | `"#f4f4f5"`, `"#fff"`, `"#0008"`, `"#11223344"`               |
| Named    | `"red"`, `"slategray"`, `"rebeccapurple"`, `"transparent"`, … |
| `rgb()`  | `"rgb(20, 30, 40)"`                                           |
| `rgba()` | `"rgba(20, 30, 40, 0.5)"` or `"rgba(20, 30, 40, 128)"`        |

All standard CSS named colors are supported. Alpha in `rgba()` accepts either a `0`–`1` float or a `0`–`255` integer.

### Booleans

Boolean props accept `true`/`false`. When passed as a string, these values count as `false`: `""`, `"0"`, `"false"`, `"hidden"`, `"none"`, `"no"`, `"off"`. Everything else is `true`.

## Size and Spacing

| Prop                                    | Description               |
| --------------------------------------- | ------------------------- |
| `w`, `h`                                | Width and height.         |
| `minW`, `minH`                          | Minimum width and height. |
| `p`, `px`, `py`, `pt`, `pr`, `pb`, `pl` | Padding.                  |
| `m`, `mx`, `my`, `mt`, `mr`, `mb`, `ml` | Margin.                   |

## Layout

| Prop                             | Values                  |
| -------------------------------- | ----------------------- |
| `display`                        | `flex`, `block`, `none` |
| `position`                       | `relative`, `absolute`  |
| `top`, `right`, `bottom`, `left` | Number or string offset |

## Flex

| Prop         | Values                                                                                                                                  |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------- |
| `flexDir`    | `row`, `col` / `column`, `row-reverse`, `col-reverse`                                                                                   |
| `flexWrap`   | `nowrap` / `no-wrap`, `wrap`, `wrap-reverse`                                                                                            |
| `flex`       | `true`, number, or a `flexDir` string (sets `display:flex` and direction)                                                               |
| `flexGrow`   | Number                                                                                                                                  |
| `flexShrink` | Number                                                                                                                                  |
| `items`      | `start` / `flex-start`, `end` / `flex-end`, `center`, `stretch`, `baseline`                                                             |
| `justify`    | `start` / `flex-start`, `end` / `flex-end`, `center`, `between` / `space-between`, `around` / `space-around`, `evenly` / `space-evenly` |
| `gap`        | Number or string                                                                                                                        |

```tsx
<view display="flex" flexDir="row" items="center" justify="between" gap={12} />
```

`flex` accepts a direction string as a shortcut — `<view flex="col" />` is the same as `<view display="flex" flexDir="col" />`.

## Color and Typography

| Prop          | Values                                               |
| ------------- | ---------------------------------------------------- |
| `bg`, `color` | Hex color                                            |
| `opacity`     | Number or string                                     |
| `visibility`  | `visible`, `hidden`                                  |
| `fontSize`    | Number or string                                     |
| `fontWeight`  | Number (`100`–`900`) or name (see below)             |
| `fontFamily`  | String                                               |
| `textAlign`   | `left`, `center`, `right`, `start`, `end`, `justify` |
| `textWrap`    | `wrap`, `nowrap` / `none`, `anywhere`, `break-word`  |
| `wordBreak`   | `normal`, `break-all`, `keep-all`                    |

`fontWeight` accepts a numeric weight or one of these names:

| Name                                                        | Number |
| ----------------------------------------------------------- | ------ |
| `thin`                                                      | `100`  |
| `extra-light` / `extralight` / `ultra-light` / `ultralight` | `200`  |
| `light`                                                     | `300`  |
| `normal` / `regular`                                        | `400`  |
| `medium`                                                    | `500`  |
| `semi-bold` / `semibold` / `demi-bold` / `demibold`         | `600`  |
| `bold`                                                      | `700`  |
| `extra-bold` / `extrabold` / `ultra-bold` / `ultrabold`     | `800`  |
| `black` / `heavy`                                           | `900`  |

## Borders, Corners, and Outlines

| Prop                                                               | Description                |
| ------------------------------------------------------------------ | -------------------------- |
| `rounded`, `roundedTL`, `roundedTR`, `roundedBR`, `roundedBL`      | Corner radius.             |
| `border`, `borderTop`, `borderRight`, `borderBottom`, `borderLeft` | Border width.              |
| `borderColor`                                                      | Border color.              |
| `outline`, `outlineColor`, `outlineOffset`                         | Focus or emphasis outline. |

## Transforms

```tsx
<view translate={[8, 0]} rotate={-3} scale={1.05} hover:scale={1.08} />
```

| Prop                       | Description                      |
| -------------------------- | -------------------------------- |
| `translate`                | Number, `[x, y]`, or `{ x, y }`. |
| `translateX`, `translateY` | Single-axis translate.           |
| `rotate`                   | Rotation in degrees.             |
| `scale`                    | Number, `[x, y]`, or `{ x, y }`. |
| `scaleX`, `scaleY`         | Single-axis scale.               |

## Interaction

| Prop         | Description                                         |
| ------------ | --------------------------------------------------- |
| `cursor`     | Cursor shown while hovering.                        |
| `focusable`  | Allows a view to receive focus and keyboard events. |
| `selectable` | Allows text inside a view to be selected.           |

## Scrolling

```tsx
<view scrollY h={320} scrollbarWidth={6} scrollbarRadius={999}>
  {rows}
</view>
```

| Prop                           | Description              |
| ------------------------------ | ------------------------ |
| `scroll`, `scrollX`, `scrollY` | Enable scrolling.        |
| `scrollbarWidth`               | Scrollbar thickness.     |
| `scrollbarColor`               | Default scrollbar color. |
| `scrollbarHoverColor`          | Hover color.             |
| `scrollbarActiveColor`         | Active drag color.       |
| `scrollbarRadius`              | Scrollbar radius.        |

## State Variants

Prefix most visual props with `hover:`, `active:`, or `focus:`.

```tsx
<button
  bg="#18181b"
  hover:bg="#27272a"
  active:bg="#3f3f46"
  focus:outline={2}
  focus:outlineColor="#60a5fa"
>
  <text>Open</text>
</button>
```

See [Elements](/reference/elements/) for element-specific props and [Events](/reference/events/) for event handlers.
