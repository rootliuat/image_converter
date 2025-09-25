// WebP现代图像格式编码器 - 支持有损和无损压缩

use anyhow::Result;
use image::DynamicImage;
// std::io::Cursor 已移除 - 未在代码中使用

/// 高性能WebP有损编码器
pub fn encode_webp_lossy(
    image: &DynamicImage,
    quality: f32, // 0.0-100.0
    target_size_bytes: Option<usize>
) -> Result<Vec<u8>> {
    let start = std::time::Instant::now();

    // 转换为RGB格式（WebP最优）
    let rgb_image = image.to_rgb8();
    let width = rgb_image.width();
    let height = rgb_image.height();
    let pixels = rgb_image.as_raw();

    // 使用webp crate编码
    let encoder = webp::Encoder::from_rgb(pixels, width, height);

    // 设置质量参数
    let encoded = encoder.encode(quality);

    let result = encoded.to_vec();

    // 检查大小限制
    if let Some(max_size) = target_size_bytes {
        if result.len() > max_size && quality > 10.0 {
            // 递归降质量重试
            let new_quality = quality * 0.8;
            return encode_webp_lossy(image, new_quality, target_size_bytes);
        }
    }

    log::debug!("WebP有损编码耗时: {:?}, 质量: {:.1}, 大小: {} bytes",
               start.elapsed(), quality, result.len());

    Ok(result)
}

/// 高性能WebP无损编码器
pub fn encode_webp_lossless(image: &DynamicImage) -> Result<Vec<u8>> {
    let start = std::time::Instant::now();

    // 无损WebP支持RGBA
    let rgba_image = image.to_rgba8();
    let width = rgba_image.width();
    let height = rgba_image.height();
    let pixels = rgba_image.as_raw();

    // 使用webp crate进行无损编码
    let encoder = webp::Encoder::from_rgba(pixels, width, height);
    let encoded = encoder.encode_lossless();

    let result = encoded.to_vec();

    log::debug!("WebP无损编码耗时: {:?}, 大小: {} bytes",
               start.elapsed(), result.len());

    Ok(result)
}

/// 智能WebP编码器 - 自动选择最佳参数
pub fn encode_webp_smart(
    image: &DynamicImage,
    target_size_bytes: usize,
    prefer_quality: bool
) -> Result<Vec<u8>> {
    let pixels = image.width() * image.height();

    // 根据图像复杂度和目标大小选择策略
    let initial_quality = if target_size_bytes < 50_000 {
        50.0 // 小文件：中等质量
    } else if target_size_bytes < 200_000 {
        75.0 // 中等文件：高质量
    } else {
        85.0 // 大文件：很高质量
    };

    // 先尝试有损压缩
    let lossy_result = encode_webp_lossy(image, initial_quality, Some(target_size_bytes))?;

    // 如果更注重质量，也尝试无损压缩进行比较
    if prefer_quality || pixels < 500_000 { // 小图像可以考虑无损
        if let Ok(lossless_result) = encode_webp_lossless(image) {
            // 如果无损结果不超过目标太多，选择无损
            if lossless_result.len() <= target_size_bytes * 12 / 10 { // 允许20%的超出
                return Ok(lossless_result);
            }
        }
    }

    Ok(lossy_result)
}

/// WebP格式质量评估
#[allow(dead_code)]
pub fn estimate_webp_compression_ratio(width: u32, height: u32, quality: f32) -> f32 {
    let pixels = width * height;

    // 基于经验的WebP压缩比估算
    let base_ratio = match quality as u8 {
        0..=20 => 0.05,   // 极高压缩
        21..=40 => 0.08,  // 高压缩
        41..=60 => 0.12,  // 中等压缩
        61..=80 => 0.18,  // 低压缩
        _ => 0.25,        // 最低压缩
    };

    // 大图片通常有更好的压缩比
    let size_factor = if pixels > 1_000_000 { 0.9 } else { 1.1 };

    base_ratio * size_factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webp_lossy_encoding() {
        let test_image = DynamicImage::new_rgb8(100, 100);
        let result = encode_webp_lossy(&test_image, 80.0, None);
        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(data.len() > 50); // WebP应该产生一些数据
        assert!(data.starts_with(b"RIFF")); // WebP文件标识
    }

    #[test]
    fn test_webp_lossless_encoding() {
        let test_image = DynamicImage::new_rgba8(50, 50);
        let result = encode_webp_lossless(&test_image);
        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(data.len() > 50);
        assert!(data.starts_with(b"RIFF"));
    }

    #[test]
    fn test_webp_smart_encoding() {
        let test_image = DynamicImage::new_rgb8(200, 200);
        let result = encode_webp_smart(&test_image, 50000, true);
        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(data.len() <= 60000); // 应该接近目标大小
    }

    #[test]
    fn test_compression_ratio_estimation() {
        let ratio = estimate_webp_compression_ratio(1000, 1000, 80.0);
        assert!(ratio > 0.0 && ratio < 1.0);
    }
}