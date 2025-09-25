// 菜单栏组件模块
use eframe::egui;
use crate::utils::config::AppMode;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuAction {
    None,
    NewProject,
    OpenFile,
    SaveAs,
    Exit,
    ImageToPdf,
    ImageConverter,
    About,
    Settings,
}

/// 菜单栏状态
pub struct MenuBarState {
    pub current_mode: AppMode,
    pub show_about: bool,
    pub show_settings: bool,
}

impl Default for MenuBarState {
    fn default() -> Self {
        Self {
            current_mode: AppMode::ImageConverter,
            show_about: false,
            show_settings: false,
        }
    }
}

/// 绘制菜单栏并返回用户操作
pub fn draw_menu_bar(ctx: &egui::Context, state: &mut MenuBarState) -> MenuAction {
    let mut action = MenuAction::None;

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            // 文件菜单
            ui.menu_button("文件", |ui| {
                if ui.button("🆕 新建项目").clicked() {
                    action = MenuAction::NewProject;
                    ui.close_menu();
                }

                if ui.button("📂 打开文件").clicked() {
                    action = MenuAction::OpenFile;
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("💾 另存为").clicked() {
                    action = MenuAction::SaveAs;
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("❌ 退出").clicked() {
                    action = MenuAction::Exit;
                    ui.close_menu();
                }
            });

            // 功能菜单
            ui.menu_button("功能", |ui| {
                if ui.button("🖼️ 图片格式转换").clicked() {
                    action = MenuAction::ImageConverter;
                    state.current_mode = AppMode::ImageConverter;
                    ui.close_menu();
                }

                if ui.button("📄 图片转PDF").clicked() {
                    action = MenuAction::ImageToPdf;
                    state.current_mode = AppMode::ImageToPdf;
                    ui.close_menu();
                }
            });

            // 工具菜单
            ui.menu_button("工具", |ui| {
                if ui.button("⚙️ 设置").clicked() {
                    state.show_settings = true;
                    action = MenuAction::Settings;
                    ui.close_menu();
                }
            });

            // 帮助菜单
            ui.menu_button("帮助", |ui| {
                if ui.button("❓ 关于").clicked() {
                    state.show_about = true;
                    action = MenuAction::About;
                    ui.close_menu();
                }
            });

            // 右侧状态显示
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("当前模式: {}", match state.current_mode {
                    AppMode::ImageConverter => "图片转换",
                    AppMode::ImageToPdf => "转换PDF",
                    AppMode::PdfToImage => "PDF转图片",
                    AppMode::PureWatermark => "纯水印",
                }));

                ui.separator();

                ui.label("🛠️ 图片转换工具 v1.0");
            });
        });
    });

    // 显示关于对话框
    if state.show_about {
        draw_about_dialog(ctx, &mut state.show_about);
    }

    // 显示设置对话框
    if state.show_settings {
        draw_settings_dialog(ctx, &mut state.show_settings);
    }

    action
}

/// 绘制关于对话框
fn draw_about_dialog(ctx: &egui::Context, show: &mut bool) {
    let mut open = *show;

    egui::Window::new("关于图片转换工具")
        .open(&mut open)
        .resizable(false)
        .fixed_size([400.0, 300.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // 应用图标（可以用emoji替代）
                ui.label(egui::RichText::new("🖼️").size(64.0));

                ui.add_space(15.0);

                ui.label(egui::RichText::new("图片转换工具")
                    .size(24.0)
                    .color(egui::Color32::from_rgb(70, 130, 180)));

                ui.add_space(10.0);

                ui.label("版本 1.0.0");
                ui.add_space(5.0);
                ui.label("基于 Rust + egui 开发");

                ui.add_space(20.0);

                ui.label("✨ 功能特性：");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        ui.label("• 图片格式转换 (JPEG/PNG/WebP)");
                        ui.label("• 批量处理支持");
                        ui.label("• 图片转PDF功能");
                        ui.label("• 水印添加功能");
                        ui.label("• 高质量压缩算法");
                    });
                });

                ui.add_space(20.0);

                ui.label("© 2024 图片转换工具");

                ui.add_space(15.0);

                if ui.button("确定").clicked() {
                    *show = false;
                }
            });
        });

    *show = open;
}

/// 绘制设置对话框
fn draw_settings_dialog(ctx: &egui::Context, show: &mut bool) {
    let mut open = *show;

    egui::Window::new("设置")
        .open(&mut open)
        .resizable(true)
        .min_size([500.0, 400.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("应用设置");
                ui.separator();

                ui.add_space(10.0);

                // 外观设置
                ui.collapsing("🎨 外观", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("主题：");
                        ui.radio_value(&mut true, true, "明亮");
                        ui.radio_value(&mut true, false, "黑暗");
                    });

                    ui.add_space(5.0);

                    ui.checkbox(&mut true, "启用动画效果");
                    ui.checkbox(&mut true, "显示进度条");
                });

                ui.add_space(10.0);

                // 性能设置
                ui.collapsing("⚡ 性能", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("最大并发线程：");
                        let mut threads = 4;
                        ui.add(egui::Slider::new(&mut threads, 1..=16).text("个"));
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label("内存限制：");
                        let mut memory = 2048;
                        ui.add(egui::Slider::new(&mut memory, 512..=8192).text("MB"));
                    });

                    ui.checkbox(&mut true, "启用SIMD优化");
                    ui.checkbox(&mut true, "启用内存池");
                });

                ui.add_space(10.0);

                // 输出设置
                ui.collapsing("📁 输出", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("默认输出格式：");
                        ui.radio_value(&mut 0, 0, "JPEG");
                        ui.radio_value(&mut 0, 1, "PNG");
                        ui.radio_value(&mut 0, 2, "WebP");
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label("默认质量：");
                        let mut quality = 85;
                        ui.add(egui::Slider::new(&mut quality, 10..=100).text("%"));
                    });

                    ui.checkbox(&mut false, "保持原始文件名");
                    ui.checkbox(&mut true, "转换完成后打开输出目录");
                });

                ui.add_space(20.0);

                // 按钮区域
                ui.horizontal(|ui| {
                    if ui.button("重置为默认").clicked() {
                        // TODO: 重置设置
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("确定").clicked() {
                            *show = false;
                        }

                        if ui.button("取消").clicked() {
                            *show = false;
                        }
                    });
                });
            });
        });

    *show = open;
}