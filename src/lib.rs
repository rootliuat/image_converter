// 库文件 - 公开内部模块供测试使用

pub mod app;
pub mod converter;
pub mod utils;
pub mod ui;

// Re-export common types for external use
pub use utils::config::OutputFormat;
pub use converter::image_converter::compress_and_save;