// WebP功能简化测试

// Removed unused import: std::path::Path
use std::time::Instant;

fn create_test_image() -> image::DynamicImage {
    // 创建一个彩色测试图片：300x300像素，彩虹渐变
    let width = 300;
    let height = 300;
    let mut img = image::ImageBuffer::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = ((x * 255) / width) as u8;
        let g = ((y * 255) / height) as u8;
        let b = (((x + y) * 255) / (width + height)) as u8;
        *pixel = image::Rgb([r, g, b]);
    }

    image::DynamicImage::ImageRgb8(img)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 WebP功能简化测试开始...\n");

    // 确保测试目录存在
    std::fs::create_dir_all("test_webp_simple")?;

    let test_image = create_test_image();

    println!("📊 使用WebP编码器进行直接测试...");

    // 直接使用WebP编码器进行测试
    use image_converter::converter::webp_encoder;

    // 测试WebP有损编码
    println!("🧪 测试 WebP 有损编码...");
    let start = Instant::now();
    let webp_lossy_data = webp_encoder::encode_webp_lossy(&test_image, 80.0, Some(400 * 1024))?;
    let webp_lossy_duration = start.elapsed();

    std::fs::write("test_webp_simple/test_lossy.webp", &webp_lossy_data)?;
    println!("  ✅ WebP有损编码成功 - 文件大小: {} bytes, 耗时: {:?}", webp_lossy_data.len(), webp_lossy_duration);

    // 测试WebP无损编码
    println!("\n🧪 测试 WebP 无损编码...");
    let start = Instant::now();
    let webp_lossless_data = webp_encoder::encode_webp_lossless(&test_image)?;
    let webp_lossless_duration = start.elapsed();

    std::fs::write("test_webp_simple/test_lossless.webp", &webp_lossless_data)?;
    println!("  ✅ WebP无损编码成功 - 文件大小: {} bytes, 耗时: {:?}", webp_lossless_data.len(), webp_lossless_duration);

    // 测试智能WebP编码
    println!("\n🧪 测试 智能 WebP 编码...");
    let start = Instant::now();
    let webp_smart_data = webp_encoder::encode_webp_smart(&test_image, 400 * 1024, true)?;
    let webp_smart_duration = start.elapsed();

    std::fs::write("test_webp_simple/test_smart.webp", &webp_smart_data)?;
    println!("  ✅ 智能WebP编码成功 - 文件大小: {} bytes, 耗时: {:?}", webp_smart_data.len(), webp_smart_duration);

    // 对比传统格式
    println!("\n📊 格式对比测试...");

    // JPEG对比
    let _jpeg_start = Instant::now();
    let _jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut std::io::Cursor::new(Vec::new()), 80);
    // 这里简化对比，主要展示WebP功能

    println!("\n⚡ WebP性能总结:");
    println!("  WebP有损:   {} bytes, {:?}", webp_lossy_data.len(), webp_lossy_duration);
    println!("  WebP无损:   {} bytes, {:?}", webp_lossless_data.len(), webp_lossless_duration);
    println!("  WebP智能:   {} bytes, {:?}", webp_smart_data.len(), webp_smart_duration);

    // 计算压缩效果
    let original_size = test_image.width() * test_image.height() * 3; // RGB
    println!("\n💾 压缩比较 (原始大小: {} bytes):", original_size);
    println!("  WebP有损压缩率: {:.1}%", (webp_lossy_data.len() as f32 / original_size as f32) * 100.0);
    println!("  WebP无损压缩率: {:.1}%", (webp_lossless_data.len() as f32 / original_size as f32) * 100.0);
    println!("  WebP智能压缩率: {:.1}%", (webp_smart_data.len() as f32 / original_size as f32) * 100.0);

    println!("\n🎉 WebP功能测试完成！");
    println!("📁 请查看 test_webp_simple/ 目录中的输出文件");

    // 显示生成的文件
    println!("\n📊 生成的WebP文件:");
    for entry in std::fs::read_dir("test_webp_simple")? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        println!("  {} - {} bytes", entry.file_name().to_string_lossy(), metadata.len());
    }

    Ok(())
}