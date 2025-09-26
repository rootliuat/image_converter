// 测试简单PDF转换
use std::path::PathBuf;
use image_converter::converter::image_to_pdf::*;

fn main() {
    println!("🧪 测试PDF转换...");

    // 查找测试图片
    let test_images = ["test_image.png", "ui.png", "test_watermark_output.png"];
    let mut found_image = None;

    for img in &test_images {
        let path = PathBuf::from(img);
        if path.exists() {
            found_image = Some(path);
            println!("✅ 找到测试图片: {}", img);
            break;
        }
    }

    if let Some(image_path) = found_image {
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

        println!("🔄 转换图片到PDF...");
        match ImageToPdfConverter::convert_single_image(&image_path, &config) {
            Ok(()) => {
                println!("✅ PDF转换成功！输出文件: test_simple_output.pdf");

                // 检查文件大小
                if let Ok(metadata) = std::fs::metadata("test_simple_output.pdf") {
                    println!("📊 PDF文件大小: {} 字节", metadata.len());
                }
            },
            Err(e) => {
                println!("❌ PDF转换失败: {}", e);
                eprintln!("详细错误: {:?}", e);
            }
        }
    } else {
        println!("⚠️  未找到测试图片，请确保项目目录中有图片文件");
    }
}