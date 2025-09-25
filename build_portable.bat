@echo off
chcp 65001 > nul
echo ================================
echo    图片格式转换工具便携版构建
echo ================================
echo.

:: 检查是否安装了Rust
echo [1/6] 检查Rust环境...
rustc --version > nul 2>&1
if errorlevel 1 (
    echo ❌ 错误: 未找到Rust编译器
    echo 请先安装Rust: https://rustup.rs/
    pause
    exit /b 1
)
echo ✅ Rust环境检查通过

:: 检查是否在正确的目录
echo [2/6] 检查项目目录...
if not exist "Cargo.toml" (
    echo ❌ 错误: 未找到Cargo.toml文件
    echo 请在项目根目录运行此脚本
    pause
    exit /b 1
)
echo ✅ 项目目录检查通过

:: 清理之前的构建
echo [3/6] 清理之前的构建...
if exist "target\release" (
    echo 清理旧的release构建...
    rmdir /s /q "target\release" 2>nul
)
if exist "dist" (
    echo 清理旧的dist目录...
    rmdir /s /q "dist" 2>nul
)
echo ✅ 清理完成

:: 构建Release版本
echo [4/6] 构建Release版本...
echo 正在编译，请稍候...
cargo build --release
if errorlevel 1 (
    echo ❌ 构建失败
    pause
    exit /b 1
)
echo ✅ Release构建完成

:: 创建分发目录
echo [5/6] 创建便携版目录...
mkdir dist 2>nul

:: 复制主程序
echo 复制主程序...
if exist "target\release\image_converter.exe" (
    copy "target\release\image_converter.exe" "dist\" > nul
    echo ✅ 主程序复制完成
) else (
    echo ❌ 错误: 找不到编译后的可执行文件
    pause
    exit /b 1
)

:: 复制资源文件
echo 复制资源文件...
if exist "resources\README.md" (
    copy "resources\README.md" "dist\使用说明.md" > nul
    echo ✅ 使用说明复制完成
)

:: 创建启动脚本
echo 创建启动脚本...
(
    echo @echo off
    echo chcp 65001 ^> nul
    echo echo 正在启动图片格式转换工具...
    echo start "" "image_converter.exe"
) > "dist\启动工具.bat"
echo ✅ 启动脚本创建完成

:: 创建配置示例
echo 创建配置示例...
(
    echo {
    echo   "default_input_path": "",
    echo   "default_output_path": "./output",
    echo   "default_target_size": 400,
    echo   "default_output_format": "Jpeg",
    echo   "default_processing_mode": "SingleFile",
    echo   "window_settings": {
    echo     "width": 800.0,
    echo     "height": 600.0,
    echo     "maximized": false,
    echo     "position": null
    echo   },
    echo   "advanced_settings": {
    echo     "jpeg_quality_range": [10, 95],
    echo     "png_compression_level": 6,
    echo     "pdf_render_dpi": 150.0,
    echo     "max_concurrent_jobs": 4,
    echo     "keep_original_files": true,
    echo     "show_detailed_progress": true,
    echo     "auto_open_output_folder": false
    echo   }
    echo }
) > "dist\config_example.json"
echo ✅ 配置示例创建完成

:: 获取文件信息
echo [6/6] 生成版本信息...
for %%F in ("dist\image_converter.exe") do set FileSize=%%~zF
set /a FileSizeMB=%FileSize%/1024/1024

:: 显示完成信息
echo.
echo ================================
echo        构建完成！
echo ================================
echo 📁 输出目录: %cd%\dist
echo 📦 程序大小: %FileSizeMB% MB
echo 📄 包含文件:
echo    - image_converter.exe (主程序)
echo    - 启动工具.bat (快速启动)
echo    - 使用说明.md (用户手册)
echo    - config_example.json (配置示例)
echo.
echo 💡 使用方法:
echo    1. 将dist文件夹复制到目标计算机
echo    2. 双击"启动工具.bat"运行程序
echo    3. 或直接运行"image_converter.exe"
echo.

:: 询问是否打开输出目录
set /p choice="是否现在打开输出目录? (y/n): "
if /i "%choice%"=="y" (
    explorer "dist"
)

:: 询问是否测试运行
set /p test="是否测试运行程序? (y/n): "
if /i "%test%"=="y" (
    echo 正在启动程序进行测试...
    cd dist
    start "" "image_converter.exe"
    cd ..
)

echo.
echo 构建脚本执行完成！
pause