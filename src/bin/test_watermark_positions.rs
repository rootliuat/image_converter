// 水印位置功能测试

use image_converter::converter::simple_watermark::{
    SimpleWatermarkProcessor, SimpleTextWatermark, WatermarkPosition
};
use image::{Rgba, DynamicImage};

fn main() {
    println!("🎨 开始测试水印位置功能...");

    // 创建测试图像 (600x400 深蓝色背景)
    let mut test_image = DynamicImage::new_rgb8(600, 400);
    // 填充背景色
    let image_buffer = test_image.as_mut_rgb8().unwrap();
    for pixel in image_buffer.pixels_mut() {
        *pixel = image::Rgb([30, 30, 100]); // 深蓝色背景
    }

    let processor = SimpleWatermarkProcessor;

    // 测试所有预设位置
    let positions = WatermarkPosition::all_positions();

    println!("📍 可用的水印位置:");
    for (position, name) in &positions {
        println!("   - {}: {}", name, position.display_name());

        // 为每个位置创建一个水印
        let text_watermark = SimpleTextWatermark {
            text: name.to_string(),
            font_size: 16,
            color: Rgba([255, 255, 255, 220]), // 半透明白色
            position: *position,
            opacity: 0.9,
            margin: 15,
            background: Some(Rgba([0, 0, 0, 100])), // 半透明黑色背景
        };

        // 添加文字水印
        match processor.add_text_watermark(test_image.clone(), &text_watermark) {
            Ok(watermarked_image) => {
                // 保存结果
                let filename = format!("test_position_{}.png", name.replace(" ", "_"));
                if let Err(e) = watermarked_image.save(&filename) {
                    eprintln!("❌ 保存 {} 失败: {}", filename, e);
                } else {
                    println!("   ✅ 已保存: {}", filename);
                }
            },
            Err(e) => {
                eprintln!("❌ 水印处理失败 {}: {}", name, e);
            }
        }
    }

    // 测试自定义位置
    println!("\n🎯 测试自定义位置...");
    let custom_watermark = SimpleTextWatermark {
        text: "CUSTOM POSITION".to_string(),
        font_size: 20,
        color: Rgba([255, 255, 0, 200]), // 半透明黄色
        position: WatermarkPosition::Custom(100, 150),
        opacity: 0.8,
        margin: 0, // 自定义位置时margin无效
        background: Some(Rgba([255, 0, 0, 80])), // 半透明红色背景
    };

    match processor.add_text_watermark(test_image, &custom_watermark) {
        Ok(watermarked_image) => {
            if let Err(e) = watermarked_image.save("test_position_custom.png") {
                eprintln!("❌ 保存自定义位置测试失败: {}", e);
            } else {
                println!("✅ 已保存: test_position_custom.png (位置: 100, 150)");
            }
        },
        Err(e) => {
            eprintln!("❌ 自定义位置水印处理失败: {}", e);
        }
    }

    println!("\n🔧 水印位置功能测试完成！");
    println!("   总共测试了 {} 个预设位置 + 1 个自定义位置", positions.len());
}