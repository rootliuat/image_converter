// 测试PDF转换功能

use std::path::PathBuf;

fn main() {
    println!("🧪 测试PDF转换功能...");

    // 检查是否存在测试图片
    let test_image = PathBuf::from("test_image.png");
    if test_image.exists() {
        println!("✅ 找到测试图片: {}", test_image.display());

        // 测试1: 保持宽高比（推荐）
        let pdf_config_aspect = image_converter::converter::image_to_pdf::PdfConfig {
            output_path: PathBuf::from("test_aspect_ratio_output.pdf"),
            preserve_original_size: true, // 保持宽高比，图片不变形
            page_orientation: image_converter::converter::image_to_pdf::PageOrientation::Auto,
            image_quality: 90,
            one_image_per_page: true,
            dpi: 300.0,
            margin_mm: 0.0,
            auto_rotate: true,
            page_mode: image_converter::converter::image_to_pdf::PageMode::AdaptiveSize,
        };

        println!("🔄 测试1: 保持宽高比模式...");
        match image_converter::converter::image_to_pdf::ImageToPdfConverter::convert_single_image(&test_image, &pdf_config_aspect) {
            Ok(()) => {
                println!("✅ 保持宽高比PDF转换成功！输出文件: test_aspect_ratio_output.pdf");
            },
            Err(e) => {
                println!("❌ 保持宽高比PDF转换失败: {}", e);
            }
        }

        // 测试2: 拉伸填满（可能变形）
        let pdf_config_stretch = image_converter::converter::image_to_pdf::PdfConfig {
            output_path: PathBuf::from("test_stretch_output.pdf"),
            preserve_original_size: false, // 拉伸填满页面，可能变形
            page_orientation: image_converter::converter::image_to_pdf::PageOrientation::Auto,
            image_quality: 90,
            one_image_per_page: true,
            dpi: 300.0,
            margin_mm: 0.0,
            auto_rotate: true,
            page_mode: image_converter::converter::image_to_pdf::PageMode::AdaptiveSize,
        };

        println!("🔄 测试2: 拉伸填满模式...");
        match image_converter::converter::image_to_pdf::ImageToPdfConverter::convert_single_image(&test_image, &pdf_config_stretch) {
            Ok(()) => {
                println!("✅ 拉伸填满PDF转换成功！输出文件: test_stretch_output.pdf");
            },
            Err(e) => {
                println!("❌ 拉伸填满PDF转换失败: {}", e);
            }
        }
    } else {
        println!("⚠️  未找到测试图片，跳过测试");
        println!("💡 请在项目根目录放置 test_image.png 文件进行测试");
    }
}