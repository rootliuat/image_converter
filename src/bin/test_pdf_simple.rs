// 简单的PDF转换测试

use image_converter::converter::image_to_pdf::{ImageToPdfConverter, PdfConfig, PageOrientation, PageMode};
use std::path::PathBuf;

fn main() {
    println!("🔄 开始PDF转换测试...");

    // 检查测试图片
    let test_image = "test_position_正中央.png";
    if !std::path::Path::new(test_image).exists() {
        println!("❌ 未找到测试图片: {}", test_image);
        return;
    }

    println!("✅ 找到测试图片: {}", test_image);

    // 创建PDF配置
    let config = PdfConfig {
        output_path: PathBuf::from("test_simple_output.pdf"),
        preserve_original_size: true,
        page_orientation: PageOrientation::Auto,
        image_quality: 90,
        one_image_per_page: true,
        dpi: 300.0,
        margin_mm: 0.0,
        auto_rotate: true,
        page_mode: PageMode::AdaptiveSize,
    };

    println!("📄 PDF配置:");
    println!("  - 输出路径: {:?}", config.output_path);
    println!("  - 保持原始尺寸: {}", config.preserve_original_size);
    println!("  - 图片质量: {}%", config.image_quality);

    // 执行转换
    match ImageToPdfConverter::convert_single_image(&PathBuf::from(test_image), &config) {
        Ok(()) => {
            println!("🎉 转换成功！");
            println!("📄 输出文件: {}", config.output_path.display());

            // 检查文件是否存在
            if config.output_path.exists() {
                let metadata = std::fs::metadata(&config.output_path).unwrap();
                println!("📊 文件大小: {} bytes", metadata.len());
            } else {
                println!("⚠️  输出文件不存在");
            }
        },
        Err(e) => {
            println!("❌ 转换失败: {}", e);
            println!("🔍 错误详情: {:?}", e);
        }
    }
}