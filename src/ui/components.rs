use crate::ui::styles;
use egui::{Color32, Response, RichText, Ui, Vec2};

/// 创建一个带有自定义样式的主要按钮
#[allow(dead_code)]
pub fn primary_button(ui: &mut Ui, text: &str) -> Response {
    let button = egui::Button::new(RichText::new(text).size(14.0))
        .min_size(Vec2::new(100.0, 35.0));
    ui.add(button)
}

/// 创建一个带有自定义样式的次要按钮
pub fn secondary_button(ui: &mut Ui, text: &str) -> Response {
    let button = egui::Button::new(RichText::new(text).size(12.0))
        .fill(styles::MEDIUM_GRAY)
        .min_size(Vec2::new(80.0, 30.0));
    ui.add(button)
}

/// 创建一个危险操作按钮（红色）
#[allow(dead_code)]
pub fn danger_button(ui: &mut Ui, text: &str) -> Response {
    let button = egui::Button::new(RichText::new(text).size(12.0).color(Color32::WHITE))
        .fill(styles::ERROR_COLOR)
        .min_size(Vec2::new(80.0, 30.0));
    ui.add(button)
}

// ... 其他您设计的按钮函数，如 success_button, icon_button ...

/// 创建一个带有百分比显示的进度条
pub fn progress_bar_with_text(ui: &mut Ui, progress: f32, text: &str) -> Response {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(text);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{:.0}%", progress * 100.0));
            });
        });
        
        let progress_bar = egui::ProgressBar::new(progress)
            .fill(styles::PRIMARY_GREEN)
            .animate(true);
        ui.add(progress_bar)
    }).inner
}

/// 创建一个文件路径显示组件
#[allow(dead_code)]
pub fn file_path_display(ui: &mut Ui, label: &str, path: &mut String, on_browse: impl FnOnce()) -> Response {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        
        let path_response = ui.add(
            egui::TextEdit::singleline(path)
                .desired_width(ui.available_width() - 80.0)
                .hint_text("请选择文件或文件夹路径")
        );
        
        if ui.button("📂 浏览").clicked() {
            on_browse();
        }
        
        path_response
    }).inner
}

/// 创建一个参数设置组件
pub fn parameter_group<R>(
    ui: &mut Ui,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(styles::subheading_text(title));
            ui.add_space(8.0);
            add_contents(ui)
        }).inner
    })
}

/// 创建一个统计信息显示组件
pub fn statistics_display(
    ui: &mut Ui,
    processed: usize,
    failed: usize,
    total: usize,
) {
    ui.horizontal(|ui| {
        ui.label("✅ 成功:");
        ui.label(styles::success_text(&processed.to_string()));
        ui.add_space(20.0);
        ui.label("❌ 失败:");
        ui.label(styles::error_text(&failed.to_string()));
        ui.add_space(20.0);
        ui.label("📁 总计:");
        ui.label(styles::emphasis_text(&total.to_string()));
    });
}

/// 创建一个数值输入框（带单位显示）
pub fn number_input_with_unit(
    ui: &mut Ui,
    label: &str,
    value: &mut u32,
    unit: &str,
    min: u32,
    max: u32,
) -> Response {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        
        let mut temp_value = *value as f64;
        
        // --- 【API修复】egui API 变更 ---
        // 旧API `.range(...)` 已被移除
        // 新API使用 `.clamp_range(...)`
        let response = ui.add(
            egui::DragValue::new(&mut temp_value)
                .clamp_range(min as f64..=max as f64)
                .speed(10.0)
                .suffix(format!(" {}", unit))
        );
        
        if response.changed() {
            *value = temp_value as u32;
        }
        response
    }).inner
}

/// 创建一个文件格式选择器
pub fn format_selector<T: PartialEq + Clone>(
    ui: &mut Ui,
    label: &str,
    current: &mut T,
    options: &[(T, &str)],
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        ui.add_space(10.0);

        for (i, (value, display_name)) in options.iter().enumerate() {
            if i > 0 {
                ui.add_space(5.0);
            }

            // 使用更大的按钮，确保点击区域足够
            let is_selected = *current == *value;
            let button_text = egui::RichText::new(*display_name).size(14.0);

            let button = if is_selected {
                egui::Button::new(button_text)
                    .fill(egui::Color32::from_rgb(70, 130, 180)) // 选中状态的蓝色
                    .min_size([120.0, 35.0].into()) // 增大按钮尺寸
                    .rounding(6.0) // 圆角
            } else {
                egui::Button::new(button_text)
                    .fill(egui::Color32::from_rgb(245, 245, 245)) // 更明显的背景色
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200)))
                    .min_size([120.0, 35.0].into()) // 增大按钮尺寸
                    .rounding(6.0) // 圆角
            };

            let response = ui.add(button);

            // 使用hovered()和clicked()双重检查，确保点击反应更好
            if response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            if response.clicked() && !is_selected {
                *current = value.clone();
                changed = true;
                // 强制UI立即更新
                ui.ctx().request_repaint();
            }
        }
    });
    changed
}

/// 创建一个状态消息显示区域
pub fn status_message_area(ui: &mut Ui, message: &str, is_error: bool) {
    if !message.is_empty() {
        let (fill_color, stroke_color) = if is_error {
            (Color32::from_rgba_unmultiplied(220, 53, 69, 30), styles::ERROR_COLOR)
        } else {
            (styles::LIGHT_GRAY, styles::DARK_GRAY)
        };
        
        egui::Frame::none()
            .fill(fill_color)
            .stroke(egui::Stroke::new(1.0, stroke_color))
            .rounding(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.label(RichText::new(message).color(if is_error { styles::ERROR_COLOR } else { styles::DARK_GRAY }));
            });
    }
}
