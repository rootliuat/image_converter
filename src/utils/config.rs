use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::converter::simple_watermark::{WatermarkPosition, SimpleTextWatermark, ImageWatermark};
use image::Rgba;

/// 应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 默认输入路径
    pub default_input_path: String,
    /// 默认输出路径
    pub default_output_path: String,
    /// 默认目标文件大小（KB）
    pub default_target_size: u32,
    /// 默认输出格式
    pub default_output_format: OutputFormat,
    /// 默认压缩模式
    pub default_compression_mode: CompressionMode,
    /// 默认处理模式
    pub default_processing_mode: ProcessingMode,
    /// 默认应用模式
    pub default_app_mode: AppMode,
    /// 窗口设置
    pub window_settings: WindowSettings,
    /// 高级设置
    pub advanced_settings: AdvancedSettings,
    /// 水印设置
    pub watermark_settings: WatermarkSettings,
    /// PDF转换设置
    pub pdf_settings: PdfSettings,
}

/// 压缩模式配置
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CompressionMode {
    /// 压缩到指定大小
    SizeTarget,
    /// 原始质量（无压缩）
    Original,
}

/// 输出格式配置
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    Jpeg,
    /// PNG格式（压缩到目标大小）
    PngCompressed,
    /// PNG格式（原始质量，无压缩）
    PngOriginal,
    /// WebP格式（有损压缩，现代高效）
    WebPLossy,
    /// WebP格式（无损压缩，比PNG更小）
    WebPLossless,
}

/// 处理模式配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProcessingMode {
    SingleFile,
    Folder,
}

/// 应用模式配置
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AppMode {
    /// 图片格式转换模式
    ImageConverter,
    /// 图片转PDF模式
    ImageToPdf,
    /// PDF转图片模式
    PdfToImage,
    /// 纯水印模式（不压缩，只添加水印）
    PureWatermark,
}

/// 窗口设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSettings {
    /// 窗口宽度
    pub width: f32,
    /// 窗口高度
    pub height: f32,
    /// 是否最大化
    pub maximized: bool,
    /// 窗口位置
    pub position: Option<(f32, f32)>,
}

/// 高级设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSettings {
    /// JPEG质量范围
    pub jpeg_quality_range: (u8, u8),
    /// PNG压缩级别
    pub png_compression_level: u8,
    /// PDF渲染DPI
    pub pdf_render_dpi: f32,
    /// 最大并发处理数量
    pub max_concurrent_jobs: usize,
    /// 是否保留原始文件
    pub keep_original_files: bool,
    /// 是否显示详细进度
    pub show_detailed_progress: bool,
    /// 自动打开输出文件夹
    pub auto_open_output_folder: bool,
}

/// 水印设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkSettings {
    /// 是否启用文字水印
    pub enable_text_watermark: bool,
    /// 文字水印内容
    pub text_content: String,
    /// 文字大小
    pub text_size: u32,
    /// 文字颜色 (RGBA)
    pub text_color: [u8; 4],
    /// 文字透明度 (0.0-1.0)
    pub text_opacity: f32,
    /// 文字位置
    pub text_position: WatermarkPosition,
    /// 文字边距
    pub text_margin: u32,
    /// 是否启用图片水印
    pub enable_image_watermark: bool,
    /// 图片水印路径
    pub image_watermark_path: String,
    /// 图片水印缩放比例
    pub image_scale: f32,
    /// 图片水印透明度
    pub image_opacity: f32,
    /// 图片水印位置
    pub image_position: WatermarkPosition,
    /// 图片水印边距
    pub image_margin: u32,
}

/// PDF转换设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfSettings {
    /// 是否保持原始尺寸
    pub preserve_original_size: bool,
    /// 页面方向
    pub page_orientation: PdfPageOrientation,
    /// 图片质量 (0-100)
    pub image_quality: u8,
    /// 每张图片单独页面
    pub one_image_per_page: bool,
    /// 默认输出PDF文件名
    pub default_output_name: String,
}

/// PDF页面方向
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PdfPageOrientation {
    /// 自动检测
    Auto,
    /// 横向
    Landscape,
    /// 纵向
    Portrait,
}

/// 获取应用程序数据目录
fn get_app_data_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("ImageConverter");
    path
}

/// 获取默认输出路径字符串
fn get_default_output_path() -> String {
    let mut output_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    output_dir.push("output");
    output_dir.to_string_lossy().to_string()
}


impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_input_path: String::new(),
            default_output_path: get_default_output_path(),
            default_target_size: 400,
            default_output_format: OutputFormat::PngCompressed,
            default_compression_mode: CompressionMode::SizeTarget,
            default_processing_mode: ProcessingMode::SingleFile,
            default_app_mode: AppMode::ImageConverter,
            window_settings: WindowSettings::default(),
            advanced_settings: AdvancedSettings::default(),
            watermark_settings: WatermarkSettings::default(),
            pdf_settings: PdfSettings::default(),
        }
    }
}

impl Default for CompressionMode {
    fn default() -> Self {
        CompressionMode::SizeTarget
    }
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            maximized: false,
            position: None,
        }
    }
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            jpeg_quality_range: (10, 95),
            png_compression_level: 6,
            pdf_render_dpi: 150.0,
            max_concurrent_jobs: 4,
            keep_original_files: true,
            show_detailed_progress: true,
            auto_open_output_folder: false,
        }
    }
}

impl Default for WatermarkSettings {
    fn default() -> Self {
        Self {
            enable_text_watermark: false,
            text_content: "Copyright".to_string(),
            text_size: 24,
            text_color: [255, 255, 255, 200], // 半透明白色
            text_opacity: 0.8,
            text_position: WatermarkPosition::BottomRight,
            text_margin: 20,
            enable_image_watermark: false,
            image_watermark_path: String::new(),
            image_scale: 0.2,
            image_opacity: 0.8,
            image_position: WatermarkPosition::BottomRight,
            image_margin: 20,
        }
    }
}

impl AppConfig {
    /// 从文件加载配置
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("无法读取配置文件: {}", path.as_ref().display()))?;

        let config: AppConfig =
            serde_json::from_str(&content).with_context(|| "无法解析配置文件")?;

        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self).with_context(|| "无法序列化配置")?;

        // 确保配置文件目录存在
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("无法创建配置目录: {}", parent.display()))?;
        }

        std::fs::write(path.as_ref(), content)
            .with_context(|| format!("无法写入配置文件: {}", path.as_ref().display()))?;

        Ok(())
    }

    /// 获取配置文件路径
    pub fn get_config_path() -> PathBuf {
        let mut config_dir = get_app_data_dir();
        config_dir.push("config.json");
        config_dir
    }

    /// 加载或创建默认配置
    pub fn load_or_default() -> Self {
        // 首先尝试项目根目录的配置文件
        let project_config = PathBuf::from("config.json");
        if project_config.exists() {
            match Self::load_from_file(&project_config) {
                Ok(config) => {
                    println!("✅ 已加载项目配置文件: {}", project_config.display());
                    return config;
                }
                Err(e) => {
                    println!("⚠️  项目配置文件解析失败: {}", e);
                }
            }
        }

        // 然后尝试用户目录的配置文件
        let config_path = Self::get_config_path();
        if config_path.exists() {
            match Self::load_from_file(&config_path) {
                Ok(config) => {
                    println!("✅ 已加载用户配置文件: {}", config_path.display());
                    config
                }
                Err(e) => {
                    println!("⚠️  用户配置文件解析失败: {}，使用默认配置", e);
                    Self::default()
                }
            }
        } else {
            println!("📋 配置文件不存在，使用默认配置");
            Self::default()
        }
    }

    /// 保存当前配置
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path();
        self.save_to_file(config_path)
    }

    // 已移除 reset_to_default 方法 - 未使用
    // 已移除 validate 方法 - 未使用
}

impl WatermarkSettings {
    /// 转换为文字水印配置
    pub fn to_text_watermark(&self) -> SimpleTextWatermark {
        SimpleTextWatermark {
            text: self.text_content.clone(),
            font_size: self.text_size,
            color: Rgba(self.text_color),
            position: self.text_position,
            opacity: self.text_opacity,
            margin: self.text_margin,
            background: None,
        }
    }

    /// 转换为图片水印配置
    pub fn to_image_watermark(&self) -> ImageWatermark {
        ImageWatermark {
            watermark_path: self.image_watermark_path.clone(),
            position: self.image_position,
            opacity: self.image_opacity,
            scale: self.image_scale,
            margin: self.image_margin,
        }
    }
}

impl OutputFormat {
    /// 获取文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Jpeg => "jpg",
            OutputFormat::PngCompressed | OutputFormat::PngOriginal => "png",
            OutputFormat::WebPLossy | OutputFormat::WebPLossless => "webp",
        }
    }

    // 已移除 display_name 方法 - 未使用

    /// 获取所有可用格式
    pub fn all_formats() -> Vec<(Self, &'static str)> {
        vec![
            (OutputFormat::Jpeg, "JPEG"),
            (OutputFormat::PngCompressed, "PNG (压缩)"),
            (OutputFormat::PngOriginal, "PNG (原始)"),
            (OutputFormat::WebPLossy, "WebP (有损)"),
            (OutputFormat::WebPLossless, "WebP (无损)"),
        ]
    }

    // 已移除 is_png 方法 - 未使用
    // 已移除 needs_compression 方法 - 未使用
}

impl CompressionMode {
    // 已移除 display_name 方法 - 未使用

    // 已移除 all_modes 方法 - 未使用
}

impl ProcessingMode {
    // 已移除 display_name 方法 - 未使用

    /// 获取所有可用模式
    pub fn all_modes() -> Vec<(Self, &'static str)> {
        vec![
            (ProcessingMode::SingleFile, "单个文件"),
            (ProcessingMode::Folder, "文件夹"),
        ]
    }
}

impl Default for PdfSettings {
    fn default() -> Self {
        Self {
            preserve_original_size: true,
            page_orientation: PdfPageOrientation::Auto,
            image_quality: 90,
            one_image_per_page: true,
            default_output_name: "converted.pdf".to_string(),
        }
    }
}

impl AppMode {
    // 已移除 display_name 方法 - 未使用

    /// 获取所有可用模式
    pub fn all_modes() -> Vec<(Self, &'static str)> {
        vec![
            (AppMode::ImageConverter, "图片格式转换"),
            (AppMode::ImageToPdf, "图片转PDF"),
            (AppMode::PdfToImage, "PDF转图片"),
            (AppMode::PureWatermark, "纯水印模式"),
        ]
    }
}

impl PdfPageOrientation {
    // 已移除 display_name 方法 - 未使用

    /// 获取所有可用选项
    pub fn all_orientations() -> Vec<(Self, &'static str)> {
        vec![
            (PdfPageOrientation::Auto, "自动检测"),
            (PdfPageOrientation::Landscape, "横向"),
            (PdfPageOrientation::Portrait, "纵向"),
        ]
    }
}