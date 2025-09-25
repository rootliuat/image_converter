// 测试PDF转换功能

use std::path::PathBuf;

fn main() {
    println!("🧪 测试PDF转换功能...");

    // 检查是否存在测试图片
    let test_image = PathBuf::from("test_position_正中央.png");
    if test_image.exists() {
        println!("✅ 找到测试图片: {}", test_image.display());

        // 测试图片转PDF功能
        let pdf_config = image_converter::converter::image_to_pdf::PdfConfig {
            output_path: PathBuf::from("test_output.pdf"),
            preserve_original_size: true,
            page_orientation: image_converter::converter::image_to_pdf::PageOrientation::Auto,
            image_quality: 90,
            one_image_per_page: true,
        };

        println!("🔄 开始转换...");
        match image_converter::converter::image_to_pdf::ImageToPdfConverter::convert_single_image(&test_image, &pdf_config) {
            Ok(()) => {
                println!("🎉 PDF转换成功！输出文件: test_output.pdf");
            },
            Err(e) => {
                println!("❌ PDF转换失败: {}", e);
            }
        }
    } else {
        println!("⚠️  未找到测试图片，跳过测试");
        println!("💡 请在项目根目录放置 test_position_正中央.png 文件进行测试");
    }
}