// src/ui/styles.rs (最终完整修复版)

use egui::{style::Visuals, Color32, FontId, RichText, Rounding, Stroke, TextStyle};

// 主题色彩定义 (优化配色方案)
pub const PRIMARY_GREEN: Color32 = Color32::from_rgb(34, 139, 34);
#[allow(dead_code)]
pub const LIGHT_GREEN: Color32 = Color32::from_rgb(144, 238, 144);
pub const DARK_GREEN: Color32 = Color32::from_rgb(0, 100, 0);
pub const WHITE: Color32 = Color32::WHITE;
pub const LIGHT_GRAY: Color32 = Color32::from_rgb(248, 248, 248);
pub const MEDIUM_GRAY: Color32 = Color32::from_rgb(220, 220, 220);
pub const DARK_GRAY: Color32 = Color32::from_rgb(100, 100, 100);
pub const RED: Color32 = Color32::from_rgb(220, 20, 60);
#[allow(dead_code)]
pub const BLUE: Color32 = Color32::from_rgb(30, 144, 255);

// 文本颜色优化
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(33, 33, 33);
#[allow(dead_code)]
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(66, 66, 66);
#[allow(dead_code)]
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(150, 150, 150);
pub const TEXT_ON_PRIMARY: Color32 = Color32::WHITE;

// 成功和错误色彩
pub const SUCCESS_COLOR: Color32 = PRIMARY_GREEN;
pub const ERROR_COLOR: Color32 = RED;
#[allow(dead_code)]
pub const WARNING_COLOR: Color32 = Color32::from_rgb(255, 193, 7);
#[allow(dead_code)]
pub const INFO_COLOR: Color32 = BLUE;

// 背景色彩优化
pub const BACKGROUND_COLOR: Color32 = Color32::from_rgb(253, 253, 253);
pub const PANEL_BACKGROUND: Color32 = Color32::from_rgb(245, 248, 250);
pub const CARD_BACKGROUND: Color32 = WHITE;
pub const BUTTON_BACKGROUND: Color32 = PRIMARY_GREEN;
pub const BUTTON_HOVER: Color32 = DARK_GREEN;
pub const BUTTON_ACTIVE: Color32 = Color32::from_rgb(0, 80, 0);

// 边框颜色
pub const BORDER_COLOR: Color32 = Color32::from_rgb(229, 231, 235);
#[allow(dead_code)]
pub const BORDER_FOCUS: Color32 = PRIMARY_GREEN;

/// 应用统一的UI样式
pub fn apply_custom_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let visuals = &mut style.visuals;

    *visuals = Visuals::light();
    visuals.override_text_color = Some(TEXT_PRIMARY);

    let widget_visuals = &mut visuals.widgets;
    widget_visuals.inactive.bg_fill = BUTTON_BACKGROUND;
    widget_visuals.inactive.fg_stroke = Stroke::new(1.0, TEXT_ON_PRIMARY);
    widget_visuals.inactive.bg_stroke = Stroke::new(1.0, BUTTON_BACKGROUND);
    widget_visuals.inactive.rounding = Rounding::same(8.0);

    widget_visuals.hovered.bg_fill = BUTTON_HOVER;
    widget_visuals.hovered.fg_stroke = Stroke::new(1.0, TEXT_ON_PRIMARY);
    widget_visuals.hovered.bg_stroke = Stroke::new(1.0, BUTTON_HOVER);
    widget_visuals.hovered.rounding = Rounding::same(8.0);

    widget_visuals.active.bg_fill = BUTTON_ACTIVE;
    widget_visuals.active.fg_stroke = Stroke::new(1.0, TEXT_ON_PRIMARY);
    widget_visuals.active.bg_stroke = Stroke::new(1.0, BUTTON_ACTIVE);
    widget_visuals.active.rounding = Rounding::same(8.0);

    widget_visuals.open.bg_fill = CARD_BACKGROUND;
    widget_visuals.open.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    widget_visuals.open.bg_stroke = Stroke::new(1.0, BORDER_COLOR);
    widget_visuals.open.rounding = Rounding::same(8.0);

    visuals.panel_fill = PANEL_BACKGROUND;
    visuals.window_fill = BACKGROUND_COLOR;
    visuals.extreme_bg_color = CARD_BACKGROUND;

    // 这是修复后的关键行
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(34, 139, 34, 80);
    visuals.selection.stroke = Stroke::new(1.0, PRIMARY_GREEN);

    visuals.window_rounding = Rounding::same(12.0);
    visuals.menu_rounding = Rounding::same(8.0);

    visuals.window_shadow.color = Color32::from_rgba_unmultiplied(0, 0, 0, 20);
    visuals.popup_shadow.color = Color32::from_rgba_unmultiplied(0, 0, 0, 15);

    style.text_styles.insert(TextStyle::Heading, FontId::new(24.0, egui::FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Body, FontId::new(14.0, egui::FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Button, FontId::new(14.0, egui::FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Small, FontId::new(12.0, egui::FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Monospace, FontId::new(12.0, egui::FontFamily::Monospace));

    style.spacing.button_padding = egui::Vec2::new(12.0, 8.0);
    style.spacing.item_spacing = egui::Vec2::new(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12.0);
    style.spacing.menu_margin = egui::Margin::same(8.0);

    ctx.set_style(style);
}

// ... 文件的其余部分保持不变 ...

pub fn success_text(text: &str) -> RichText {
    RichText::new(text).color(SUCCESS_COLOR).strong()
}

pub fn error_text(text: &str) -> RichText {
    RichText::new(text).color(ERROR_COLOR).strong()
}

#[allow(dead_code)]
pub fn warning_text(text: &str) -> RichText {
    RichText::new(text).color(WARNING_COLOR).strong()
}

#[allow(dead_code)]
pub fn info_text(text: &str) -> RichText {
    RichText::new(text).color(INFO_COLOR)
}

pub fn heading_text(text: &str) -> RichText {
    RichText::new(text)
        .size(22.0)
        .color(PRIMARY_GREEN)
        .strong()
}

pub fn subheading_text(text: &str) -> RichText {
    RichText::new(text)
        .size(16.0)
        .color(TEXT_PRIMARY)
        .strong()
}

pub fn emphasis_text(text: &str) -> RichText {
    RichText::new(text).strong().color(TEXT_PRIMARY)
}

#[allow(dead_code)]
pub fn secondary_text(text: &str) -> RichText {
    RichText::new(text).color(TEXT_SECONDARY)
}

#[allow(dead_code)]
pub fn disabled_text(text: &str) -> RichText {
    RichText::new(text).color(TEXT_DISABLED)
}

#[allow(dead_code)]
pub fn primary_button_text(text: &str) -> RichText {
    RichText::new(text).color(TEXT_ON_PRIMARY).strong()
}

#[allow(dead_code)]
pub fn link_text(text: &str) -> RichText {
    RichText::new(text).color(BLUE).underline()
}

#[allow(dead_code)]
pub fn draw_card<R>(
    ui: &mut egui::Ui,
    title: Option<&str>,
    content: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let card_frame = egui::Frame::default()
        .fill(CARD_BACKGROUND)
        .stroke(Stroke::new(1.0, BORDER_COLOR))
        .rounding(Rounding::same(10.0))
        .inner_margin(egui::Margin::same(16.0));

    card_frame
        .show(ui, |ui| {
            if let Some(title) = title {
                ui.label(subheading_text(title));
                ui.add_space(8.0);
            }
            content(ui)
        })
        .inner
}

#[allow(dead_code)]
pub fn icon_button(ui: &mut egui::Ui, icon: &str, text: &str) -> egui::Response {
    let button_text = format!("{} {}", icon, text);
    ui.button(primary_button_text(&button_text))
}

#[allow(dead_code)]
pub fn status_indicator(ui: &mut egui::Ui, status: &str, is_success: bool) {
    let (color, icon) = if is_success {
        (SUCCESS_COLOR, "✓")
    } else {
        (ERROR_COLOR, "✗")
    };

    ui.horizontal(|ui| {
        ui.label(RichText::new(icon).color(color).strong());
        ui.label(RichText::new(status).color(color));
    });
}

#[allow(dead_code)]
pub fn styled_progress_bar(ui: &mut egui::Ui, progress: f32, text: Option<&str>) {
    let progress_bar = egui::ProgressBar::new(progress)
        .fill(PRIMARY_GREEN)
        .animate(true)
        .rounding(Rounding::same(4.0));

    if let Some(text) = text {
        ui.add(progress_bar.text(text));
    } else {
        ui.add(progress_bar.text(format!("{:.0}%", progress * 100.0)));
    }
}