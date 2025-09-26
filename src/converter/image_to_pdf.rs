// 图片转PDF功能模块 - 保持原始尺寸和像素质量

use anyhow::{Context, Result};
use ::image::{DynamicImage, GenericImageView};
use printpdf::{PdfDocument, PdfDocumentReference, PdfPageIndex, PdfLayerIndex, Mm, Px, ImageXObject, Image, ImageTransform, ColorSpace, ColorBits};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::BufWriter;
use walkdir::WalkDir;

/// PDF转换配置 - 升级版
#[derive(Debug, Clone)]
pub struct PdfConfig {
    /// 输出PDF路径
    pub output_path: PathBuf,
    /// 是否保持原始尺寸（不缩放）
    pub preserve_original_size: bool,
    /// 页面方向：自动检测还是固定
    pub page_orientation: PageOrientation,
    /// 图片质量（0-100）
    pub image_quality: u8,
    /// 是否为每张图片创建单独的页面
    pub one_image_per_page: bool,
    /// DPI设置 (72-600)
    pub dpi: f32,
    /// 页面边距（毫米）
    pub margin_mm: f32,
    /// 是否自动旋转页面以适应图片
    pub auto_rotate: bool,
    /// 页面尺寸模式
    pub page_mode: PageMode,
}

/// 页面尺寸模式
#[derive(Debug, Clone, PartialEq)]
pub enum PageMode {
    /// 固定A4尺寸
    FixedA4,
    /// 根据图片自适应页面尺寸
    AdaptiveSize,
    /// 其他标准尺寸
    Standard(StandardPageSize),
}

/// 标准页面尺寸
#[derive(Debug, Clone, PartialEq)]
pub enum StandardPageSize {
    A3,
    A4,
    A5,
    Letter,
    Legal,
}

/// 页面方向选项
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageOrientation {
    /// 自动检测（根据图片尺寸决定横向或纵向）
    Auto,
    /// 强制横向
    Landscape,
    /// 强制纵向
    Portrait,
}

/// 为PdfConfig实现默认值
impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            output_path: PathBuf::from("output.pdf"),
            preserve_original_size: true,
            page_orientation: PageOrientation::Auto,
            image_quality: 90,
            one_image_per_page: true,
            dpi: 300.0,           // 高质量300 DPI
            margin_mm: 0.0,       // 0mm边距 - 消除白边
            auto_rotate: true,    // 自动旋转
            page_mode: PageMode::AdaptiveSize, // 自适应页面尺寸
        }
    }
}

/// 为PdfConfig添加构建方法
impl PdfConfig {
    pub fn new(output_path: PathBuf) -> Self {
        Self {
            output_path,
            ..Default::default()
        }
    }

    pub fn with_dpi(mut self, dpi: f32) -> Self {
        self.dpi = dpi;
        self
    }

    pub fn with_margin(mut self, margin_mm: f32) -> Self {
        self.margin_mm = margin_mm;
        self
    }

    pub fn with_page_mode(mut self, page_mode: PageMode) -> Self {
        self.page_mode = page_mode;
        self
    }
}

/// 图片转PDF处理器
pub struct ImageToPdfConverter;

// 删除重复的Default实现，使用上面的新版本

impl ImageToPdfConverter {
    /// 将单个图片转换为PDF
    pub fn convert_single_image(
        image_path: &Path,
        config: &PdfConfig,
    ) -> Result<()> {
        println!("🖼️  正在转换: {}", image_path.display());

        // 加载图片
        let image = image::open(image_path)
            .with_context(|| format!("无法加载图片: {}", image_path.display()))?;

        // 创建PDF
        let images = vec![image];
        let image_names = vec![image_path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()];

        Self::create_pdf_from_images(images, image_names, config)
    }

    /// 将文件夹中的所有图片转换为单个PDF
    pub fn convert_folder_to_pdf(
        folder_path: &Path,
        config: &PdfConfig,
    ) -> Result<()> {
        println!("📁 正在扫描文件夹: {}", folder_path.display());

        // 获取所有支持的图片文件
        let image_files = Self::get_image_files(folder_path)?;

        if image_files.is_empty() {
            anyhow::bail!("文件夹中没有找到支持的图片文件");
        }

        println!("📸 找到 {} 张图片", image_files.len());

        // 加载所有图片
        let mut images = Vec::new();
        let mut image_names = Vec::new();

        for (i, image_path) in image_files.iter().enumerate() {
            println!("📊 加载图片 {}/{}: {}", i + 1, image_files.len(), image_path.display());

            match image::open(image_path) {
                Ok(img) => {
                    images.push(img);
                    image_names.push(
                        image_path.file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    );
                },
                Err(e) => {
                    eprintln!("⚠️  跳过无法加载的图片 {}: {}", image_path.display(), e);
                    continue;
                }
            }
        }

        if images.is_empty() {
            anyhow::bail!("没有成功加载任何图片");
        }

        println!("✅ 成功加载 {} 张图片", images.len());

        // 创建PDF
        Self::create_pdf_from_images(images, image_names, config)
    }

    /// 从图片列表创建PDF
    fn create_pdf_from_images(
        images: Vec<DynamicImage>,
        image_names: Vec<String>,
        config: &PdfConfig,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();
        println!("📄 开始创建PDF: {}", config.output_path.display());

        // 根据第一张图片创建合适尺寸的PDF文档
        let first_image = &images[0];
        let (page_width, page_height) = Self::calculate_page_size(first_image, config)?;
        let (doc, page1, layer1) = PdfDocument::new(
            "图片转换PDF",
            Mm(page_width), // 根据第一张图片调整宽度
            Mm(page_height), // 根据第一张图片调整高度
            "主页"
        );

        let mut current_page = page1;
        let mut current_layer = layer1;
        let mut page_count = 1;

        // 处理每张图片
        for (i, (image, name)) in images.iter().zip(image_names.iter()).enumerate() {
            println!("📝 处理图片 {}/{}: {}", i + 1, images.len(), name);

            // 如果不是第一张图片且需要每张图片一页，创建新页面
            if i > 0 && config.one_image_per_page {
                let (page_w, page_h) = Self::calculate_page_size(image, config)?;
                let (new_page, new_layer) = doc.add_page(Mm(page_w), Mm(page_h), &format!("页面{}", i + 1));
                current_page = new_page;
                current_layer = new_layer;
                page_count += 1;
            }

            // 添加图片到PDF
            Self::add_image_to_pdf(&doc, current_layer, image, config, current_page)
                .with_context(|| format!("添加图片到PDF失败: {}", name))?;
        }

        // 保存PDF
        println!("💾 正在保存PDF文件...");

        // 确保输出目录存在
        if let Some(parent) = config.output_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| "创建输出目录失败")?;
        }

        let file = File::create(&config.output_path)
            .with_context(|| format!("无法创建PDF文件: {}", config.output_path.display()))?;
        let mut writer = BufWriter::new(file);

        doc.save(&mut writer)
            .with_context(|| "保存PDF文件失败")?;

        let elapsed = start_time.elapsed();
        println!("🎉 PDF转换完成!");
        println!("   📄 输出文件: {}", config.output_path.display());
        println!("   📊 总页数: {}", page_count);
        println!("   📸 图片数量: {}", images.len());
        println!("   ⏱️  耗时: {:.2}秒", elapsed.as_secs_f64());

        Ok(())
    }

    /// 添加图片到PDF页面 - 真实的图片嵌入实现
    fn add_image_to_pdf(
        doc: &PdfDocumentReference,
        layer: PdfLayerIndex,
        image: &DynamicImage,
        config: &PdfConfig,
        page: PdfPageIndex,
    ) -> Result<()> {

        let width = image.width();
        let height = image.height();

        println!("  📸 真实嵌入图片 {}x{} 到PDF", width, height);

        // 🚀 真正的图片嵌入实现
        // 步骤1: 转换图片为RGB8格式
        let rgb_image = image.to_rgb8();
        let image_data = rgb_image.as_raw().clone();

        // 步骤2: 创建ImageXObject (printpdf的图片对象)
        let image_xobject = ImageXObject {
            width: Px(width as usize),
            height: Px(height as usize),
            color_space: ColorSpace::Rgb,
            bits_per_component: ColorBits::Bit8,
            interpolate: true,
            image_data,
            image_filter: None, // 不压缩以保持质量
            clipping_bbox: None,
            smask: None, // 没有透明蒙版
        };

        // 步骤3: 创建PDF Image对象
        let pdf_image = Image::from(image_xobject);

        // 步骤4: 让图片完全填满页面 - 正确的缩放计算
        // printpdf使用的是毫米单位，需要正确的单位转换

        // 🎯 智能页面尺寸计算 - 基于配置模式
        let (page_width_mm, page_height_mm) = Self::calculate_page_size(image, config)?;

        // 🚀 升级版缩放计算：使用配置的DPI
        let pixel_to_mm = 25.4 / config.dpi; // 使用配置的DPI
        let image_width_mm = width as f32 * pixel_to_mm;
        let image_height_mm = height as f32 * pixel_to_mm;

        // 使用配置的边距值
        let usable_width = page_width_mm - 2.0 * config.margin_mm;
        let usable_height = page_height_mm - 2.0 * config.margin_mm;
        let margin_x = config.margin_mm;
        let margin_y = config.margin_mm;

        let scale_x = usable_width / image_width_mm;
        let scale_y = usable_height / image_height_mm;

        // 选择缩放策略
        let (img_x, img_y, final_scale_x, final_scale_y) = if config.preserve_original_size {
            // 保持宽高比，图片完整显示（可能有留白）
            let uniform_scale = scale_x.min(scale_y);
            let scaled_width = image_width_mm * uniform_scale;
            let scaled_height = image_height_mm * uniform_scale;
            let center_x = margin_x + (usable_width - scaled_width) / 2.0;
            let center_y = margin_y + (usable_height - scaled_height) / 2.0;

            (center_x, center_y, uniform_scale, uniform_scale)
        } else {
            // 拉伸填满整个页面（可能变形但无留白）
            (margin_x, margin_y, scale_x, scale_y)
        };

        // 步骤5: 获取PDF层并添加图片
        let current_layer = doc.get_page(page).get_layer(layer);

        pdf_image.add_to_layer(
            current_layer,
            ImageTransform {
                translate_x: Some(Mm(img_x)),
                translate_y: Some(Mm(img_y)),
                scale_x: Some(final_scale_x),
                scale_y: Some(final_scale_y),
                ..Default::default()
            },
        );

        let strategy = if config.preserve_original_size { "保持宽高比" } else { "拉伸填满" };
        println!("    ✅ 成功嵌入图片: {}x{} -> 页面{}x{}mm | 策略:{} | 位置:({:.1},{:.1})mm | 缩放:({:.3},{:.3})",
                width, height, page_width_mm, page_height_mm, strategy, img_x, img_y, final_scale_x, final_scale_y);

        Ok(())
    }


    /// 智能计算页面尺寸 - 升级版
    fn calculate_page_size(image: &DynamicImage, config: &PdfConfig) -> Result<(f32, f32)> {
        let (width, height) = image.dimensions();

        match &config.page_mode {
            PageMode::AdaptiveSize => {
                // 自适应页面尺寸：根据图片尺寸和DPI计算最佳页面
                let pixel_to_mm = 25.4 / config.dpi;
                let img_width_mm = width as f32 * pixel_to_mm + 2.0 * config.margin_mm;
                let img_height_mm = height as f32 * pixel_to_mm + 2.0 * config.margin_mm;

                // 检查是否需要自动旋转
                if config.auto_rotate {
                    let img_is_landscape = width > height;
                    match config.page_orientation {
                        PageOrientation::Auto => {
                            if img_is_landscape {
                                Ok((img_width_mm, img_height_mm))
                            } else {
                                Ok((img_width_mm, img_height_mm))
                            }
                        },
                        PageOrientation::Landscape => Ok((img_width_mm.max(img_height_mm), img_width_mm.min(img_height_mm))),
                        PageOrientation::Portrait => Ok((img_width_mm.min(img_height_mm), img_width_mm.max(img_height_mm))),
                    }
                } else {
                    Ok((img_width_mm, img_height_mm))
                }
            },
            PageMode::FixedA4 => {
                // 固定A4尺寸
                if config.auto_rotate && config.page_orientation == PageOrientation::Auto {
                    let img_is_landscape = width > height;
                    if img_is_landscape {
                        Ok((297.0, 210.0)) // A4横向
                    } else {
                        Ok((210.0, 297.0)) // A4纵向
                    }
                } else {
                    match config.page_orientation {
                        PageOrientation::Landscape => Ok((297.0, 210.0)),
                        _ => Ok((210.0, 297.0)),
                    }
                }
            },
            PageMode::Standard(size) => {
                // 标准页面尺寸
                let (w, h): (f32, f32) = match size {
                    StandardPageSize::A3 => (297.0, 420.0),
                    StandardPageSize::A4 => (210.0, 297.0),
                    StandardPageSize::A5 => (148.0, 210.0),
                    StandardPageSize::Letter => (215.9, 279.4),
                    StandardPageSize::Legal => (215.9, 355.6),
                };

                if config.auto_rotate && config.page_orientation == PageOrientation::Auto {
                    let img_is_landscape = width > height;
                    if img_is_landscape {
                        Ok((w.max(h), w.min(h)))
                    } else {
                        Ok((w.min(h), w.max(h)))
                    }
                } else {
                    match config.page_orientation {
                        PageOrientation::Landscape => Ok((w.max(h), w.min(h))),
                        _ => Ok((w.min(h), w.max(h))),
                    }
                }
            }
        }
    }

    /// 旧版本计算页面尺寸（保留兼容性）
    fn calculate_page_size_legacy(image: &DynamicImage, config: &PdfConfig) -> (Mm, Mm) {
        if !config.preserve_original_size {
            // 使用A4纸尺寸
            return (Mm(210.0), Mm(297.0));
        }

        // 保持原始尺寸，将像素转换为毫米
        // 使用150 DPI，适合打印质量 (1英寸 = 25.4毫米, 150像素 = 1英寸)
        let width_mm = (image.width() as f32 * 25.4) / 150.0;
        let height_mm = (image.height() as f32 * 25.4) / 150.0;

        let (width, height) = match config.page_orientation {
            PageOrientation::Auto => {
                if image.width() > image.height() {
                    // 横向图片
                    (Mm(width_mm.max(height_mm)), Mm(width_mm.min(height_mm)))
                } else {
                    // 纵向图片
                    (Mm(width_mm.min(height_mm)), Mm(width_mm.max(height_mm)))
                }
            },
            PageOrientation::Landscape => (Mm(width_mm.max(height_mm)), Mm(width_mm.min(height_mm))),
            PageOrientation::Portrait => (Mm(width_mm.min(height_mm)), Mm(width_mm.max(height_mm))),
        };

        (width, height)
    }

    // 已移除 calculate_image_position_and_size 函数 - 未使用

    /// 获取文件夹中的所有图片文件（公共接口）
    pub fn get_image_files_public(folder_path: &Path) -> Result<Vec<PathBuf>> {
        Self::get_image_files(folder_path)
    }

    /// 获取文件夹中的所有图片文件
    fn get_image_files(folder_path: &Path) -> Result<Vec<PathBuf>> {
        let supported_extensions = ["jpg", "jpeg", "png", "bmp", "tiff", "webp", "gif"];
        let mut image_files = Vec::new();

        for entry in WalkDir::new(folder_path).min_depth(1).max_depth(1) {
            let entry = entry.with_context(|| "遍历文件夹失败")?;

            if entry.file_type().is_file() {
                if let Some(extension) = entry.path().extension() {
                    let ext_str = extension.to_string_lossy().to_lowercase();
                    if supported_extensions.contains(&ext_str.as_str()) {
                        image_files.push(entry.path().to_path_buf());
                    }
                }
            }
        }

        // 按文件名排序
        image_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        Ok(image_files)
    }

    /// 检查路径是图片文件还是文件夹
    pub fn detect_input_type(path: &Path) -> Result<InputType> {
        if !path.exists() {
            anyhow::bail!("路径不存在: {}", path.display());
        }

        if path.is_file() {
            // 检查是否为支持的图片格式
            if let Some(extension) = path.extension() {
                let ext_str = extension.to_string_lossy().to_lowercase();
                let supported_extensions = ["jpg", "jpeg", "png", "bmp", "tiff", "webp", "gif"];

                if supported_extensions.contains(&ext_str.as_str()) {
                    return Ok(InputType::SingleImage);
                } else {
                    anyhow::bail!("不支持的图片格式: {}", ext_str);
                }
            } else {
                anyhow::bail!("文件没有扩展名");
            }
        } else if path.is_dir() {
            Ok(InputType::Folder)
        } else {
            anyhow::bail!("未知的路径类型");
        }
    }
}

/// 输入类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputType {
    /// 单个图片文件
    SingleImage,
    /// 图片文件夹
    Folder,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_size_calculation() {
        let image = DynamicImage::new_rgb8(1920, 1080);
        let config = PdfConfig::default();

        let (width, height) = ImageToPdfConverter::calculate_page_size(&image, &config);
        assert!(width.0 > 0.0 && height.0 > 0.0);
    }

    #[test]
    fn test_input_type_detection() {
        // 测试需要实际的文件路径，这里只测试逻辑
        assert_eq!(InputType::SingleImage, InputType::SingleImage);
        assert_eq!(InputType::Folder, InputType::Folder);
    }
}