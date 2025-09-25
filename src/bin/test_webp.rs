// WebP功能测试

use std::path::Path;
use std::time::Instant;
use image_converter::{compress_and_save, OutputFormat};

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
    println!("🚀 WebP功能测试开始...\n");

    // 确保测试目录存在
    std::fs::create_dir_all("test_webp_output")?;

    let test_image = create_test_image();

    // 测试1: WebP有损压缩
    println!("🧪 测试 WebP 有损压缩...");
    let webp_lossy_path = Path::new("test_webp_output/test_webp_lossy.webp");
    let start = Instant::now();
    compress_and_save(&test_image, webp_lossy_path, 400, OutputFormat::WebPLossy)?;
    let webp_lossy_duration = start.elapsed();

    if webp_lossy_path.exists() {
        let file_size = std::fs::metadata(webp_lossy_path)?.len();
        println!("  ✅ WebP有损压缩成功 - 文件大小: {} bytes, 耗时: {:?}", file_size, webp_lossy_duration);
    }

    // 测试2: WebP无损压缩
    println!("\n🧪 测试 WebP 无损压缩...");
    let webp_lossless_path = Path::new("test_webp_output/test_webp_lossless.webp");
    let start = Instant::now();
    compress_and_save(&test_image, webp_lossless_path, 400, OutputFormat::WebPLossless)?;
    let webp_lossless_duration = start.elapsed();

    if webp_lossless_path.exists() {
        let file_size = std::fs::metadata(webp_lossless_path)?.len();
        println!("  ✅ WebP无损压缩成功 - 文件大小: {} bytes, 耗时: {:?}", file_size, webp_lossless_duration);
    }

    // 对比测试: JPEG vs PNG vs WebP
    println!("\n📊 格式对比测试...");

    // JPEG
    let jpeg_path = Path::new("test_webp_output/compare_jpeg.jpg");
    let start = Instant::now();
    compress_and_save(&test_image, jpeg_path, 400, OutputFormat::Jpeg)?;
    let jpeg_duration = start.elapsed();
    let jpeg_size = std::fs::metadata(jpeg_path)?.len();

    // PNG压缩
    let png_path = Path::new("test_webp_output/compare_png.png");
    let start = Instant::now();
    compress_and_save(&test_image, png_path, 400, OutputFormat::PngCompressed)?;
    let png_duration = start.elapsed();
    let png_size = std::fs::metadata(png_path)?.len();

    // WebP有损 (再次测试以获得准确对比)
    let webp_compare_path = Path::new("test_webp_output/compare_webp.webp");
    let start = Instant::now();
    compress_and_save(&test_image, webp_compare_path, 400, OutputFormat::WebPLossy)?;
    let webp_compare_duration = start.elapsed();
    let webp_compare_size = std::fs::metadata(webp_compare_path)?.len();

    println!("\n⚡ 性能和压缩率对比:");
    println!("  格式      | 文件大小  | 处理时间  | 压缩率");
    println!("  ---------|----------|----------|----------");
    println!("  JPEG     | {:8} | {:8?} | 基准", jpeg_size, jpeg_duration);
    println!("  PNG      | {:8} | {:8?} | {:.1}x", png_size, png_duration, jpeg_size as f32 / png_size as f32);
    println!("  WebP     | {:8} | {:8?} | {:.1}x", webp_compare_size, webp_compare_duration, jpeg_size as f32 / webp_compare_size as f32);

    // 计算节省的空间
    let jpeg_vs_webp_savings = (1.0 - webp_compare_size as f32 / jpeg_size as f32) * 100.0;
    let png_vs_webp_savings = (1.0 - webp_compare_size as f32 / png_size as f32) * 100.0;

    println!("\n💾 空间节省:");
    println!("  WebP vs JPEG: 节省 {:.1}% 空间", jpeg_vs_webp_savings);
    println!("  WebP vs PNG:  节省 {:.1}% 空间", png_vs_webp_savings);

    // 速度对比
    println!("\n🏃 速度对比:");
    if webp_compare_duration < jpeg_duration {
        let speedup = jpeg_duration.as_nanos() as f32 / webp_compare_duration.as_nanos() as f32;
        println!("  WebP比JPEG快 {:.1}x 倍", speedup);
    }
    if webp_compare_duration < png_duration {
        let speedup = png_duration.as_nanos() as f32 / webp_compare_duration.as_nanos() as f32;
        println!("  WebP比PNG快 {:.1}x 倍", speedup);
    }

    println!("\n🎉 WebP功能测试完成！");
    println!("📁 请查看 test_webp_output/ 目录中的输出文件");

    // 显示所有输出文件
    println!("\n📊 生成的文件:");
    for entry in std::fs::read_dir("test_webp_output")? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let extension_str = entry.path().extension().unwrap_or_default().to_string_lossy().to_string();
        println!("  {} ({}) - {} bytes",
                entry.file_name().to_string_lossy(),
                extension_str.to_uppercase(),
                metadata.len());
    }

    Ok(())
}