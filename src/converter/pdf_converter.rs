use anyhow::{anyhow, Context, Result};
use image::DynamicImage;
use pdfium_render::prelude::*;
use std::path::Path; // --- 【API修复】引入 Path 类型 ---

/// 将PDF文件转换为一系列图像。
pub fn convert_pdf_to_images(
    pdf_path: &Path,
    dpi: f32,
) -> Result<Vec<DynamicImage>> {
    // --- 【线程安全修复】在函数内部创建 Pdfium 实例，确保线程安全 ---
    // 1. 获取库的绑定
    let bindings = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
        .or_else(|_| Pdfium::bind_to_system_library())
        .context("❌ PDFium库初始化失败！\n\n📋 解决方案：\n1. 下载pdfium.dll文件到程序目录\n2. 下载地址: https://github.com/bblanchon/pdfium-binaries/releases\n3. 选择Windows x64版本\n4. 解压后将pdfium.dll复制到exe同目录\n\n或者重新编译程序使用静态链接")?;
    
    // 2. 使用绑定来创建一个 Pdfium 实例
    let pdfium = Pdfium::new(bindings);

    // 加载 PDF 文档
    let document = pdfium.load_pdf_from_file(pdf_path, None)
        .map_err(|e| anyhow!("无法加载PDF文件 '{}': {}", pdf_path.display(), e))?;

    let render_config = PdfRenderConfig::new()
        .scale_page_by_factor(dpi / 72.0);

    // --- 【内存优化】改为逐页处理，避免一次性加载所有页面到内存 ---
    let mut images = Vec::new();

    for (page_index, page) in document.pages().iter().enumerate() {
        match page.render_with_config(&render_config) {
            Ok(bitmap) => {
                let image = bitmap.as_image();
                images.push(image);
                // 每10页输出一次进度信息，避免日志过多
                let total_pages = document.pages().len() as usize;
                if page_index % 10 == 0 || page_index == total_pages - 1 {
                    println!("  📄 已渲染页面 {}/{}", page_index + 1, total_pages);
                }
            },
            Err(e) => {
                eprintln!("⚠️  跳过页面 {}: 渲染失败 - {}", page_index + 1, e);
                continue; // 跳过有问题的页面，继续处理其他页面
            }
        }
    }

    if images.is_empty() {
        Err(anyhow!("PDF文件 '{}' 中没有可渲染的页面。", pdf_path.display()))
    } else {
        Ok(images)
    }
}

/// 快速获取PDF文件的总页数
pub fn get_pdf_page_count(pdf_path: &Path) -> Result<usize> {
    // --- 【线程安全修复】同样在函数内部创建独立的 Pdfium 实例 ---
    let bindings = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
        .or_else(|_| Pdfium::bind_to_system_library())
        .context("❌ PDFium库初始化失败！请下载pdfium.dll到程序目录")?;
    let pdfium = Pdfium::new(bindings);
    
    let document = pdfium.load_pdf_from_file(pdf_path, None)
        .map_err(|e| anyhow!("无法加载PDF文件 '{}': {}", pdf_path.display(), e))?;
    
    // --- 【类型修复】将 u16 转换为 usize ---
    Ok(document.pages().len().into())
}