// 简化水印处理器 - 无需外部字体依赖

use anyhow::{Context, Result};
use image::{DynamicImage, Rgba, RgbaImage};

/// 水印位置枚举
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum WatermarkPosition {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Custom(u32, u32),
}

impl WatermarkPosition {
    /// 获取所有预设位置选项
    pub fn all_positions() -> Vec<(Self, &'static str)> {
        vec![
            (WatermarkPosition::TopLeft, "左上角"),
            (WatermarkPosition::TopCenter, "上方中央"),
            (WatermarkPosition::TopRight, "右上角"),
            (WatermarkPosition::MiddleLeft, "左侧中央"),
            (WatermarkPosition::MiddleCenter, "正中央"),
            (WatermarkPosition::MiddleRight, "右侧中央"),
            (WatermarkPosition::BottomLeft, "左下角"),
            (WatermarkPosition::BottomCenter, "下方中央"),
            (WatermarkPosition::BottomRight, "右下角"),
        ]
    }

    /// 获取显示名称
    #[allow(dead_code)]
    pub fn display_name(&self) -> String {
        match self {
            WatermarkPosition::TopLeft => "左上角".to_string(),
            WatermarkPosition::TopCenter => "上方中央".to_string(),
            WatermarkPosition::TopRight => "右上角".to_string(),
            WatermarkPosition::MiddleLeft => "左侧中央".to_string(),
            WatermarkPosition::MiddleCenter => "正中央".to_string(),
            WatermarkPosition::MiddleRight => "右侧中央".to_string(),
            WatermarkPosition::BottomLeft => "左下角".to_string(),
            WatermarkPosition::BottomCenter => "下方中央".to_string(),
            WatermarkPosition::BottomRight => "右下角".to_string(),
            WatermarkPosition::Custom(x, y) => format!("自定义 ({}, {})", x, y),
        }
    }
}

/// 简化文字水印配置
#[derive(Debug, Clone)]
pub struct SimpleTextWatermark {
    pub text: String,
    pub font_size: u32,
    pub color: Rgba<u8>,
    pub position: WatermarkPosition,
    pub opacity: f32,
    pub margin: u32,
    pub background: Option<Rgba<u8>>, // 可选背景色
    pub letter_spacing: f32, // 字符间距（像素）
}

/// 图片水印配置
#[derive(Debug, Clone)]
pub struct ImageWatermark {
    pub watermark_path: String,
    pub position: WatermarkPosition,
    pub opacity: f32,
    pub scale: f32,
    pub margin: u32,
}

/// 简化水印处理器
pub struct SimpleWatermarkProcessor;

impl Default for WatermarkPosition {
    fn default() -> Self {
        WatermarkPosition::BottomRight
    }
}

impl Default for SimpleTextWatermark {
    fn default() -> Self {
        Self {
            text: "Watermark".to_string(),
            font_size: 20,
            color: Rgba([255, 255, 255, 220]), // 半透明白色
            position: WatermarkPosition::BottomRight,
            opacity: 0.8,
            margin: 20,
            background: Some(Rgba([0, 0, 0, 100])), // 半透明黑色背景
            letter_spacing: 2.0, // 默认字符间距2像素
        }
    }
}

impl Default for ImageWatermark {
    fn default() -> Self {
        Self {
            watermark_path: String::new(),
            position: WatermarkPosition::BottomRight,
            opacity: 0.8,
            scale: 0.2,
            margin: 20,
        }
    }
}

impl SimpleWatermarkProcessor {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    /// 添加简化文字水印（使用像素绘制）
    pub fn add_text_watermark(
        &self,
        image: DynamicImage,
        config: &SimpleTextWatermark,
    ) -> Result<DynamicImage> {
        let start = std::time::Instant::now();

        let mut rgba_image = image.to_rgba8();

        // 计算文字区域尺寸（基于字符数、字体大小和字符间距）
        let char_width = config.font_size * 6 / 10; // 近似字符宽度
        let text_width = if config.text.len() > 0 {
            (config.text.len() as f32 * char_width as f32 + (config.text.len() as f32 - 1.0) * config.letter_spacing) as u32
        } else {
            0
        };
        let text_height = config.font_size;

        // 计算位置
        let (x, y) = self.calculate_position(
            rgba_image.width(),
            rgba_image.height(),
            text_width,
            text_height,
            config.position,
            config.margin,
        );

        // 绘制背景（如果有）
        if let Some(bg_color) = config.background {
            let bg_color = self.apply_opacity(bg_color, config.opacity);
            self.draw_text_background(&mut rgba_image, x, y, text_width, text_height, bg_color);
        }

        // 绘制简化文字（使用像素点阵和字符间距）
        let text_color = self.apply_opacity(config.color, config.opacity);
        self.draw_pixel_text(&mut rgba_image, &config.text, x + 5, y + 3, config.font_size, text_color, config.letter_spacing)?;

        log::debug!("简化文字水印处理耗时: {:?}", start.elapsed());

        Ok(DynamicImage::ImageRgba8(rgba_image))
    }

    /// 添加图片水印
    pub fn add_image_watermark(
        &self,
        image: DynamicImage,
        config: &ImageWatermark,
    ) -> Result<DynamicImage> {
        let start = std::time::Instant::now();

        // 加载水印图片
        let watermark = image::open(&config.watermark_path)
            .with_context(|| format!("无法加载水印图片: {}", config.watermark_path))?;

        // 缩放水印
        let scaled_watermark = if config.scale != 1.0 {
            let new_width = (watermark.width() as f32 * config.scale).round().max(1.0) as u32;
            let new_height = (watermark.height() as f32 * config.scale).round().max(1.0) as u32;
            watermark.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            watermark
        };

        // 转换为RGBA
        let mut base_rgba = image.to_rgba8();
        let watermark_rgba = scaled_watermark.to_rgba8();

        // 计算位置
        let (x, y) = self.calculate_position(
            base_rgba.width(),
            base_rgba.height(),
            watermark_rgba.width(),
            watermark_rgba.height(),
            config.position,
            config.margin,
        );

        // 混合图像
        self.blend_images(&mut base_rgba, &watermark_rgba, x, y, config.opacity)?;

        log::debug!("图片水印处理耗时: {:?}", start.elapsed());

        Ok(DynamicImage::ImageRgba8(base_rgba))
    }

    /// 批量添加水印
    #[allow(dead_code)]
    pub fn batch_add_watermark(
        &self,
        images: Vec<DynamicImage>,
        text_config: Option<&SimpleTextWatermark>,
        image_config: Option<&ImageWatermark>,
    ) -> Vec<Result<DynamicImage>> {
        use rayon::prelude::*;

        images
            .into_par_iter()
            .map(|mut img| {
                // 先添加文字水印
                if let Some(text_config) = text_config {
                    img = self.add_text_watermark(img, text_config)?;
                }

                // 再添加图片水印
                if let Some(image_config) = image_config {
                    img = self.add_image_watermark(img, image_config)?;
                }

                Ok(img)
            })
            .collect()
    }

    /// 计算水印位置
    fn calculate_position(
        &self,
        img_width: u32,
        img_height: u32,
        wm_width: u32,
        wm_height: u32,
        position: WatermarkPosition,
        margin: u32,
    ) -> (u32, u32) {
        match position {
            WatermarkPosition::TopLeft => (margin, margin),
            WatermarkPosition::TopCenter => ((img_width.saturating_sub(wm_width)) / 2, margin),
            WatermarkPosition::TopRight => (img_width.saturating_sub(wm_width + margin), margin),
            WatermarkPosition::MiddleLeft => (margin, (img_height.saturating_sub(wm_height)) / 2),
            WatermarkPosition::MiddleCenter => (
                (img_width.saturating_sub(wm_width)) / 2,
                (img_height.saturating_sub(wm_height)) / 2,
            ),
            WatermarkPosition::MiddleRight => (
                img_width.saturating_sub(wm_width + margin),
                (img_height.saturating_sub(wm_height)) / 2,
            ),
            WatermarkPosition::BottomLeft => (margin, img_height.saturating_sub(wm_height + margin)),
            WatermarkPosition::BottomCenter => (
                (img_width.saturating_sub(wm_width)) / 2,
                img_height.saturating_sub(wm_height + margin),
            ),
            WatermarkPosition::BottomRight => (
                img_width.saturating_sub(wm_width + margin),
                img_height.saturating_sub(wm_height + margin),
            ),
            WatermarkPosition::Custom(x, y) => (x, y),
        }
    }

    /// 应用透明度到颜色
    fn apply_opacity(&self, mut color: Rgba<u8>, opacity: f32) -> Rgba<u8> {
        color.0[3] = (color.0[3] as f32 * opacity.clamp(0.0, 1.0)) as u8;
        color
    }

    /// 绘制文字背景
    fn draw_text_background(
        &self,
        image: &mut RgbaImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: Rgba<u8>,
    ) {
        // 绘制背景矩形
        let x2 = (x + width + 6).min(image.width() - 1);
        let y2 = (y + height + 4).min(image.height() - 1);

        for py in y..y2 {
            for px in x..x2 {
                if px < image.width() && py < image.height() {
                    let pixel = image.get_pixel_mut(px, py);
                    self.blend_pixel(pixel, &color);
                }
            }
        }
    }

    /// 绘制简化像素文字（支持字符间距）
    fn draw_pixel_text(
        &self,
        image: &mut RgbaImage,
        text: &str,
        start_x: u32,
        start_y: u32,
        font_size: u32,
        color: Rgba<u8>,
        letter_spacing: f32,
    ) -> Result<()> {
        let char_width = font_size * 6 / 10;
        let _char_height = font_size;

        for (char_idx, ch) in text.chars().enumerate() {
            let char_x = start_x + (char_idx as f32 * (char_width as f32 + letter_spacing)) as u32;

            // 简化字符绘制（基于ASCII）
            self.draw_simple_char(image, ch, char_x, start_y, font_size, color)?;
        }

        Ok(())
    }

    /// 绘制简化字符
    fn draw_simple_char(
        &self,
        image: &mut RgbaImage,
        ch: char,
        x: u32,
        y: u32,
        size: u32,
        color: Rgba<u8>,
    ) -> Result<()> {
        // 简化的字符点阵（8x8基础网格，根据size缩放）
        let pattern = self.get_char_pattern(ch);
        let scale = (size / 8).max(1);

        for row in 0..8 {
            for col in 0..8 {
                if (pattern[row] >> (7 - col)) & 1 == 1 {
                    // 绘制像素块
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let px = x + col as u32 * scale + dx;
                            let py = y + row as u32 * scale + dy;

                            if px < image.width() && py < image.height() {
                                let pixel = image.get_pixel_mut(px, py);
                                self.blend_pixel(pixel, &color);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 获取字符点阵模式（8x8简化版）
    fn get_char_pattern(&self, ch: char) -> [u8; 8] {
        match ch {
            'A' | 'a' => [0x18, 0x24, 0x42, 0x42, 0x7E, 0x42, 0x42, 0x00],
            'B' | 'b' => [0x7C, 0x42, 0x42, 0x7C, 0x42, 0x42, 0x7C, 0x00],
            'C' | 'c' => [0x3C, 0x42, 0x40, 0x40, 0x40, 0x42, 0x3C, 0x00],
            'D' | 'd' => [0x78, 0x44, 0x42, 0x42, 0x42, 0x44, 0x78, 0x00],
            'E' | 'e' => [0x7E, 0x40, 0x40, 0x7C, 0x40, 0x40, 0x7E, 0x00],
            'F' | 'f' => [0x7E, 0x40, 0x40, 0x7C, 0x40, 0x40, 0x40, 0x00],
            'G' | 'g' => [0x3C, 0x42, 0x40, 0x4E, 0x42, 0x42, 0x3C, 0x00],
            'H' | 'h' => [0x42, 0x42, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00],
            'I' | 'i' => [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00],
            'J' | 'j' => [0x3E, 0x04, 0x04, 0x04, 0x44, 0x44, 0x38, 0x00],
            'K' | 'k' => [0x42, 0x44, 0x48, 0x70, 0x48, 0x44, 0x42, 0x00],
            'L' | 'l' => [0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7E, 0x00],
            'M' | 'm' => [0x42, 0x66, 0x5A, 0x42, 0x42, 0x42, 0x42, 0x00],
            'N' | 'n' => [0x42, 0x62, 0x52, 0x4A, 0x46, 0x42, 0x42, 0x00],
            'O' | 'o' => [0x3C, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00],
            'P' | 'p' => [0x7C, 0x42, 0x42, 0x7C, 0x40, 0x40, 0x40, 0x00],
            'Q' | 'q' => [0x3C, 0x42, 0x42, 0x52, 0x4A, 0x44, 0x3A, 0x00],
            'R' | 'r' => [0x7C, 0x42, 0x42, 0x7C, 0x48, 0x44, 0x42, 0x00],
            'S' | 's' => [0x3C, 0x42, 0x40, 0x3C, 0x02, 0x42, 0x3C, 0x00],
            'T' | 't' => [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00],
            'U' | 'u' => [0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00],
            'V' | 'v' => [0x42, 0x42, 0x42, 0x42, 0x24, 0x24, 0x18, 0x00],
            'W' | 'w' => [0x42, 0x42, 0x42, 0x42, 0x5A, 0x66, 0x42, 0x00],
            'X' | 'x' => [0x42, 0x24, 0x18, 0x18, 0x18, 0x24, 0x42, 0x00],
            'Y' | 'y' => [0x42, 0x42, 0x24, 0x18, 0x18, 0x18, 0x18, 0x00],
            'Z' | 'z' => [0x7E, 0x04, 0x08, 0x10, 0x20, 0x40, 0x7E, 0x00],
            '0' => [0x3C, 0x46, 0x4A, 0x52, 0x52, 0x62, 0x3C, 0x00],
            '1' => [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00],
            '2' => [0x3C, 0x42, 0x02, 0x3C, 0x40, 0x40, 0x7E, 0x00],
            '3' => [0x3C, 0x42, 0x02, 0x1C, 0x02, 0x42, 0x3C, 0x00],
            '4' => [0x08, 0x18, 0x28, 0x48, 0x7E, 0x08, 0x08, 0x00],
            '5' => [0x7E, 0x40, 0x7C, 0x02, 0x02, 0x42, 0x3C, 0x00],
            '6' => [0x3C, 0x40, 0x40, 0x7C, 0x42, 0x42, 0x3C, 0x00],
            '7' => [0x7E, 0x02, 0x04, 0x08, 0x10, 0x20, 0x20, 0x00],
            '8' => [0x3C, 0x42, 0x42, 0x3C, 0x42, 0x42, 0x3C, 0x00],
            '9' => [0x3C, 0x42, 0x42, 0x3E, 0x02, 0x02, 0x3C, 0x00],
            '©' => [0x3C, 0x42, 0x9D, 0xA1, 0xA1, 0x9D, 0x42, 0x3C],
            '@' => [0x3C, 0x42, 0x9A, 0xAA, 0x9E, 0x80, 0x7C, 0x00],
            ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x00],
            ',' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x10, 0x20],
            ':' => [0x00, 0x00, 0x18, 0x00, 0x00, 0x18, 0x00, 0x00],
            '-' => [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00],
            '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF],
            _ => [0x3C, 0x42, 0x99, 0xA1, 0xA1, 0x99, 0x42, 0x3C], // 默认问号图案
        }
    }

    /// 混合像素
    fn blend_pixel(&self, base_pixel: &mut Rgba<u8>, overlay: &Rgba<u8>) {
        let alpha = overlay.0[3] as f32 / 255.0;
        let inv_alpha = 1.0 - alpha;

        for i in 0..3 {
            base_pixel.0[i] = (base_pixel.0[i] as f32 * inv_alpha + overlay.0[i] as f32 * alpha) as u8;
        }
        base_pixel.0[3] = (base_pixel.0[3] as f32 + overlay.0[3] as f32 * alpha).min(255.0) as u8;
    }

    /// 混合两个图像
    fn blend_images(
        &self,
        base: &mut RgbaImage,
        watermark: &RgbaImage,
        x: u32,
        y: u32,
        opacity: f32,
    ) -> Result<()> {
        for (wm_x, wm_y, wm_pixel) in watermark.enumerate_pixels() {
            let base_x = x + wm_x;
            let base_y = y + wm_y;

            if base_x >= base.width() || base_y >= base.height() {
                continue;
            }

            let base_pixel = base.get_pixel_mut(base_x, base_y);
            let mut overlay_pixel = *wm_pixel;
            overlay_pixel.0[3] = (overlay_pixel.0[3] as f32 * opacity) as u8;

            self.blend_pixel(base_pixel, &overlay_pixel);
        }

        Ok(())
    }

    /// 创建版权水印预设
    #[allow(dead_code)]
    pub fn create_copyright_watermark(author: &str) -> SimpleTextWatermark {
        SimpleTextWatermark {
            text: format!("© {}", author),
            font_size: 16,
            color: Rgba([255, 255, 255, 200]),
            position: WatermarkPosition::BottomRight,
            opacity: 0.8,
            margin: 15,
            background: Some(Rgba([0, 0, 0, 80])),
            letter_spacing: 1.0, // 版权水印使用较小间距
        }
    }

    /// 创建品牌水印预设
    #[allow(dead_code)]
    pub fn create_brand_watermark(brand_name: &str) -> SimpleTextWatermark {
        SimpleTextWatermark {
            text: brand_name.to_string(),
            font_size: 24,
            color: Rgba([200, 200, 200, 180]),
            position: WatermarkPosition::BottomCenter,
            opacity: 0.6,
            margin: 30,
            background: None,
            letter_spacing: 3.0, // 品牌水印使用较大间距，更显眼
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_calculation() {
        let processor = SimpleWatermarkProcessor::new();
        let (x, y) = processor.calculate_position(1000, 800, 100, 50, WatermarkPosition::BottomRight, 20);

        assert_eq!(x, 880); // 1000 - 100 - 20
        assert_eq!(y, 730); // 800 - 50 - 20
    }

    #[test]
    fn test_watermark_presets() {
        let copyright = SimpleWatermarkProcessor::create_copyright_watermark("TestAuthor");
        assert!(copyright.text.contains("TestAuthor"));

        let brand = SimpleWatermarkProcessor::create_brand_watermark("BrandName");
        assert_eq!(brand.text, "BrandName");
    }

    #[test]
    fn test_char_patterns() {
        let processor = SimpleWatermarkProcessor::new();
        let pattern_a = processor.get_char_pattern('A');
        let pattern_space = processor.get_char_pattern(' ');

        assert_ne!(pattern_a, [0; 8]);
        assert_eq!(pattern_space, [0; 8]);
    }
}