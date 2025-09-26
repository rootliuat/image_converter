// 简单图片转PDF实现 - 直接使用printpdf
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use printpdf::*;
use image::DynamicImage;

/// 简单PDF转换器
pub struct SimpleImageToPdf;

impl SimpleImageToPdf {
    /// 将图片转换为PDF
    pub fn convert_image_to_pdf(
        image_path: &Path,
        output_path: &Path,
    ) -> Result<()> {
        println!("🖼️  转换图片: {} -> {}", image_path.display(), output_path.display());

        // 读取图片
        let image = image::open(image_path)
            .with_context(|| format!("无法读取图片文件: {}", image_path.display()))?;

        // 创建PDF文档
        let (doc, page1, layer1) = PdfDocument::new("Image to PDF", Mm(210.0), Mm(297.0), "Layer 1");

        // 添加图片到PDF
        Self::add_image_to_page(&doc, &page1, &layer1, &image)?;

        // 保存PDF
        doc.save(&mut std::io::BufWriter::new(std::fs::File::create(output_path)?))
            .with_context(|| "保存PDF文件失败")?;

        println!("✅ 转换完成: {}", output_path.display());
        Ok(())
    }

    /// 添加图片到PDF页面
    fn add_image_to_page(
        doc: &PdfDocumentReference,
        page: &PdfPageIndex,
        layer: &PdfLayerIndex,
        image: &DynamicImage,
    ) -> Result<()> {
        // 转换为RGB8格式
        let rgb_image = image.to_rgb8();
        let (width, height) = rgb_image.dimensions();

        // 创建图片对象
        let image_obj = ImageXObject {
            width: Px(width as usize),
            height: Px(height as usize),
            color_space: ColorSpace::Rgb,
            bits_per_component: ColorBits::Bit8,
            interpolate: true,
            image_data: rgb_image.into_raw(),
            image_filter: None,
            clipping_bbox: None,
            smask: None,
        };

        // 计算缩放以适应A4页面 - 简化计算
        let page_width_pts = 595.0; // A4宽度点数
        let page_height_pts = 842.0; // A4高度点数

        let scale_x = page_width_pts * 0.9 / width as f64; // 留10%边距
        let scale_y = page_height_pts * 0.9 / height as f64;
        let scale = scale_x.min(scale_y);

        let scaled_width = width as f64 * scale;
        let scaled_height = height as f64 * scale;

        // 居中定位
        let x = (page_width_pts - scaled_width) / 2.0;
        let y = (page_height_pts - scaled_height) / 2.0;

        // 添加图片到页面
        let current_layer = doc.get_page(*page).get_layer(*layer);

        let transform = ImageTransform {
            translate_x: Some(Pt(x)),
            translate_y: Some(Pt(y)),
            scale_x: Some(scale),
            scale_y: Some(scale),
            rotate: None,
            skew_x: None,
            skew_y: None,
        };

        image_obj.add_to_layer(current_layer, transform);

        Ok(())
    }

    /// 批量转换文件夹中的图片
    pub fn convert_folder_to_pdf(
        folder_path: &Path,
        output_path: &Path,
    ) -> Result<()> {
        println!("📁 扫描文件夹: {}", folder_path.display());

        let image_files = Self::get_image_files(folder_path)?;
        if image_files.is_empty() {
            anyhow::bail!("文件夹中没有找到图片文件");
        }

        println!("📸 找到 {} 张图片", image_files.len());

        // 创建PDF文档
        let (doc, page1, layer1) = PdfDocument::new("Images to PDF", Mm(210.0), Mm(297.0), "Layer 1");
        let mut current_page = page1;
        let mut current_layer = layer1;

        for (i, image_path) in image_files.iter().enumerate() {
            println!("📊 处理 {}/{}: {}", i + 1, image_files.len(), image_path.display());

            match image::open(image_path) {
                Ok(image) => {
                    // 为除第一张图片外的其他图片添加新页面
                    if i > 0 {
                        let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                        current_page = page;
                        current_layer = layer;
                    }

                    if let Err(e) = Self::add_image_to_page(&doc, &current_page, &current_layer, &image) {
                        eprintln!("⚠️  跳过图片 {}: {}", image_path.display(), e);
                    }
                },
                Err(e) => {
                    eprintln!("⚠️  跳过图片 {}: {}", image_path.display(), e);
                }
            }
        }

        // 保存PDF
        doc.save(&mut std::io::BufWriter::new(std::fs::File::create(output_path)?))
            .with_context(|| "保存PDF文件失败")?;

        println!("🎉 批量转换完成: {}", output_path.display());
        Ok(())
    }

    /// 获取文件夹中的图片文件
    fn get_image_files(folder_path: &Path) -> Result<Vec<PathBuf>> {
        let supported_extensions = ["jpg", "jpeg", "png", "bmp", "tiff", "webp", "gif"];
        let mut image_files = Vec::new();

        for entry in std::fs::read_dir(folder_path)? {
            let entry = entry.with_context(|| "遍历文件夹失败")?;

            if entry.file_type()?.is_file() {
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
}