// 声明所有模块
mod app;
mod converter;
mod ui;
mod utils;

// 引入我们需要的模块
use app::ImageConverterApp;
use eframe::egui;
use egui::IconData; // 直接从 egui 引入 IconData
use image::GenericImageView;
use ui::styles;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "图片格式转换工具 (作者: IceCod)",
        options,
        Box::new(|cc| {
            styles::apply_custom_style(&cc.egui_ctx);
            setup_custom_fonts(&cc.egui_ctx);
            Box::new(ImageConverterApp::new(cc))
        }),
    )
}

/// 设置自定义字体，支持中文显示
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "noto_sans_sc".to_owned(),
        egui::FontData::from_static(include_bytes!("../resources/NotoSansSC-Regular.ttf")),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans_sc".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("noto_sans_sc".to_owned());
    ctx.set_fonts(fonts);
}

/// 从文件加载图标数据的辅助函数 (更稳健的版本)
fn load_icon() -> IconData {
    // 我们用一个闭包和Result来优雅地处理可能发生的任何错误
    let load_result = (|| -> Result<IconData, Box<dyn std::error::Error>> {
        let image_bytes = include_bytes!("../resources/icon.png");
        let image = image::load_from_memory(image_bytes)?;
        let (width, height) = image.dimensions();
        let rgba = image.into_rgba8().into_raw();
        Ok(IconData { rgba, width, height })
    })();

    // 如果加载成功，就使用我们的图标；如果失败，就打印警告并使用默认图标
    match load_result {
        Ok(icon) => icon,
        Err(e) => {
            log::warn!("加载应用图标失败: {}. 将使用默认图标.", e);
            IconData::default()
        }
    }
}
