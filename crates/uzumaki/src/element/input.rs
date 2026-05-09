use parley::BoundingBox;
use vello::Scene;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color as VelloColor, Fill};

use crate::input::input_align_offset;
use crate::style::{Bounds, Color, Corners, Edges, TextStyle, UzStyle};
use crate::text::TextRenderer;

pub struct InputRenderInfo {
    pub display_text: String,
    pub placeholder: String,
    pub text_style: TextStyle,
    pub focused: bool,
    pub cursor_rect: Option<BoundingBox>,
    pub selection_rects: Vec<BoundingBox>,
    pub scroll_offset: f32,
    pub scroll_offset_y: f32,
    pub blink_visible: bool,
    pub multiline: bool,
    pub layout_height: f32,
    pub preedit: Option<PreeditRenderInfo>,
}

pub struct PreeditRenderInfo {
    pub text: String,
    pub cursor_x: f32,
    pub width: f32,
}

/// Paint an input element with its text, selection highlight, and cursor.
pub fn paint_input(
    scene: &mut Scene,
    text_renderer: &mut TextRenderer,
    bounds: Bounds,
    style: &UzStyle,
    input: &InputRenderInfo,
    transform: Affine,
) {
    let pad_l = style.padding.left as f64;
    let pad_r = style.padding.right as f64;
    let pad_t = style.padding.top as f64;
    let pad_b = style.padding.bottom as f64;
    let content_x = pad_l;
    let content_y = pad_t;
    let content_w = (bounds.width - pad_l - pad_r).max(0.0);
    let content_h = (bounds.height - pad_t - pad_b).max(0.0);

    let mut paint_style = style.clone();
    if !paint_style.border_widths.any_nonzero() {
        paint_style.border_widths = Edges::all(1.0);
    }
    if paint_style.border_color.is_none() {
        paint_style.border_color = Some(Color::rgba(60, 60, 60, 255));
    }
    if paint_style.background.is_none() {
        paint_style.background = Some(Color::rgba(30, 30, 30, 255));
    }
    if !paint_style.corner_radii.any_nonzero() {
        paint_style.corner_radii = Corners::uniform(4.0);
    }

    paint_style.paint(bounds, scene, transform, |_| {});

    // Clip to text area
    let clip_rect = Rect::new(
        content_x,
        content_y,
        content_x + content_w,
        content_y + content_h,
    );
    scene.push_clip_layer(Fill::NonZero, transform, &clip_rect);

    let is_empty = input.display_text.is_empty();
    let line_height = (input.text_style.font_size * input.text_style.line_height).round();
    let scroll_y = input.scroll_offset_y as f64;

    // Browser-style horizontal alignment for single-line inputs: when the text
    // fits the content box, shift it by `align_offset`; once it overflows the
    // offset is 0 and `scroll_offset` keeps the cursor in view. Multiline
    // alignment is baked into the editor's layout, so the offset is 0 there.
    let align_offset = if input.multiline || is_empty {
        0.0
    } else {
        let (natural_w, _) =
            text_renderer.measure_text(&input.display_text, &input.text_style, None, None);
        input_align_offset(content_w as f32, natural_w, input.text_style.text_align) as f64
    };
    let single_x_shift = align_offset - input.scroll_offset as f64;

    // Placeholder
    if is_empty && !input.placeholder.is_empty() {
        let py = if input.multiline {
            content_y as f32
        } else {
            content_y as f32 + ((content_h as f32 - line_height) / 2.0).max(0.0)
        };
        // Placeholder respects text-align: pass the content width so the
        // editor's alignment is applied, single-line or multiline.
        text_renderer.draw_text(
            scene,
            &input.placeholder,
            &input.text_style,
            Some(content_w as f32),
            (content_x as f32, py),
            VelloColor::from_rgba8(128, 128, 128, 255),
            transform,
        );
    }

    if !is_empty {
        let x_shift = if input.multiline { 0.0 } else { single_x_shift };

        // Selection highlights
        if input.focused && !input.selection_rects.is_empty() {
            let sel_color = VelloColor::from_rgba8(56, 121, 185, 128);
            let oy = if input.multiline {
                content_y - scroll_y
            } else {
                content_y + ((content_h - line_height as f64) / 2.0).max(0.0)
            };
            for rect in &input.selection_rects {
                let x1 = content_x + x_shift + rect.x0;
                let x2 = content_x + x_shift + rect.x1;
                let y1 = oy + rect.y0;
                let y2 = oy + rect.y1;
                scene.fill(
                    Fill::NonZero,
                    transform,
                    sel_color,
                    None,
                    &Rect::new(x1, y1, x2, y2),
                );
            }
        }

        // Text. For single-line we draw with no wrap width so the layout
        // doesn't apply any alignment of its own — we position via x_shift.
        // For multiline the editor already aligned within content_w.
        let ty = if input.multiline {
            (content_y - scroll_y) as f32
        } else {
            content_y as f32 + ((content_h as f32 - line_height) / 2.0).max(0.0)
        };
        let tx = (content_x + x_shift) as f32;
        let tw = if input.multiline {
            Some(content_w as f32)
        } else {
            None
        };
        text_renderer.draw_text(
            scene,
            &input.display_text,
            &input.text_style,
            tw,
            (tx, ty),
            input.text_style.color.to_vello(),
            transform,
        );
    }

    // Preedit (IME composition text)
    if let Some(preedit) = &input.preedit
        && let Some(cr) = &input.cursor_rect
    {
        let x_shift = if input.multiline { 0.0 } else { single_x_shift };
        let oy = if input.multiline {
            content_y - scroll_y
        } else {
            content_y + ((content_h - line_height as f64) / 2.0).max(0.0)
        };
        let px = content_x + x_shift + cr.x0;
        let py = oy + cr.y0;
        let preedit_h = cr.y1 - cr.y0;

        // Background highlight for preedit
        let preedit_bg = VelloColor::from_rgba8(50, 50, 60, 180);
        let preedit_rect = Rect::new(px, py, px + preedit.width as f64, py + preedit_h);
        scene.fill(Fill::NonZero, transform, preedit_bg, None, &preedit_rect);

        // Preedit text — natural single-line layout, positioned manually.
        text_renderer.draw_text(
            scene,
            &preedit.text,
            &input.text_style,
            None,
            (px as f32, py as f32),
            input.text_style.color.to_vello(),
            transform,
        );

        // Underline
        let underline_y = py + preedit_h - 1.0;
        let underline = Rect::new(
            px,
            underline_y,
            px + preedit.width as f64,
            underline_y + 1.0,
        );
        scene.fill(
            Fill::NonZero,
            transform,
            VelloColor::from_rgba8(180, 180, 180, 255),
            None,
            &underline,
        );
    }

    // Cursor (hide during preedit)
    if input.focused
        && input.blink_visible
        && input.preedit.is_none()
        && let Some(cr) = &input.cursor_rect
    {
        let x_shift = if input.multiline { 0.0 } else { single_x_shift };
        let oy = if input.multiline {
            content_y - scroll_y
        } else {
            content_y + ((content_h - line_height as f64) / 2.0).max(0.0)
        };
        let cx = content_x + x_shift + cr.x0;
        let cy = oy + cr.y0;
        let cursor_rect = Rect::new(cx, cy + 2.0, cx + 1.5, cy + cr.y1 - cr.y0 - 2.0);
        scene.fill(
            Fill::NonZero,
            transform,
            VelloColor::from_rgba8(212, 212, 212, 255),
            None,
            &cursor_rect,
        );
    }

    scene.pop_layer();
}
