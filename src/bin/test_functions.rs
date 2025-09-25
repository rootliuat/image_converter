// 功能测试二进制文件 - 测试新增的无压缩PNG功能

use std::path::Path;
use std::time::Instant;
use image_converter::converter::image_converter::compress_and_save;
use image_converter::utils::config::OutputFormat;

fn create_test_image() -> image::DynamicImage {
    // 创建一个简单的测试图片：200x200像素，渐变效果
    let width = 200;
    let height = 200;
    let mut img = image::ImageBuffer::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = ((x + y) % 256) as u8;
        let g = ((x * 2) % 256) as u8;
        let b = 255 - r;
        *pixel = image::Rgb([r, g, b]);
    }

    image::DynamicImage::ImageRgb8(img)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 图片转换工具功能测试开始...\n");

    // 确保测试目录存在
    std::fs::create_dir_all("test_output")?;

    let test_image = create_test_image();

    // 测试1: JPEG转换
    println!("🧪 测试 JPEG 格式转换...");
    let jpeg_path = Path::new("test_output/test_jpeg.jpg");
    let start = Instant::now();
    compress_and_save(&test_image, jpeg_path, 400, OutputFormat::Jpeg)?;
    let jpeg_duration = start.elapsed();

    if jpeg_path.exists() {
        let file_size = std::fs::metadata(jpeg_path)?.len();
        println!("  ✅ JPEG转换成功 - 文件大小: {} bytes, 耗时: {:?}", file_size, jpeg_duration);
    }

    // 测试2: PNG压缩模式
    println!("\n🧪 测试 PNG 压缩模式...");
    let png_comp_path = Path::new("test_output/test_png_compressed.png");
    let start = Instant::now();
    compress_and_save(&test_image, png_comp_path, 400, OutputFormat::PngCompressed)?;
    let png_comp_duration = start.elapsed();

    if png_comp_path.exists() {
        let file_size = std::fs::metadata(png_comp_path)?.len();
        println!("  ✅ PNG压缩转换成功 - 文件大小: {} bytes, 耗时: {:?}", file_size, png_comp_duration);
    }

    // 测试3: PNG原始模式 (新功能)
    println!("\n🧪 测试 PNG 原始模式 (新功能)...");
    let png_orig_path = Path::new("test_output/test_png_original.png");
    let start = Instant::now();
    compress_and_save(&test_image, png_orig_path, 400, OutputFormat::PngOriginal)?;
    let png_orig_duration = start.elapsed();

    if png_orig_path.exists() {
        let file_size = std::fs::metadata(png_orig_path)?.len();
        println!("  ✅ PNG原始转换成功 - 文件大小: {} bytes, 耗时: {:?}", file_size, png_orig_duration);
        println!("  ✨ 这是新增的无压缩PNG功能！");
    }

    // 性能对比
    println!("\n⚡ 性能对比结果:");
    println!("  JPEG 模式耗时: {:?}", jpeg_duration);
    println!("  PNG压缩模式耗时: {:?}", png_comp_duration);
    println!("  PNG原始模式耗时: {:?}", png_orig_duration);

    if png_orig_duration < png_comp_duration {
        let speedup = png_comp_duration.as_nanos() as f64 / png_orig_duration.as_nanos() as f64;
        println!("  🚀 PNG原始模式比压缩模式快 {:.2}x 倍!", speedup);
    }

    println!("\n🎉 所有功能测试完成！");
    println!("📁 请查看 test_output/ 目录中的输出文件");

    // 显示文件信息
    println!("\n📊 输出文件详情:");
    for entry in std::fs::read_dir("test_output")? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        println!("  {} - {} bytes", entry.file_name().to_string_lossy(), metadata.len());
    }

    Ok(())
}