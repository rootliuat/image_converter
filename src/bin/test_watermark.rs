// 水印功能测试

use image_converter::converter::simple_watermark::{
    SimpleWatermarkProcessor, SimpleTextWatermark, WatermarkPosition
};
use image::{Rgba, DynamicImage};

fn main() {
    println!("🎨 开始测试水印功能...");

    // 创建测试图像 (500x400 蓝色背景)
    let test_image = DynamicImage::new_rgb8(500, 400);

    // 创建水印处理器
    let processor = SimpleWatermarkProcessor;

    // 创建文字水印配置
    let text_watermark = SimpleTextWatermark {
        text: "TEST COPYRIGHT".to_string(),
        font_size: 24,
        color: Rgba([255, 255, 255, 200]), // 半透明白色
        position: WatermarkPosition::BottomRight,
        opacity: 0.8,
        margin: 20,
        background: None,
    };

    // 添加文字水印
    match processor.add_text_watermark(test_image, &text_watermark) {
        Ok(watermarked_image) => {
            // 保存结果
            if let Err(e) = watermarked_image.save("test_watermark_output.png") {
                eprintln!("❌ 保存失败: {}", e);
            } else {
                println!("✅ 水印测试成功！输出文件: test_watermark_output.png");
                println!("   文字: {}", text_watermark.text);
                println!("   大小: {}px", text_watermark.font_size);
                println!("   位置: {:?}", text_watermark.position);
                println!("   透明度: {}", text_watermark.opacity);
            }
        },
        Err(e) => {
            eprintln!("❌ 水印处理失败: {}", e);
        }
    }

    println!("\n🔧 水印功能测试完成！");
}