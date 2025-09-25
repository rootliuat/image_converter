use anyhow::{Context, Result};
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;
use std::path::Path;
// Re-export OutputFormat for external use
pub use crate::utils::config::OutputFormat;
use crate::converter::{turbo_encoder, webp_encoder, simple_watermark};

/// 核心函数：压缩图像并保存到文件
pub fn compress_and_save(
    image: &DynamicImage,
    output_path: &Path,
    target_kb: u32,
    output_format: OutputFormat,
) -> Result<()> {
    let target_bytes = target_kb as usize * 1024;

    let compressed_data = match output_format {
        OutputFormat::Jpeg => {
            // 使用涡轮增压JPEG编码器
            turbo_encode_jpeg_adaptive(image, target_bytes)?
        },
        OutputFormat::PngCompressed => {
            // 使用涡轮增压PNG压缩编码器
            turbo_encoder::turbo_encode_png_compressed(image, target_bytes)?
        },
        OutputFormat::PngOriginal => {
            // 使用涡轮增压PNG快速编码器
            turbo_encoder::turbo_encode_png_fast(image)?
        },
        OutputFormat::WebPLossy => {
            // 使用WebP有损压缩（现代高效格式）
            webp_encoder::encode_webp_smart(image, target_bytes, false)?
        },
        OutputFormat::WebPLossless => {
            // 使用WebP无损压缩（比PNG更小）
            webp_encoder::encode_webp_lossless(image)?
        }
    };

    std::fs::write(output_path, compressed_data)
        .with_context(|| format!("无法写入文件到 '{}'", output_path.display()))?;

    Ok(())
}

/// 涡轮增压自适应JPEG编码器
fn turbo_encode_jpeg_adaptive(image: &DynamicImage, target_bytes: usize) -> Result<Vec<u8>> {
    // 快速预估最佳质量
    let pixels = image.width() * image.height();
    let _complexity_factor = if pixels > 1_000_000 { 0.7 } else { 0.8 }; // 大图片通常压缩比更高

    let initial_quality = if target_bytes < 50_000 {
        40 // 小目标：低质量
    } else if target_bytes < 200_000 {
        60 // 中目标：中等质量
    } else {
        80 // 大目标：高质量
    };

    // 使用涡轮增压编码器，允许自动调整质量
    turbo_encoder::turbo_encode_jpeg(image, initial_quality, Some(target_bytes))
}

// 已移除 compress_jpeg 函数 - 未使用
// 已移除 find_best_quality 函数 - 未使用
// 已移除 compress_png_optimized 函数 - 未使用
// 已移除 encode_png_fast 函数 - 未使用
// 已移除 encode_png_original 函数 - 未使用

/// 传统PNG压缩（保持向后兼容）
#[allow(dead_code)]
fn compress_png(image: &DynamicImage) -> Result<Vec<u8>> {
    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, ImageFormat::Png)
        .context("编码PNG失败")?;
    Ok(buffer.into_inner())
}

/// 添加水印并压缩保存图像
pub fn compress_and_save_with_watermark(
    image: &DynamicImage,
    output_path: &Path,
    target_kb: u32,
    output_format: OutputFormat,
    text_watermark: Option<&simple_watermark::SimpleTextWatermark>,
    image_watermark: Option<&simple_watermark::ImageWatermark>,
) -> Result<()> {
    let processor = simple_watermark::SimpleWatermarkProcessor;
    let mut processed_image = image.clone();

    // 添加文字水印
    if let Some(text_config) = text_watermark {
        processed_image = processor.add_text_watermark(processed_image, text_config)?;
    }

    // 添加图片水印
    if let Some(image_config) = image_watermark {
        processed_image = processor.add_image_watermark(processed_image, image_config)?;
    }

    // 使用原有的压缩保存逻辑
    compress_and_save(&processed_image, output_path, target_kb, output_format)
}