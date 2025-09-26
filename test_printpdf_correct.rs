// 测试printpdf的正确用法 - 解决图片缩放问题
use anyhow::Result;
use printpdf::{PdfDocument, PdfDocumentReference, PdfLayerIndex, PdfPageIndex, Mm, Px, ImageXObject, Image, ImageTransform, ColorSpace, ColorBits};
use image::DynamicImage;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

fn main() -> Result<()> {
    println!("🧪 测试printpdf正确用法");

    // 加载测试图片
    let test_image_path = "test_watermark_output.png"; // 使用已有的测试图片
    if !Path::new(test_image_path).exists() {
        println!("❌ 测试图片不存在: {}", test_image_path);
        return Ok(());
    }

    let image = image::open(test_image_path)?;
    println!("📸 加载图片: {}x{}", image.width(), image.height());

    // 测试不同的缩放方法
    test_method_1(&image, "method1_default_transform.pdf")?;
    test_method_2(&image, "method2_calculated_scale.pdf")?;
    test_method_3(&image, "method3_full_page_fit.pdf")?;

    println!("✅ 所有测试完成");
    Ok(())
}

/// 方法1: 使用默认ImageTransform (当前有问题的方法)
fn test_method_1(image: &DynamicImage, output_name: &str) -> Result<()> {
    println!("\n🔬 方法1: 默认ImageTransform");

    let (doc, page1, layer1) = PdfDocument::new("测试1", Mm(210.0), Mm(297.0), "Layer 1");

    // 转换为RGB并创建ImageXObject
    let rgb_image = image.to_rgb8();
    let image_data = rgb_image.as_raw().clone();

    let image_xobject = ImageXObject {
        width: Px(image.width() as usize),
        height: Px(image.height() as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data,
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    };

    let pdf_image = Image::from(image_xobject);
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // 使用默认变换 (这是问题所在)
    pdf_image.add_to_layer(current_layer, ImageTransform::default());

    let file = File::create(output_name)?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer)?;

    println!("💾 已保存: {}", output_name);
    Ok(())
}

/// 方法2: 手动计算缩放比例
fn test_method_2(image: &DynamicImage, output_name: &str) -> Result<()> {
    println!("\n🔬 方法2: 手动计算缩放");

    let page_width_mm = 210.0;  // A4宽度
    let page_height_mm = 297.0; // A4高度

    let (doc, page1, layer1) = PdfDocument::new("测试2", Mm(page_width_mm), Mm(page_height_mm), "Layer 1");

    // 转换为RGB并创建ImageXObject
    let rgb_image = image.to_rgb8();
    let image_data = rgb_image.as_raw().clone();

    let image_xobject = ImageXObject {
        width: Px(image.width() as usize),
        height: Px(image.height() as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data,
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    };

    let pdf_image = Image::from(image_xobject);
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // 🎯 关键修复: 正确计算缩放比例
    // printpdf中，缩放是 输出毫米 / 输入像素
    let scale_x = page_width_mm / image.width() as f32;
    let scale_y = page_height_mm / image.height() as f32;

    println!("📐 图片尺寸: {}x{} px", image.width(), image.height());
    println!("📐 页面尺寸: {:.1}x{:.1} mm", page_width_mm, page_height_mm);
    println!("📐 计算缩放: scale_x={:.4}, scale_y={:.4}", scale_x, scale_y);

    pdf_image.add_to_layer(current_layer, ImageTransform {
        translate_x: Some(Mm(0.0)),
        translate_y: Some(Mm(0.0)),
        scale_x: Some(scale_x),
        scale_y: Some(scale_y),
        ..Default::default()
    });

    let file = File::create(output_name)?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer)?;

    println!("💾 已保存: {}", output_name);
    Ok(())
}

/// 方法3: 保持宽高比的页面填充
fn test_method_3(image: &DynamicImage, output_name: &str) -> Result<()> {
    println!("\n🔬 方法3: 保持宽高比填充页面");

    let page_width_mm = 210.0;  // A4宽度
    let page_height_mm = 297.0; // A4高度

    let (doc, page1, layer1) = PdfDocument::new("测试3", Mm(page_width_mm), Mm(page_height_mm), "Layer 1");

    // 转换为RGB并创建ImageXObject
    let rgb_image = image.to_rgb8();
    let image_data = rgb_image.as_raw().clone();

    let image_xobject = ImageXObject {
        width: Px(image.width() as usize),
        height: Px(image.height() as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data,
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    };

    let pdf_image = Image::from(image_xobject);
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // 🎯 关键修复: 保持宽高比的最佳填充
    let scale_x = page_width_mm / image.width() as f32;
    let scale_y = page_height_mm / image.height() as f32;

    // 使用较小的缩放比例以保持宽高比，图片完整显示
    let uniform_scale = scale_x.min(scale_y);

    // 计算居中位置
    let scaled_width = image.width() as f32 * uniform_scale;
    let scaled_height = image.height() as f32 * uniform_scale;
    let center_x = (page_width_mm - scaled_width) / 2.0;
    let center_y = (page_height_mm - scaled_height) / 2.0;

    println!("📐 图片尺寸: {}x{} px", image.width(), image.height());
    println!("📐 页面尺寸: {:.1}x{:.1} mm", page_width_mm, page_height_mm);
    println!("📐 统一缩放: {:.4}", uniform_scale);
    println!("📐 缩放后尺寸: {:.1}x{:.1} mm", scaled_width, scaled_height);
    println!("📐 居中位置: ({:.1}, {:.1}) mm", center_x, center_y);

    pdf_image.add_to_layer(current_layer, ImageTransform {
        translate_x: Some(Mm(center_x)),
        translate_y: Some(Mm(center_y)),
        scale_x: Some(uniform_scale),
        scale_y: Some(uniform_scale),
        ..Default::default()
    });

    let file = File::create(output_name)?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer)?;

    println!("💾 已保存: {}", output_name);
    Ok(())
}