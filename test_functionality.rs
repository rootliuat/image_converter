// 功能测试脚本 - 测试新增的无压缩PNG功能

use std::path::Path;
use std::time::Instant;

mod converter;
mod utils;

use converter::image_converter;
use utils::config::OutputFormat;

fn create_test_image() -> image::DynamicImage {
    // 创建一个简单的测试图片：200x200像素，红色背景
    let width = 200;
    let height = 200;
    let mut img = image::ImageBuffer::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = ((x + y) % 256) as u8;  // 创建渐变效果
        let g = 0;
        let b = 255 - r;
        *pixel = image::Rgb([r, g, b]);
    }

    image::DynamicImage::ImageRgb8(img)
}

fn test_jpeg_conversion() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试 JPEG 格式转换...");

    let test_image = create_test_image();
    let output_path = Path::new("test_output/test_jpeg.jpg");

    let start_time = Instant::now();
    image_converter::compress_and_save(&test_image, output_path, 400, OutputFormat::Jpeg)?;
    let duration = start_time.elapsed();

    // 验证文件是否存在
    if output_path.exists() {
        let file_size = std::fs::metadata(output_path)?.len();
        println!("  ✅ JPEG转换成功 - 文件大小: {} bytes, 耗时: {:?}", file_size, duration);

        // 验证文件大小是否合理（应该小于800KB）
        if file_size < 800 * 1024 {
            println!("  ✅ JPEG文件大小验证通过");
        } else {
            println!("  ⚠️ JPEG文件大小可能过大: {} bytes", file_size);
        }
    } else {
        return Err("JPEG文件未生成".into());
    }

    Ok(())
}

fn test_png_compressed() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试 PNG 压缩模式...");

    let test_image = create_test_image();
    let output_path = Path::new("test_output/test_png_compressed.png");

    let start_time = Instant::now();
    image_converter::compress_and_save(&test_image, output_path, 400, OutputFormat::PngCompressed)?;
    let duration = start_time.elapsed();

    if output_path.exists() {
        let file_size = std::fs::metadata(output_path)?.len();
        println!("  ✅ PNG压缩转换成功 - 文件大小: {} bytes, 耗时: {:?}", file_size, duration);

        // 验证文件大小是否合理（应该接近400KB或更小）
        if file_size < 800 * 1024 {
            println!("  ✅ PNG压缩文件大小验证通过");
        } else {
            println!("  ⚠️ PNG压缩文件大小可能过大: {} bytes", file_size);
        }
    } else {
        return Err("PNG压缩文件未生成".into());
    }

    Ok(())
}

fn test_png_original() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试 PNG 原始模式 (新功能)...");

    let test_image = create_test_image();
    let output_path = Path::new("test_output/test_png_original.png");

    let start_time = Instant::now();
    image_converter::compress_and_save(&test_image, output_path, 400, OutputFormat::PngOriginal)?;
    let duration = start_time.elapsed();

    if output_path.exists() {
        let file_size = std::fs::metadata(output_path)?.len();
        println!("  ✅ PNG原始转换成功 - 文件大小: {} bytes, 耗时: {:?}", file_size, duration);
        println!("  ✨ 这是新增的无压缩PNG功能！");
    } else {
        return Err("PNG原始文件未生成".into());
    }

    Ok(())
}

fn performance_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ 性能对比测试 - PNG压缩 vs PNG原始...");

    let test_image = create_test_image();

    // 测试PNG压缩模式性能
    let start_compressed = Instant::now();
    image_converter::compress_and_save(
        &test_image,
        Path::new("test_output/perf_compressed.png"),
        400,
        OutputFormat::PngCompressed
    )?;
    let compressed_duration = start_compressed.elapsed();

    // 测试PNG原始模式性能
    let start_original = Instant::now();
    image_converter::compress_and_save(
        &test_image,
        Path::new("test_output/perf_original.png"),
        400,
        OutputFormat::PngOriginal
    )?;
    let original_duration = start_original.elapsed();

    println!("  PNG压缩模式耗时: {:?}", compressed_duration);
    println!("  PNG原始模式耗时: {:?}", original_duration);

    if original_duration < compressed_duration {
        let speedup = compressed_duration.as_nanos() as f64 / original_duration.as_nanos() as f64;
        println!("  🚀 PNG原始模式比压缩模式快 {:.2}x 倍!", speedup);
    } else {
        println!("  📊 性能结果记录完成");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 图片转换工具功能测试开始...\n");

    // 测试各种格式转换
    test_jpeg_conversion()?;
    println!();

    test_png_compressed()?;
    println!();

    test_png_original()?;
    println!();

    performance_comparison()?;
    println!();

    println!("🎉 所有功能测试完成！");
    println!("📁 请查看 test_output/ 目录中的输出文件");

    Ok(())
}