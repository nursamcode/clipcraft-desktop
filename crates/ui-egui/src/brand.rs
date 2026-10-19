//! ClipCraft Studio brand wordmark and icons rendered procedurally in egui.

use egui::{Color32, FontId, Pos2, Rect, Stroke, Vec2, pos2, vec2};

/// Draw the ClipCraft Studio wordmark `height` points tall with its left edge at `left_center`.
/// Returns the drawn rect.
pub fn paint_wordmark(ui: &egui::Ui, left_center: Pos2, height: f32, _dark: bool) -> Option<Rect> {
    let p = ui.painter();
    
    // Draw small scissors/clip icon badge
    let icon_size = height * 0.9;
    let icon_rect = Rect::from_center_size(
        pos2(left_center.x + icon_size * 0.5, left_center.y),
        vec2(icon_size, icon_size)
    );
    
    // Badge background (purple-violet gradient feel)
    p.rect_filled(icon_rect, 4.0, Color32::from_rgb(124, 58, 237));
    
    // Icon lines (clip symbol)
    let center = icon_rect.center();
    let r = icon_size * 0.25;
    p.circle_stroke(pos2(center.x - r * 0.6, center.y - r * 0.4), r * 0.7, Stroke::new(1.5, Color32::WHITE));
    p.circle_stroke(pos2(center.x - r * 0.6, center.y + r * 0.4), r * 0.7, Stroke::new(1.5, Color32::WHITE));
    p.line_segment(
        [pos2(center.x - r * 0.2, center.y - r * 0.3), pos2(center.x + r * 0.8, center.y + r * 0.5)],
        Stroke::new(1.8, Color32::WHITE)
    );
    p.line_segment(
        [pos2(center.x - r * 0.2, center.y + r * 0.3), pos2(center.x + r * 0.8, center.y - r * 0.5)],
        Stroke::new(1.8, Color32::WHITE)
    );

    // Text "ClipCraft" in bold white + "Studio" in light purple
    let text_x = icon_rect.max.x + 8.0;
    let font_bold = FontId::proportional(height * 0.75);
    let galley_clip = p.layout_no_wrap("ClipCraft ".to_string(), font_bold, Color32::WHITE);
    let clip_w = galley_clip.size().x;
    p.galley(pos2(text_x, left_center.y - galley_clip.size().y * 0.5), galley_clip, Color32::WHITE);

    let font_regular = FontId::proportional(height * 0.70);
    let galley_studio = p.layout_no_wrap("Studio".to_string(), font_regular, Color32::from_rgb(167, 139, 250));
    let studio_w = galley_studio.size().x;
    p.galley(pos2(text_x + clip_w, left_center.y - galley_studio.size().y * 0.5), galley_studio, Color32::from_rgb(167, 139, 250));

    let total_w = icon_size + 8.0 + clip_w + studio_w;
    let bounding = Rect::from_min_size(pos2(left_center.x, left_center.y - height * 0.5), vec2(total_w, height));
    Some(bounding)
}
