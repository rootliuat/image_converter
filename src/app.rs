use crate::converter::batch_processor::BatchProcessor;
use crate::converter::image_to_pdf::{ImageToPdfConverter, PdfConfig, InputType, PageOrientation};
use crate::converter::simple_watermark::WatermarkPosition;
use crate::ui::{components, styles, menu_bar};
use crate::utils::config::{AppConfig, OutputFormat, ProcessingMode, AppMode, PdfPageOrientation};
use crate::utils::file_utils;
use eframe::egui;
use rfd::FileDialog;
use std::path::Path;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Default)]
pub struct ProgressUpdate {
    pub processed: usize,
    pub failed: usize,
    pub total: usize,
    pub current_file: String,
    pub is_complete: bool,
    pub error_message: Option<String>,
}

pub struct ImageConverterApp {
    config: AppConfig,
    input_path: String,
    output_path: String,
    status_message: String,
    is_error: bool,
    is_processing: bool,
    progress: ProgressUpdate,
    tokio_runtime: tokio::runtime::Runtime,
    progress_receiver: mpsc::UnboundedReceiver<ProgressUpdate>,
    progress_sender: mpsc::UnboundedSender<ProgressUpdate>,
    menu_bar_state: menu_bar::MenuBarState,
    #[allow(dead_code)]
    last_button_click: std::time::Instant,
}

impl ImageConverterApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::load_or_default();
        let (progress_sender, progress_receiver) = mpsc::unbounded_channel();
        let mut menu_bar_state = menu_bar::MenuBarState::default();
        menu_bar_state.current_mode = config.default_app_mode;

        Self {
            input_path: config.default_input_path.clone(),
            output_path: config.default_output_path.clone(),
            config,
            status_message: "欢迎使用pdf/图片格式转换工具！".to_string(),
            is_error: false,
            is_processing: false,
            progress: ProgressUpdate::default(),
            last_button_click: std::time::Instant::now(),
            tokio_runtime: tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap(),
            progress_receiver,
            progress_sender,
            menu_bar_state,
        }
    }

    fn start_processing(&mut self) {
        if self.is_processing { return; }
        if self.input_path.trim().is_empty() || self.output_path.trim().is_empty() {
            self.status_message = "错误：输入和输出路径不能为空".to_string();
            self.is_error = true;
            return;
        }
        self.is_processing = true;
        self.is_error = false;
        self.progress = ProgressUpdate::default();
        self.status_message = "正在准备处理...".to_string();
        let (input_path, output_path, config, progress_sender) = (
            self.input_path.clone().into(),
            self.output_path.clone().into(),
            self.config.clone(),
            self.progress_sender.clone(),
        );
        match self.config.default_app_mode {
            AppMode::ImageConverter => {
                self.tokio_runtime.spawn(async move {
                    BatchProcessor::process_files(
                        input_path,
                        output_path,
                        config.default_target_size,
                        config.default_output_format,
                        config.default_processing_mode,
                        config.watermark_settings,
                        progress_sender,
                    ).await;
                });
            },
            AppMode::ImageToPdf => {
                self.tokio_runtime.spawn(async move {
                    Self::process_pdf_conversion(
                        input_path,
                        output_path,
                        config,
                        progress_sender,
                    ).await;
                });
            },
            AppMode::PdfToImage => {
                self.tokio_runtime.spawn(async move {
                    Self::process_pdf_to_image_conversion(
                        input_path,
                        output_path,
                        config,
                        progress_sender,
                    ).await;
                });
            },
            AppMode::PureWatermark => {
                self.tokio_runtime.spawn(async move {
                    Self::process_pure_watermark(
                        input_path,
                        output_path,
                        config,
                        progress_sender,
                    ).await;
                });
            },
        }
    }

    /// PDF转换处理函数
    async fn process_pdf_conversion(
        input_path: std::path::PathBuf,
        output_path: std::path::PathBuf,
        config: AppConfig,
        progress_sender: tokio::sync::mpsc::UnboundedSender<ProgressUpdate>,
    ) {
        let _ = progress_sender.send(ProgressUpdate {
            current_file: "正在准备PDF转换...".to_string(),
            ..Default::default()
        });

        // 创建PDF配置
        let pdf_config = PdfConfig {
            output_path: {
                let mut path = output_path.clone();
                path.push(&config.pdf_settings.default_output_name);
                path
            },
            preserve_original_size: config.pdf_settings.preserve_original_size,
            page_orientation: match config.pdf_settings.page_orientation {
                PdfPageOrientation::Auto => PageOrientation::Auto,
                PdfPageOrientation::Landscape => PageOrientation::Landscape,
                PdfPageOrientation::Portrait => PageOrientation::Portrait,
            },
            image_quality: config.pdf_settings.image_quality,
            one_image_per_page: config.pdf_settings.one_image_per_page,
        };

        // 执行转换
        let result = tokio::task::spawn_blocking(move || {
            match ImageToPdfConverter::detect_input_type(&input_path) {
                Ok(InputType::SingleImage) => {
                    ImageToPdfConverter::convert_single_image(&input_path, &pdf_config)
                },
                Ok(InputType::Folder) => {
                    ImageToPdfConverter::convert_folder_to_pdf(&input_path, &pdf_config)
                },
                Err(e) => Err(e),
            }
        }).await;

        match result {
            Ok(Ok(())) => {
                let _ = progress_sender.send(ProgressUpdate {
                    processed: 1,
                    total: 1,
                    is_complete: true,
                    current_file: "PDF转换完成".to_string(),
                    ..Default::default()
                });
            },
            Ok(Err(e)) => {
                let _ = progress_sender.send(ProgressUpdate {
                    is_complete: true,
                    error_message: Some(format!("PDF转换失败: {}", e)),
                    ..Default::default()
                });
            },
            Err(_) => {
                let _ = progress_sender.send(ProgressUpdate {
                    is_complete: true,
                    error_message: Some("PDF转换失败: 未知错误".to_string()),
                    ..Default::default()
                });
            },
        }
    }

    /// PDF转图片处理函数
    async fn process_pdf_to_image_conversion(
        input_path: std::path::PathBuf,
        output_path: std::path::PathBuf,
        config: AppConfig,
        progress_sender: tokio::sync::mpsc::UnboundedSender<ProgressUpdate>,
    ) {
        use crate::converter::pdf_converter;
        use crate::converter::image_converter;

        let _ = progress_sender.send(ProgressUpdate {
            current_file: "正在准备PDF转图片...".to_string(),
            ..Default::default()
        });

        let progress_sender_clone = progress_sender.clone();
        let result = tokio::task::spawn_blocking(move || {
            let progress_sender = progress_sender_clone;
            let dpi = config.advanced_settings.pdf_render_dpi;

            // 根据处理模式确定要处理的PDF文件列表
            let pdf_files: Vec<std::path::PathBuf> = match config.default_processing_mode {
                ProcessingMode::SingleFile => {
                    // 单文件模式：检查是否为PDF文件
                    if input_path.extension().and_then(|s| s.to_str()) != Some("pdf") {
                        return Err(anyhow::anyhow!("输入文件不是PDF格式"));
                    }
                    vec![input_path]
                },
                ProcessingMode::Folder => {
                    // 文件夹模式：遍历文件夹中的所有PDF文件
                    if !input_path.is_dir() {
                        return Err(anyhow::anyhow!("输入路径不是文件夹"));
                    }

                    let mut pdf_files = Vec::new();
                    for entry in std::fs::read_dir(&input_path)
                        .map_err(|e| anyhow::anyhow!("无法读取文件夹 {}: {}", input_path.display(), e))?
                    {
                        let entry = entry?;
                        let path = entry.path();
                        if path.is_file() &&
                           path.extension().and_then(|s| s.to_str()) == Some("pdf") {
                            pdf_files.push(path);
                        }
                    }

                    if pdf_files.is_empty() {
                        return Err(anyhow::anyhow!("文件夹中没有找到PDF文件"));
                    }

                    // 按文件名排序，确保处理顺序一致
                    pdf_files.sort();
                    pdf_files
                }
            };

            println!("🔄 开始PDF转图片转换，找到 {} 个PDF文件", pdf_files.len());

            // 预计算总页数用于进度条
            let mut total_pages = 0;
            for pdf_file in pdf_files.iter() {
                match pdf_converter::get_pdf_page_count(pdf_file) {
                    Ok(count) => {
                        total_pages += count;
                        println!("📄 PDF文件 {} 有 {} 页", pdf_file.display(), count);
                    },
                    Err(e) => {
                        eprintln!("⚠️  无法获取PDF页数 {}: {}，跳过", pdf_file.display(), e);
                    }
                }
            }

            // 发送总页数用于进度条显示
            progress_sender.send(ProgressUpdate {
                total: total_pages,
                current_file: format!("开始处理 {} 个PDF文件，共 {} 页", pdf_files.len(), total_pages),
                ..Default::default()
            })?;

            // 确保输出目录存在
            std::fs::create_dir_all(&output_path)?;

            let mut total_processed = 0;
            for (file_index, pdf_file) in pdf_files.iter().enumerate() {
                println!("📄 处理第 {} 个PDF: {}", file_index + 1, pdf_file.display());

                let images = pdf_converter::convert_pdf_to_images(pdf_file, dpi)?;
                println!("✅ 成功渲染 {} 页", images.len());

                // 为每个PDF文件创建子文件夹（如果是批量处理）
                let file_output_dir = if pdf_files.len() > 1 {
                    let file_stem = pdf_file.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    output_path.join(file_stem)
                } else {
                    output_path.clone()
                };

                std::fs::create_dir_all(&file_output_dir)?;

                // 基于libavif策略：批量并行处理，但控制并发数
                let num_cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
                let _max_parallel = (num_cores * 3 / 4).max(2).min(8); // 限制最大并发数

                use std::sync::{Arc, Mutex};
                use std::sync::atomic::{AtomicUsize, Ordering};

                let processed_counter = Arc::new(AtomicUsize::new(0));
                let progress_sender_shared = Arc::new(Mutex::new(progress_sender.clone()));

                // 🚀 高性能分块并行处理 - 基于现代图像处理技术
                use rayon::prelude::*;

                // 动态分块大小：根据内存和CPU核心数优化
                let chunk_size = if images.len() > 100 {
                    (num_cores * 4).max(8).min(32) // 大量页面：更大块
                } else {
                    (num_cores * 2).max(4).min(16) // 少量页面：较小块
                };

                println!("🧠 智能分块处理：{} 页面分为 {} 大小的块，使用 {} 个CPU核心",
                         images.len(), chunk_size, num_cores);

                // 分块并行处理 - 避免内存峰值
                images.par_chunks(chunk_size).enumerate().try_for_each(|(chunk_idx, chunk)| -> anyhow::Result<()> {
                    chunk.par_iter().enumerate().try_for_each(|(local_page_num, image)| -> anyhow::Result<()> {
                        let global_page_num = chunk_idx * chunk_size + local_page_num;
                        let output_file = file_output_dir.join(format!("page_{:03}.{}",
                            global_page_num + 1,
                            config.default_output_format.extension()
                        ));

                        // 使用配置的输出格式和质量设置
                        image_converter::compress_and_save(
                            image,
                            &output_file,
                            config.default_target_size,
                            config.default_output_format
                        )?;

                        // 原子更新计数器
                        let current_processed = processed_counter.fetch_add(1, Ordering::SeqCst) + 1;

                        // 安全发送进度更新（每10页更新一次，减少锁竞争）
                        if current_processed % 10 == 0 || current_processed == images.len() {
                            if let Ok(sender) = progress_sender_shared.lock() {
                                let _ = sender.send(ProgressUpdate {
                                    processed: total_processed + current_processed,
                                    total: total_pages,
                                current_file: format!("并行处理 {} ({}/{}页)",
                                                    pdf_file.file_name().unwrap_or_default().to_string_lossy(),
                                                    current_processed, images.len()),
                                ..Default::default()
                            });
                        }
                    }

                        Ok(())
                    })
                })?;

                total_processed += images.len();
            }

            println!("🎉 PDF转图片完成! 共处理 {} 页", total_processed);
            Ok(total_processed)
        }).await;

        match result {
            Ok(Ok(count)) => {
                let _ = progress_sender.send(ProgressUpdate {
                    processed: count,
                    total: count,
                    is_complete: true,
                    current_file: format!("PDF转图片完成，共生成 {} 张图片", count),
                    ..Default::default()
                });
            },
            Ok(Err(e)) => {
                let _ = progress_sender.send(ProgressUpdate {
                    is_complete: true,
                    error_message: Some(format!("PDF转图片失败: {}", e)),
                    ..Default::default()
                });
            },
            Err(_) => {
                let _ = progress_sender.send(ProgressUpdate {
                    is_complete: true,
                    error_message: Some("PDF转图片失败: 任务执行错误".to_string()),
                    ..Default::default()
                });
            },
        }
    }

    /// 纯水印处理函数（不压缩，保持原画质）
    async fn process_pure_watermark(
        input_path: std::path::PathBuf,
        output_path: std::path::PathBuf,
        config: AppConfig,
        progress_sender: tokio::sync::mpsc::UnboundedSender<ProgressUpdate>,
    ) {
        use crate::converter::simple_watermark::SimpleWatermarkProcessor;

        let _ = progress_sender.send(ProgressUpdate {
            current_file: "正在准备纯水印处理...".to_string(),
            ..Default::default()
        });

        let result = tokio::task::spawn_blocking(move || {
            // 检查是否启用了水印
            if !config.watermark_settings.enable_text_watermark && !config.watermark_settings.enable_image_watermark {
                return Err(anyhow::anyhow!("请至少启用一种水印类型"));
            }

            // 获取要处理的图片文件列表
            let image_files: Vec<std::path::PathBuf> = match config.default_processing_mode {
                ProcessingMode::SingleFile => {
                    // 单文件模式：检查是否为支持的图片格式
                    if let Some(ext) = input_path.extension().and_then(|s| s.to_str()) {
                        let ext = ext.to_lowercase();
                        if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tiff") {
                            vec![input_path]
                        } else {
                            return Err(anyhow::anyhow!("不支持的图片格式: {}", ext));
                        }
                    } else {
                        return Err(anyhow::anyhow!("无法识别文件格式"));
                    }
                },
                ProcessingMode::Folder => {
                    // 文件夹模式：遍历文件夹中的所有图片文件
                    if !input_path.is_dir() {
                        return Err(anyhow::anyhow!("输入路径不是文件夹"));
                    }

                    let mut image_files = Vec::new();
                    for entry in std::fs::read_dir(&input_path)
                        .map_err(|e| anyhow::anyhow!("无法读取文件夹 {}: {}", input_path.display(), e))?
                    {
                        let entry = entry?;
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                                let ext = ext.to_lowercase();
                                if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tiff") {
                                    image_files.push(path);
                                }
                            }
                        }
                    }

                    if image_files.is_empty() {
                        return Err(anyhow::anyhow!("文件夹中没有找到支持的图片文件"));
                    }

                    // 按文件名排序，确保处理顺序一致
                    image_files.sort();
                    image_files
                }
            };

            println!("💧 开始纯水印处理，找到 {} 个图片文件", image_files.len());

            // 确保输出目录存在
            std::fs::create_dir_all(&output_path)?;

            let watermark_processor = SimpleWatermarkProcessor::new();
            let mut processed_count = 0;

            for (file_index, image_file) in image_files.iter().enumerate() {
                println!("🖼️ 处理第 {} 个图片: {}", file_index + 1, image_file.display());

                // 加载原始图片
                let original_image = image::open(image_file)
                    .map_err(|e| anyhow::anyhow!("无法打开图片 '{}': {}", image_file.display(), e))?;

                let mut processed_image = original_image.clone();

                // 添加文字水印
                if config.watermark_settings.enable_text_watermark {
                    let text_watermark = config.watermark_settings.to_text_watermark();
                    processed_image = watermark_processor.add_text_watermark(processed_image, &text_watermark)?;
                }

                // 添加图片水印
                if config.watermark_settings.enable_image_watermark && !config.watermark_settings.image_watermark_path.is_empty() {
                    let image_watermark = config.watermark_settings.to_image_watermark();
                    processed_image = watermark_processor.add_image_watermark(processed_image, &image_watermark)?;
                }

                // 保存处理后的图片（保持原始格式和质量）
                let file_name = image_file.file_name().unwrap_or_default();
                let output_file = output_path.join(file_name);

                // 直接保存，不进行任何压缩
                processed_image.save(&output_file)
                    .map_err(|e| anyhow::anyhow!("保存图片失败 '{}': {}", output_file.display(), e))?;

                processed_count += 1;
                println!("✨ 已保存水印图片: {}", output_file.display());
            }

            println!("🎉 纯水印处理完成! 共处理 {} 张图片", processed_count);
            Ok(processed_count)
        }).await;

        match result {
            Ok(Ok(count)) => {
                let _ = progress_sender.send(ProgressUpdate {
                    processed: count,
                    total: count,
                    is_complete: true,
                    current_file: format!("纯水印处理完成，共处理 {} 张图片", count),
                    ..Default::default()
                });
            },
            Ok(Err(e)) => {
                let _ = progress_sender.send(ProgressUpdate {
                    is_complete: true,
                    error_message: Some(format!("纯水印处理失败: {}", e)),
                    ..Default::default()
                });
            },
            Err(_) => {
                let _ = progress_sender.send(ProgressUpdate {
                    is_complete: true,
                    error_message: Some("纯水印处理失败: 任务执行错误".to_string()),
                    ..Default::default()
                });
            },
        }
    }

    /// 显示图片转换设置界面
    fn show_image_converter_settings(&mut self, ui: &mut egui::Ui) {
        components::parameter_group(ui, "2. 设置参数 🖼️", |ui| {
            ui.label(styles::emphasis_text("🔧 当前模式：图片格式转换"));
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                // 格式选择器
                components::format_selector(ui, "输出格式", &mut self.config.default_output_format, &OutputFormat::all_formats());
                ui.add_space(20.0);

                // 根据格式动态显示不同的控件
                match self.config.default_output_format {
                    OutputFormat::Jpeg | OutputFormat::PngCompressed | OutputFormat::WebPLossy => {
                        // 显示目标大小控件
                        components::number_input_with_unit(ui, "目标大小", &mut self.config.default_target_size, "KB", 10, 10240);
                    },
                    OutputFormat::PngOriginal => {
                        // 显示原始质量提示
                        ui.label(
                            egui::RichText::new("✨ 原始质量，无压缩")
                                .color(egui::Color32::from_rgb(100, 200, 100))
                                .size(12.0)
                        );
                    },
                    OutputFormat::WebPLossless => {
                        // 显示WebP无损提示
                        ui.label(
                            egui::RichText::new("🚀 WebP无损，比PNG更小")
                                .color(egui::Color32::from_rgb(50, 150, 250))
                                .size(12.0)
                        );
                    }
                }

                ui.add_space(20.0);
                components::format_selector(ui, "处理模式", &mut self.config.default_processing_mode, &ProcessingMode::all_modes());
            });
        });
    }

    /// 显示PDF转换设置界面
    fn show_pdf_converter_settings(&mut self, ui: &mut egui::Ui) {
        components::parameter_group(ui, "2. PDF设置 📄", |ui| {
            ui.label(styles::emphasis_text("🔧 当前模式：图片转PDF"));
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                components::format_selector(ui, "处理模式", &mut self.config.default_processing_mode, &ProcessingMode::all_modes());
                ui.add_space(20.0);
                components::format_selector(ui, "页面方向", &mut self.config.pdf_settings.page_orientation, &PdfPageOrientation::all_orientations());
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.config.pdf_settings.preserve_original_size, "保持原始尺寸");
                ui.add_space(20.0);
                ui.checkbox(&mut self.config.pdf_settings.one_image_per_page, "每张图片单独页面");
                ui.add_space(20.0);
                ui.label("图片质量:");
                ui.add(egui::Slider::new(&mut self.config.pdf_settings.image_quality, 10..=100).text("%"));
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("输出文件名:");
                ui.add(egui::TextEdit::singleline(&mut self.config.pdf_settings.default_output_name).desired_width(200.0));
            });
        });
    }

    /// 显示PDF转图片设置界面
    fn show_pdf_to_image_settings(&mut self, ui: &mut egui::Ui) {
        components::parameter_group(ui, "2. PDF转图片设置 📄➡️🖼️", |ui| {
            ui.label(styles::emphasis_text("🔧 当前模式：PDF转图片"));
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                components::format_selector(ui, "处理模式", &mut self.config.default_processing_mode, &ProcessingMode::all_modes());
                ui.add_space(20.0);
                components::format_selector(ui, "输出格式", &mut self.config.default_output_format, &OutputFormat::all_formats());
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("渲染DPI:");
                ui.add(egui::Slider::new(&mut self.config.advanced_settings.pdf_render_dpi, 72.0..=600.0).text("DPI"));

                // 添加DPI说明提示
                ui.label("💡");
                if ui.label("ℹ️").hovered() {
                    egui::show_tooltip_text(ui.ctx(), egui::Id::new("dpi_tooltip"),
                        "DPI设置说明:\n• 72 DPI: 网页显示质量，文件小\n• 150 DPI: 普通打印质量 (推荐)\n• 300 DPI: 高质量打印\n• 600 DPI: 超高质量，文件大");
                }

                ui.add_space(20.0);

                // 根据输出格式显示不同的控件
                match self.config.default_output_format {
                    OutputFormat::PngOriginal => {
                        ui.label("✨ 原始质量，无压缩");
                    },
                    _ => {
                        components::number_input_with_unit(
                            ui,
                            "目标大小",
                            &mut self.config.default_target_size,
                            "KB",
                            10,
                            10240
                        );
                    }
                }
            });
        });
    }

    /// 显示水印设置界面
    fn show_watermark_settings(&mut self, ui: &mut egui::Ui) {
        components::parameter_group(ui, "3. 水印设置", |ui| {
            // 文字水印设置
            ui.checkbox(&mut self.config.watermark_settings.enable_text_watermark, "启用文字水印");
            if self.config.watermark_settings.enable_text_watermark {
                ui.indent("text_watermark", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("内容:");
                        ui.add(egui::TextEdit::singleline(&mut self.config.watermark_settings.text_content).desired_width(120.0));
                        ui.add_space(10.0);
                        ui.label("大小:");
                        ui.add(egui::DragValue::new(&mut self.config.watermark_settings.text_size).speed(1.0).clamp_range(8..=200));
                        ui.add_space(10.0);
                        ui.label("透明度:");
                        ui.add(egui::Slider::new(&mut self.config.watermark_settings.text_opacity, 0.1..=1.0).text(""));
                    });
                    ui.horizontal(|ui| {
                        components::format_selector(ui, "文字位置", &mut self.config.watermark_settings.text_position, &WatermarkPosition::all_positions());
                    });
                });
            }

            ui.add_space(8.0);

            // 图片水印设置
            ui.checkbox(&mut self.config.watermark_settings.enable_image_watermark, "启用图片水印");
            if self.config.watermark_settings.enable_image_watermark {
                ui.indent("image_watermark", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("路径:");
                        ui.add(egui::TextEdit::singleline(&mut self.config.watermark_settings.image_watermark_path).desired_width(200.0));
                        if ui.button("选择").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("图片", &["png", "jpg", "jpeg", "bmp", "tiff"])
                                .pick_file() {
                                self.config.watermark_settings.image_watermark_path = path.to_string_lossy().to_string();
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("缩放:");
                        ui.add(egui::Slider::new(&mut self.config.watermark_settings.image_scale, 0.1..=2.0).text(""));
                        ui.add_space(10.0);
                        ui.label("透明度:");
                        ui.add(egui::Slider::new(&mut self.config.watermark_settings.image_opacity, 0.1..=1.0).text(""));
                    });
                    ui.horizontal(|ui| {
                        components::format_selector(ui, "图片位置", &mut self.config.watermark_settings.image_position, &WatermarkPosition::all_positions());
                    });
                });
            }
        });
    }

    /// 显示纯水印模式设置界面
    fn show_pure_watermark_settings(&mut self, ui: &mut egui::Ui) {
        components::parameter_group(ui, "2. 纯水印设置 💧（保持原画质）", |ui| {
            ui.label(styles::emphasis_text("🎨 当前模式：纯水印（不压缩、不改变尺寸）"));
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                components::format_selector(ui, "处理模式", &mut self.config.default_processing_mode, &ProcessingMode::all_modes());
            });

            ui.add_space(10.0);

            ui.label(styles::subheading_text("🎨 输出说明："));
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.vertical(|ui| {
                    ui.label("• 保持原始文件格式和质量");
                    ui.label("• 只添加水印，不进行任何压缩");
                    ui.label("• 支持PNG、JPEG、WebP等格式");
                    ui.label("• 完美保持原始画质");
                });
            });
        });

        ui.add_space(10.0);

        // 集成水印设置
        components::parameter_group(ui, "3. 水印配置", |ui| {
            // 文字水印设置
            ui.checkbox(&mut self.config.watermark_settings.enable_text_watermark, "🔤 启用文字水印");
            if self.config.watermark_settings.enable_text_watermark {
                ui.indent("text_watermark", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("内容:");
                        ui.add(egui::TextEdit::singleline(&mut self.config.watermark_settings.text_content).desired_width(120.0));
                        ui.add_space(10.0);
                        ui.label("大小:");
                        ui.add(egui::DragValue::new(&mut self.config.watermark_settings.text_size).speed(1.0).clamp_range(8..=200));
                        ui.add_space(10.0);
                        ui.label("透明度:");
                        ui.add(egui::Slider::new(&mut self.config.watermark_settings.text_opacity, 0.1..=1.0).text(""));
                    });
                    ui.horizontal(|ui| {
                        components::format_selector(ui, "文字位置", &mut self.config.watermark_settings.text_position, &WatermarkPosition::all_positions());
                    });
                });
            }

            ui.add_space(8.0);

            // 图片水印设置
            ui.checkbox(&mut self.config.watermark_settings.enable_image_watermark, "🖼️ 启用图片水印");
            if self.config.watermark_settings.enable_image_watermark {
                ui.indent("image_watermark", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("路径:");
                        ui.add(egui::TextEdit::singleline(&mut self.config.watermark_settings.image_watermark_path).desired_width(200.0));
                        if ui.button("选择").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("图片", &["png", "jpg", "jpeg", "bmp", "tiff"])
                                .pick_file() {
                                self.config.watermark_settings.image_watermark_path = path.to_string_lossy().to_string();
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("缩放:");
                        ui.add(egui::Slider::new(&mut self.config.watermark_settings.image_scale, 0.1..=2.0).text(""));
                        ui.add_space(10.0);
                        ui.label("透明度:");
                        ui.add(egui::Slider::new(&mut self.config.watermark_settings.image_opacity, 0.1..=1.0).text(""));
                    });
                    ui.horizontal(|ui| {
                        components::format_selector(ui, "图片位置", &mut self.config.watermark_settings.image_position, &WatermarkPosition::all_positions());
                    });
                });
            }

            ui.add_space(10.0);

            if !self.config.watermark_settings.enable_text_watermark && !self.config.watermark_settings.enable_image_watermark {
                ui.label(styles::error_text("⚠️ 请至少启用一种水印类型"));
            }
        });
    }

    /// 处理菜单栏动作
    fn handle_menu_action(&mut self, action: menu_bar::MenuAction, ctx: &egui::Context) {
        use menu_bar::MenuAction;

        match action {
            MenuAction::None => {},
            MenuAction::NewProject => {
                self.input_path.clear();
                self.output_path.clear();
                self.status_message = "新建项目".to_string();
                self.is_error = false;
                self.progress = ProgressUpdate::default();
            },
            MenuAction::OpenFile => {
                if let Some(path) = FileDialog::new().pick_file() {
                    self.input_path = path.to_string_lossy().to_string();
                    self.status_message = format!("已选择文件: {}",
                        path.file_name().unwrap_or_default().to_string_lossy());
                }
            },
            MenuAction::SaveAs => {
                if let Some(path) = FileDialog::new().save_file() {
                    self.output_path = path.parent()
                        .unwrap_or_else(|| Path::new("."))
                        .to_string_lossy()
                        .to_string();
                    self.status_message = "已设置输出路径".to_string();
                }
            },
            MenuAction::Exit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            },
            MenuAction::ImageConverter => {
                self.config.default_app_mode = AppMode::ImageConverter;
                self.status_message = "切换到图片格式转换模式".to_string();
            },
            MenuAction::ImageToPdf => {
                self.config.default_app_mode = AppMode::ImageToPdf;
                self.status_message = "切换到图片转PDF模式".to_string();
            },
            MenuAction::About | MenuAction::Settings => {
                // 这些动作由菜单栏组件内部处理
            },
        }
    }
}

impl eframe::App for ImageConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理菜单栏操作
        let menu_action = menu_bar::draw_menu_bar(ctx, &mut self.menu_bar_state);
        self.handle_menu_action(menu_action, ctx);

        // 双向同步应用模式状态
        if self.config.default_app_mode != self.menu_bar_state.current_mode {
            // 优先使用配置状态（中央按钮点击后的状态）
            self.menu_bar_state.current_mode = self.config.default_app_mode;
        }

        // 优化进度更新，限制频繁更新
        let mut update_count = 0;
        while let Ok(update) = self.progress_receiver.try_recv() {
            self.progress = update.clone();
            update_count += 1;

            // 限制每次UI更新最多处理10个进度消息，避免UI卡顿
            if update_count >= 10 {
                break;
            }
        }

        // 根据最新进度更新状态
        if update_count > 0 {
            if self.progress.is_complete {
                self.is_processing = false;
                self.status_message = format!("处理完成！成功: {}, 失败: {}.", self.progress.processed, self.progress.failed);
                if let Some(err) = &self.progress.error_message {
                    self.status_message.push_str(&format!(" 出现错误: {}", err));
                    self.is_error = true;
                }
            } else if self.is_processing {
                let current_progress = self.progress.processed + self.progress.failed;
                self.status_message = format!("正在处理: {} ({}/{})",
                    self.progress.current_file,
                    current_progress,
                    self.progress.total);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                let title = match self.config.default_app_mode {
                    AppMode::ImageConverter => "图片格式转换与压缩工具",
                    AppMode::ImageToPdf => "图片转PDF工具",
                    AppMode::PdfToImage => "PDF转图片工具",
                    AppMode::PureWatermark => "纯水印处理工具（原画质）",
                };
                ui.label(styles::heading_text(title));
                ui.add_space(5.0);
                ui.label(styles::emphasis_text("高效、易用，支持批量处理"));
            });
            ui.add_space(15.0);

            // 应用模式选择
            components::parameter_group(ui, "功能模式", |ui| {

                let mode_changed = components::format_selector(ui, "选择功能", &mut self.config.default_app_mode, &AppMode::all_modes());

                // 添加模式切换反馈
                if mode_changed {
                    // 同步菜单栏状态
                    self.menu_bar_state.current_mode = self.config.default_app_mode;

                    match self.config.default_app_mode {
                        AppMode::ImageConverter => {
                            self.status_message = "已切换到图片格式转换模式".to_string();
                        },
                        AppMode::ImageToPdf => {
                            self.status_message = "已切换到图片转PDF模式".to_string();
                        },
                        AppMode::PdfToImage => {
                            self.status_message = "已切换到PDF转图片模式".to_string();
                        },
                        AppMode::PureWatermark => {
                            self.status_message = "已切换到纯水印模式（原画质）".to_string();
                        },
                    }
                    self.is_error = false;
                }
            });

            ui.add_space(15.0);

            // --- 【借用修复】路径设置 ---
            // 将UI逻辑直接写在这里，避免了复杂的借用传递
            components::parameter_group(ui, "1. 选择路径", |ui| {
                ui.horizontal(|ui| {
                    ui.label("输入路径:");
                    ui.add(egui::TextEdit::singleline(&mut self.input_path).desired_width(ui.available_width() - 80.0));
                    if ui.button("📂 选择").clicked() {
                        let dialog = FileDialog::new();
                        let path = match self.config.default_processing_mode {
                            ProcessingMode::SingleFile => dialog.pick_file(),
                            ProcessingMode::Folder => dialog.pick_folder(),
                        };
                        if let Some(p) = path {
                            self.input_path = p.to_string_lossy().to_string();
                        }
                    }
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label("输出路径:");
                    ui.add(egui::TextEdit::singleline(&mut self.output_path).desired_width(ui.available_width() - 80.0));
                    if ui.button("📂 选择").clicked() {
                        if let Some(p) = FileDialog::new().pick_folder() {
                            self.output_path = p.to_string_lossy().to_string();
                        }
                    }
                });
            });

            ui.add_space(15.0);

            // 根据应用模式显示不同的设置界面
            match self.config.default_app_mode {
                AppMode::ImageConverter => {
                    self.show_image_converter_settings(ui);
                },
                AppMode::ImageToPdf => {
                    self.show_pdf_converter_settings(ui);
                },
                AppMode::PdfToImage => {
                    self.show_pdf_to_image_settings(ui);
                },
                AppMode::PureWatermark => {
                    self.show_pure_watermark_settings(ui);
                },
            }

            ui.add_space(15.0);

            // 根据应用模式显示不同的水印设置
            if self.config.default_app_mode == AppMode::ImageConverter {
                self.show_watermark_settings(ui);
            }

            ui.add_space(25.0);

            ui.horizontal(|ui| {
                let start_button = ui.add_enabled(!self.is_processing, egui::Button::new(styles::heading_text("🚀 开始转换")).min_size([150.0, 40.0].into()));
                if start_button.clicked() { self.start_processing(); }
                if components::secondary_button(ui, "📂 打开输出文件夹").clicked() {
                    if !self.output_path.is_empty() { let _ = file_utils::open_folder_in_explorer(Path::new(&self.output_path)); }
                }
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            if self.is_processing || self.progress.total > 0 {
                let progress_val = if self.progress.total > 0 { (self.progress.processed + self.progress.failed) as f32 / self.progress.total as f32 } else { 0.0 };
                components::progress_bar_with_text(ui, progress_val, &self.status_message);
                ui.add_space(10.0);
                components::statistics_display(ui, self.progress.processed, self.progress.failed, self.progress.total);
            } else {
                components::status_message_area(ui, &self.status_message, self.is_error);
            }
        });

        // 优化重绘策略，只在必要时重绘
        if self.is_processing {
            // 处理过程中每500ms重绘一次，而不是每帧重绘
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        } else if update_count > 0 {
            // 如果有进度更新，立即重绘一次
            ctx.request_repaint();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.config.default_input_path = self.input_path.clone();
        self.config.default_output_path = self.output_path.clone();
        if let Err(e) = self.config.save() { log::error!("退出时保存配置失败: {}", e); }
    }
}
