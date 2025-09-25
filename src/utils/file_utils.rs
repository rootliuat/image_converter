use anyhow::Result; // --- 【清理】移除了未使用的 `Context` ---
use std::path::{Path, PathBuf};
// 已移除 sysinfo::Disks 导入 - 未使用
use walkdir::WalkDir;

/// 获取指定目录下的所有文件路径
pub fn get_files_in_directory(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        anyhow::bail!("提供的路径不是一个目录: {}", dir.display());
    }
    let files = WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();
    Ok(files)
}

/// 在文件浏览器中打开指定文件夹
pub fn open_folder_in_explorer(path: &Path) -> Result<()> {
    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("路径包含无效的UTF-8字符"))?;

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer").arg(path_str).spawn()?;
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        use std::process::Command;
        let cmd = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
        Command::new(cmd).arg(path_str).spawn()?;
    }

    Ok(())
}

// 已移除未使用的 check_disk_space 函数以消除编译警告