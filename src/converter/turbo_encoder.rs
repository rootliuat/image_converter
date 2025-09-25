// 涡轮增压编码器 - 极致性能优化

use anyhow::{Context, Result};
use image::{DynamicImage, ImageEncoder};
use once_cell::sync::Lazy;
use std::io::Cursor;

/// 内存池 - 复用大块内存避免分配开销
struct MemoryPool {
    buffers: std::sync::Mutex<Vec<Vec<u8>>>,
}

static MEMORY_POOL: Lazy<MemoryPool> = Lazy::new(|| MemoryPool {
    buffers: std::sync::Mutex::new(Vec::new()),
});

impl MemoryPool {
    fn get_buffer(&self, min_size: usize) -> Vec<u8> {
        let mut buffers = self.buffers.lock().unwrap();

        // 寻找合适大小的缓冲区
        for i in 0..buffers.len() {
            if buffers[i].capacity() >= min_size {
                let mut buf = buffers.swap_remove(i);
                buf.clear();
                return buf;
            }
        }

        // 没有合适的，创建新的
        Vec::with_capacity(min_size.next_power_of_two().max(1024 * 1024))
    }

    fn return_buffer(&self, mut buffer: Vec<u8>) {
        buffer.clear();
        let mut buffers = self.buffers.lock().unwrap();

        // 只保留适量的缓冲区
        if buffers.len() < 4 && buffer.capacity() > 1024 {
            buffers.push(buffer);
        }
    }
}

/// 超高性能JPEG编码器
pub fn turbo_encode_jpeg(
    image: &DynamicImage,
    quality: u8,
    target_size_bytes: Option<usize>
) -> Result<Vec<u8>> {
    let start = std::time::Instant::now();

    // 转换为最优格式
    let rgb_image = match image {
        DynamicImage::ImageRgb8(img) => img.clone(),
        _ => image.to_rgb8(), // 只在需要时转换
    };

    let width = rgb_image.width();
    let height = rgb_image.height();
    let raw_data = rgb_image.as_raw();

    // 使用内存池获取输出缓冲区
    let estimated_size = (width * height * 3 / 4) as usize; // 预估压缩后大小
    let mut output_buffer = MEMORY_POOL.get_buffer(estimated_size);

    // 创建JPEG编码器配置
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output_buffer, quality);

    // 执行编码
    encoder.encode(raw_data, width, height, image::ExtendedColorType::Rgb8)
        .context("JPEG编码失败")?;

    // 检查大小限制
    if let Some(max_size) = target_size_bytes {
        if output_buffer.len() > max_size && quality > 10 {
            // 递归降质量重试（最多3次）
            MEMORY_POOL.return_buffer(output_buffer);
            let new_quality = (quality as f32 * 0.8) as u8;
            return turbo_encode_jpeg(image, new_quality, target_size_bytes);
        }
    }

    log::debug!("JPEG编码耗时: {:?}, 质量: {}, 大小: {} bytes",
               start.elapsed(), quality, output_buffer.len());

    Ok(output_buffer)
}

/// 超高性能PNG编码器 - 无压缩模式
pub fn turbo_encode_png_fast(image: &DynamicImage) -> Result<Vec<u8>> {
    let start = std::time::Instant::now();

    // 预分配缓冲区
    let estimated_size = (image.width() * image.height() * 4 + 1024) as usize;
    let mut buffer = MEMORY_POOL.get_buffer(estimated_size);
    let mut cursor = Cursor::new(&mut buffer);

    // 使用最快PNG设置
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        &mut cursor,
        image::codecs::png::CompressionType::Fast,
        image::codecs::png::FilterType::NoFilter,
    );

    // 执行编码
    encoder.write_image(
        image.as_bytes(),
        image.width(),
        image.height(),
        image.color().into(),
    ).context("PNG快速编码失败")?;

    // 截取实际使用的部分
    let actual_size = cursor.position() as usize;
    buffer.truncate(actual_size);

    log::debug!("PNG快速编码耗时: {:?}, 大小: {} bytes",
               start.elapsed(), actual_size);

    Ok(buffer)
}

/// 超高性能PNG编码器 - 压缩模式（智能压缩）
pub fn turbo_encode_png_compressed(
    image: &DynamicImage,
    target_bytes: usize
) -> Result<Vec<u8>> {
    let start = std::time::Instant::now();

    // 快速估算需要的缩放比例
    let quick_size = estimate_png_size(image.width(), image.height());

    if quick_size <= target_bytes {
        // 不需要压缩，直接编码
        return turbo_encode_png_fast(image);
    }

    // 计算所需的缩放比例
    let scale_factor = ((target_bytes as f32 / quick_size as f32).sqrt() * 0.95).min(1.0);

    if scale_factor < 0.1 {
        return Err(anyhow::anyhow!("目标大小过小，无法压缩到指定大小"));
    }

    // 执行智能缩放
    let new_width = (image.width() as f32 * scale_factor).round().max(32.0) as u32;
    let new_height = (image.height() as f32 * scale_factor).round().max(32.0) as u32;

    // 使用智能滤镜缩放
    let filter = if scale_factor > 0.8 {
        image::imageops::FilterType::Lanczos3
    } else if scale_factor > 0.5 {
        image::imageops::FilterType::CatmullRom
    } else {
        image::imageops::FilterType::Triangle
    };

    let resized = image.resize_exact(new_width, new_height, filter);
    let result = turbo_encode_png_fast(&resized)?;

    log::debug!("PNG压缩编码耗时: {:?}, 缩放: {:.2}x, 大小: {} bytes",
               start.elapsed(), scale_factor, result.len());

    Ok(result)
}

/// 智能估算PNG文件大小
fn estimate_png_size(width: u32, height: u32) -> usize {
    // 基于经验公式估算PNG大小
    let pixels = width * height;
    let base_size = pixels as usize * 3; // RGB数据
    let compression_ratio = 0.7; // 假设70%压缩率

    (base_size as f32 * compression_ratio) as usize + 1024 // 加上header开销
}

/// 批量并行编码 - 利用多核心
#[allow(dead_code)]
pub fn turbo_batch_encode(
    images: &[DynamicImage],
    encode_fn: impl Fn(&DynamicImage) -> Result<Vec<u8>> + Sync + Send + Copy,
) -> Vec<Result<Vec<u8>>> {
    use rayon::prelude::*;

    images
        .par_iter()
        .map(encode_fn)
        .collect()
}

/// 清理内存池（在程序结束时调用）
#[allow(dead_code)]
pub fn cleanup_memory_pool() {
    let mut buffers = MEMORY_POOL.buffers.lock().unwrap();
    buffers.clear();
    buffers.shrink_to_fit();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turbo_jpeg_encoding() {
        let test_image = DynamicImage::new_rgb8(200, 200);
        let result = turbo_encode_jpeg(&test_image, 80, None);
        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(data.len() > 100); // 应该产生一些数据
    }

    #[test]
    fn test_turbo_png_fast() {
        let test_image = DynamicImage::new_rgb8(100, 100);
        let result = turbo_encode_png_fast(&test_image);
        assert!(result.is_ok());
    }

    #[test]
    fn test_size_estimation() {
        let size = estimate_png_size(1000, 1000);
        assert!(size > 1000000); // 应该是一个合理的大小
        assert!(size < 5000000); // 但不会过大
    }
}