// 文件路径: src/converter/batch_processor.rs

use crate::app::ProgressUpdate;
use crate::converter::{image_converter, pdf_converter};
use crate::utils::config::{OutputFormat, ProcessingMode, WatermarkSettings};
use crate::utils::file_utils::get_files_in_directory;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;
// use num_cpus; // <--- 注释掉未使用的导入

pub struct BatchProcessor;

impl BatchProcessor {
    pub async fn process_files(
        input_path: PathBuf,
        output_dir: PathBuf,
        target_size_kb: u32,
        output_format: OutputFormat,
        mode: ProcessingMode,
        watermark_settings: WatermarkSettings,
        progress_sender: mpsc::UnboundedSender<ProgressUpdate>,
    ) {
        tokio::task::spawn_blocking(move || {
            let result = Self::run_conversion(
                input_path,
                output_dir,
                target_size_kb,
                output_format,
                mode,
                watermark_settings,
                &progress_sender,
            );
            if let Err(e) = result {
                let _ = progress_sender.send(ProgressUpdate {
                    is_complete: true,
                    error_message: Some(format!("处理失败: {}", e)),
                    ..Default::default()
                });
            }
        })
        .await
        .ok();
    }

    fn run_conversion(
        input_path: PathBuf,
        output_dir: PathBuf,
        target_size_kb: u32,
        output_format: OutputFormat,
        mode: ProcessingMode,
        watermark_settings: WatermarkSettings,
        progress_sender: &mpsc::UnboundedSender<ProgressUpdate>,
    ) -> Result<()> {
        let files_to_process = match mode {
            ProcessingMode::SingleFile => vec![input_path],
            ProcessingMode::Folder => get_files_in_directory(&input_path)?,
        };

        if files_to_process.is_empty() {
            let _ = progress_sender.send(ProgressUpdate { is_complete: true, ..Default::default() });
            return Ok(());
        }

        // --- 1. 预扫描以获取准确的总任务数（即总输出图片数） ---
        let _ = progress_sender.send(ProgressUpdate {
            current_file: "正在计算总任务数...".to_string(),
            ..Default::default()
        });
        let total_tasks: usize = files_to_process.iter().map(|path| {
            if path.extension().map_or(false, |e| e == "pdf") {
                match pdf_converter::get_pdf_page_count(path) {
                    Ok(count) => {
                        println!("📄 PDF文件 {} 有 {} 页", path.display(), count);
                        count
                    },
                    Err(e) => {
                        eprintln!("⚠️  无法获取PDF页数 {}: {}，默认为1页", path.display(), e);
                        1
                    }
                }
            } else {
                1
            }
        }).sum();
        
        let _ = progress_sender.send(ProgressUpdate { total: total_tasks, ..Default::default() });

        if total_tasks == 0 {
            let _ = progress_sender.send(ProgressUpdate { is_complete: true, total: 0, ..Default::default() });
            return Ok(());
        }

        let processed_tasks = AtomicUsize::new(0);
        let failed_tasks = AtomicUsize::new(0);


        // --- 2. 根据输出格式调整并行策略 ---
        match output_format {
            OutputFormat::Jpeg => {
                // JPEG处理相对较快，可以使用全并行
                files_to_process.par_iter().for_each(|file_path| {
                    Self::process_single_file(
                        file_path,
                        &output_dir,
                        target_size_kb,
                        output_format,
                        &watermark_settings,
                        progress_sender,
                        total_tasks,
                        &processed_tasks,
                        &failed_tasks
                    );
                });
            },
            OutputFormat::PngCompressed => {
                // 优化后的PNG压缩可以使用更高的并行度
                Self::process_files_with_optimized_parallelism(
                    &files_to_process,
                    &output_dir,
                    target_size_kb,
                    output_format,
                    &watermark_settings,
                    progress_sender,
                    total_tasks,
                    &processed_tasks,
                    &failed_tasks,
                );
            },
            OutputFormat::PngOriginal => {
                // PNG原始处理相对较快，可以使用全并行
                files_to_process.par_iter().for_each(|file_path| {
                    Self::process_single_file(
                        file_path,
                        &output_dir,
                        target_size_kb,
                        output_format,
                        &watermark_settings,
                        progress_sender,
                        total_tasks,
                        &processed_tasks,
                        &failed_tasks
                    );
                });
            },
            OutputFormat::WebPLossy | OutputFormat::WebPLossless => {
                // WebP处理高效，可以使用全并行
                files_to_process.par_iter().for_each(|file_path| {
                    Self::process_single_file(
                        file_path,
                        &output_dir,
                        target_size_kb,
                        output_format,
                        &watermark_settings,
                        progress_sender,
                        total_tasks,
                        &processed_tasks,
                        &failed_tasks
                    );
                });
            }
        }

        // --- 3. 发送最终的完成信号 ---
        let _ = progress_sender.send(ProgressUpdate {
            processed: processed_tasks.load(Ordering::SeqCst),
            failed: failed_tasks.load(Ordering::SeqCst),
            total: total_tasks,
            is_complete: true,
            ..Default::default()
        });

        Ok(())
    }

    /// PNG优化：使用更激进的并行策略
    fn process_files_with_optimized_parallelism(
        files_to_process: &[PathBuf],
        output_dir: &Path,
        target_size_kb: u32,
        format: OutputFormat,
        watermark_settings: &WatermarkSettings,
        progress_sender: &mpsc::UnboundedSender<ProgressUpdate>,
        total_tasks: usize,
        processed_tasks: &AtomicUsize,
        failed_tasks: &AtomicUsize,
    ) {
        // 使用简单的串行处理，避免多线程竞态条件
        for file_path in files_to_process.iter() {
            Self::process_single_file(
                file_path,
                output_dir,
                target_size_kb,
                format,
                watermark_settings,
                progress_sender,
                total_tasks,
                processed_tasks,
                failed_tasks
            );
        }
    }

    /// 处理单个文件（PDF或图片）
    fn process_single_file(
        file_path: &Path,
        output_dir: &Path,
        target_size_kb: u32,
        format: OutputFormat,
        watermark_settings: &WatermarkSettings,
        progress_sender: &mpsc::UnboundedSender<ProgressUpdate>,
        total_tasks: usize,
        processed_tasks: &AtomicUsize,
        failed_tasks: &AtomicUsize,
    ) {
        let result = if file_path.extension().map_or(false, |e| e == "pdf") {
            Self::process_pdf(file_path, output_dir, target_size_kb, format, watermark_settings, progress_sender, total_tasks, processed_tasks, failed_tasks)
        } else {
            Self::process_image(file_path, output_dir, target_size_kb, format, watermark_settings, progress_sender, total_tasks, processed_tasks, failed_tasks)
        };
        
        if result.is_err() {
            let tasks_in_file = if file_path.extension().map_or(false, |e| e == "pdf") {
                pdf_converter::get_pdf_page_count(file_path).unwrap_or(1)
            } else { 1 };
            failed_tasks.fetch_add(tasks_in_file, Ordering::SeqCst);
        }
    }

    fn process_pdf(
        input_path: &Path,
        output_dir: &Path,
        target_size_kb: u32,
        format: OutputFormat,
        watermark_settings: &WatermarkSettings,
        progress_sender: &mpsc::UnboundedSender<ProgressUpdate>,
        total_tasks: usize,
        processed_tasks: &AtomicUsize,
        failed_tasks: &AtomicUsize,
    ) -> Result<()> {
        let images = pdf_converter::convert_pdf_to_images(input_path, 150.0)?;
        let pdf_stem = input_path.file_stem().unwrap().to_string_lossy();

        // PDF页面处理：使用安全的串行处理
        println!("🚀 开始处理 {} 页面", images.len());

        for (i, image) in images.iter().enumerate() {
            let output_filename = format!("{}_page_{}.{}", pdf_stem, i + 1, format.extension());
            let output_path = output_dir.join(output_filename);

            let result = if watermark_settings.enable_text_watermark || watermark_settings.enable_image_watermark {
                let text_watermark = if watermark_settings.enable_text_watermark {
                    Some(watermark_settings.to_text_watermark())
                } else {
                    None
                };

                let image_watermark = if watermark_settings.enable_image_watermark && !watermark_settings.image_watermark_path.is_empty() {
                    Some(watermark_settings.to_image_watermark())
                } else {
                    None
                };

                image_converter::compress_and_save_with_watermark(
                    image, &output_path, target_size_kb, format,
                    text_watermark.as_ref(), image_watermark.as_ref()
                )
            } else {
                image_converter::compress_and_save(image, &output_path, target_size_kb, format)
            };

            if result.is_ok() {
                processed_tasks.fetch_add(1, Ordering::SeqCst);

                // 每处理完10页或最后一页，输出进度
                if (i + 1) % 10 == 0 || i == images.len() - 1 {
                    println!("📄 已完成页面 {}/{}", i + 1, images.len());
                }
            } else {
                failed_tasks.fetch_add(1, Ordering::SeqCst);
                eprintln!("❌ 页面 {} 处理失败", i + 1);
            }

            // 发送进度更新（串行，安全）
            let _ = progress_sender.send(ProgressUpdate {
                processed: processed_tasks.load(Ordering::SeqCst),
                failed: failed_tasks.load(Ordering::SeqCst),
                total: total_tasks,
                current_file: format!("{} (第 {} 页)", input_path.to_string_lossy(), i + 1),
                ..Default::default()
            });
        }
        Ok(())
    }

    fn process_image(
        input_path: &Path,
        output_dir: &Path,
        target_size_kb: u32,
        format: OutputFormat,
        watermark_settings: &WatermarkSettings,
        progress_sender: &mpsc::UnboundedSender<ProgressUpdate>,
        total_tasks: usize,
        processed_tasks: &AtomicUsize,
        failed_tasks: &AtomicUsize,
    ) -> Result<()> {
        let image = image::open(input_path)?;
        let output_filename = input_path.file_name().unwrap();
        let output_path = output_dir.join(output_filename).with_extension(format.extension());

        // 检查是否需要添加水印
        if watermark_settings.enable_text_watermark || watermark_settings.enable_image_watermark {
            let text_watermark = if watermark_settings.enable_text_watermark {
                Some(watermark_settings.to_text_watermark())
            } else {
                None
            };

            let image_watermark = if watermark_settings.enable_image_watermark && !watermark_settings.image_watermark_path.is_empty() {
                Some(watermark_settings.to_image_watermark())
            } else {
                None
            };

            image_converter::compress_and_save_with_watermark(
                &image, &output_path, target_size_kb, format,
                text_watermark.as_ref(), image_watermark.as_ref()
            )?;
        } else {
            image_converter::compress_and_save(&image, &output_path, target_size_kb, format)?;
        }
        processed_tasks.fetch_add(1, Ordering::SeqCst);

        let _ = progress_sender.send(ProgressUpdate {
            processed: processed_tasks.load(Ordering::SeqCst),
            failed: failed_tasks.load(Ordering::SeqCst),
            total: total_tasks,
            current_file: input_path.to_string_lossy().to_string(),
            ..Default::default()
        });
        Ok(())
    }
}