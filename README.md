# 🖼️ 图片转换工具 (Image Converter)

一个用 Rust 开发的专业图片格式转换工具，具有现代化 GUI 界面，支持批量处理和多种格式转换。

![GitHub](https://img.shields.io/github/license/rootliuat/image_converter)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)

## ✨ 主要特性

### 🔄 格式转换
- **图片格式转换**: JPEG、PNG、BMP、TIFF、WebP、GIF
- **压缩优化**: 智能压缩算法，支持目标大小控制
- **质量保持**: 原始质量模式，无损压缩选项

### 📄 PDF处理
- **图片转PDF**: 保持原始尺寸和质量，自动页面适配
- **PDF转图片**: 高质量渲染，支持批量处理
- **页面设置**: 自动/横向/纵向页面方向

### 💧 水印功能
- **文字水印**: 自定义文字、颜色、透明度、字符间距
- **图片水印**: 支持PNG透明水印，可调节缩放比例
- **位置控制**: 9个预设位置 + 自定义像素定位
- **背景支持**: 半透明背景，增强可读性

### 🎨 用户界面
- **现代化GUI**: 基于egui的直观界面设计
- **菜单栏系统**: 完整的文件/功能/工具/帮助菜单
- **实时预览**: 处理进度和结果实时显示
- **多模式切换**: 图片转换/图片转PDF/PDF转图片/纯水印模式

### ⚡ 高性能
- **并发处理**: 多线程批量处理，充分利用CPU资源
- **内存优化**: 智能内存管理，避免大文件内存溢出
- **异步处理**: 非阻塞UI，后台处理任务

## 📥 安装和运行

### 系统要求
- **操作系统**: Windows 10/11, macOS 10.15+, Linux (Ubuntu 18.04+)
- **内存**: 建议 4GB 以上
- **磁盘空间**: 100MB 可用空间

### 从源码编译

1. **安装 Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **克隆项目**
   ```bash
   git clone https://github.com/rootliuat/image_converter.git
   cd image_converter
   ```

3. **编译运行**
   ```bash
   cargo run --release
   ```

### 预编译版本
访问 [Releases](https://github.com/rootliuat/image_converter/releases) 下载对应平台的预编译版本。

## 🚀 使用指南

### 基本操作

1. **启动应用程序**
   ```bash
   cargo run
   # 或直接运行编译后的可执行文件
   ./target/release/image_converter
   ```

2. **选择功能模式**
   - 菜单栏 → 功能 → 选择对应模式
   - 或使用界面右上角的模式切换按钮

3. **设置参数**
   - 输入路径：选择要处理的文件或文件夹
   - 输出路径：指定处理结果保存位置
   - 格式设置：选择输出格式和质量参数

4. **开始处理**
   - 点击"🚀 开始转换"按钮
   - 实时查看处理进度和状态

### 功能详解

#### 📸 图片格式转换
```
支持格式: JPEG ↔ PNG ↔ BMP ↔ TIFF ↔ WebP ↔ GIF
压缩模式: 目标大小控制 / 原始质量保持
批量处理: 整个文件夹一键转换
```

#### 📄 图片转PDF
```
输入: 单张图片或图片文件夹
输出: 高质量PDF文件
特性: 保持原始尺寸，自动页面适配
速度: 1500x844图片 0.05秒/张
```

#### 🖼️ PDF转图片
```
输入: 单个或多个PDF文件
输出: 高质量JPEG/PNG图片
渲染: 150DPI高清晰度
批量: 支持大文件(100+页)处理
```

#### 💧 水印添加
```
文字水印: 自定义内容、字体、颜色、位置
图片水印: 支持透明PNG，可调节缩放
位置选择: 九宫格预设 + 精确像素定位
效果控制: 透明度、背景、字符间距
```

### 配置文件

应用支持JSON配置文件 `config.json`：

```json
{
  "default_input_path": "",
  "default_output_path": "./output",
  "default_target_size": 400,
  "default_output_format": "PngCompressed",
  "default_compression_mode": "SizeTarget",
  "default_processing_mode": "SingleFile",
  "default_app_mode": "ImageConverter",
  "window_settings": {
    "width": 800.0,
    "height": 600.0,
    "maximized": false,
    "position": null
  },
  "watermark_settings": {
    "enable_text_watermark": false,
    "text_content": "Copyright",
    "text_size": 24,
    "text_color": [255, 255, 255, 200],
    "text_opacity": 0.8,
    "text_position": "BottomRight",
    "text_margin": 20,
    "text_letter_spacing": 2.0
  }
}
```

## 🧪 测试验证

### 功能测试
运行内置测试来验证功能：

```bash
# 水印功能测试
cargo run --bin test_watermark

# 水印位置测试
cargo run --bin test_watermark_positions

# PDF转换测试
cargo run --bin test_pdf_conversion
```

### 性能基准

根据最新测试结果：

| 功能 | 测试样本 | 处理时间 | 成功率 |
|------|----------|----------|--------|
| PDF→图片 | 4个PDF文件(197页) | 批量处理 | 100% |
| 图片→PDF | 2张1500x844 | 0.10秒 | 100% |
| 单图→PDF | 1张1500x844 | 0.05秒 | 100% |
| 水印处理 | 9个预设位置 | < 1秒 | 100% |

## 🛠️ 开发

### 项目结构
```
src/
├── main.rs              # 应用入口
├── app.rs               # 主应用逻辑
├── lib.rs               # 库入口
├── converter/           # 转换器模块
│   ├── mod.rs
│   ├── image_converter.rs
│   ├── image_to_pdf.rs
│   ├── pdf_to_image.rs
│   ├── batch_processor.rs
│   └── simple_watermark.rs
├── ui/                  # 用户界面
│   ├── mod.rs
│   ├── components.rs
│   ├── styles.rs
│   └── menu_bar.rs
├── utils/               # 工具模块
│   ├── mod.rs
│   └── config.rs
└── bin/                 # 测试程序
    ├── test_watermark.rs
    ├── test_watermark_positions.rs
    └── test_pdf_conversion.rs
```

### 核心依赖

```toml
[dependencies]
egui = "0.28"               # GUI框架
eframe = "0.28"             # 应用框架
image = "0.25"              # 图像处理
pdfium-render = "0.8"       # PDF渲染
printpdf = "0.7"            # PDF生成
tokio = "1.0"               # 异步运行时
rayon = "1.10"              # 并行处理
serde = "1.0"               # 序列化
anyhow = "1.0"              # 错误处理
```

### 编译选项

```toml
# 发布版本优化
[profile.release]
lto = true
codegen-units = 1
panic = "abort"

# 开发版本设置
[profile.dev]
debug = true
opt-level = 0
```

## 🐛 故障排除

### 常见问题

1. **编译失败**
   ```bash
   # 更新Rust工具链
   rustup update

   # 清理构建缓存
   cargo clean
   ```

2. **PDF处理错误**
   - 确保系统安装了必要的字体
   - 检查PDF文件是否损坏
   - 尝试降低渲染DPI设置

3. **内存不足**
   - 减少并发处理数量
   - 使用分批处理大文件
   - 增加系统虚拟内存

4. **权限问题**
   - 检查输出目录写入权限
   - 以管理员身份运行（Windows）
   - 使用 `sudo` 运行（Linux/macOS）

## 🤝 贡献

欢迎贡献代码！请遵循以下流程：

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

### 开发规范

- 使用 `rustfmt` 格式化代码
- 运行 `cargo clippy` 检查代码质量
- 添加适当的测试用例
- 更新相关文档

## 📄 许可证

本项目基于 MIT 许可证发布。详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- [egui](https://github.com/emilk/egui) - 优秀的即时模式GUI库
- [image](https://github.com/image-rs/image) - Rust图像处理库
- [pdfium-render](https://github.com/ajrcarey/pdfium-render) - PDF渲染支持
- [printpdf](https://github.com/fschutt/printpdf) - PDF生成库

## 📞 联系方式

- **GitHub**: [rootliuat](https://github.com/rootliuat)
- **Issues**: [问题反馈](https://github.com/rootliuat/image_converter/issues)
- **Discussions**: [讨论区](https://github.com/rootliuat/image_converter/discussions)

## 🎯 路线图

### 即将发布 (v1.1)
- [ ] 压缩算法优化 (减少60-80%编码次数)
- [ ] SIMD向量化加速 (20-30%性能提升)
- [ ] 内存池机制 (50%内存占用减少)
- [ ] 智能参数预估模型

### 计划中 (v1.2)
- [ ] 图片批处理预览
- [ ] 自定义输出文件名模板
- [ ] 更多水印效果和滤镜
- [ ] 插件系统支持

### 长期目标
- [ ] 云端处理集成
- [ ] AI智能优化
- [ ] 移动端支持
- [ ] Web版本开发

---

<div align="center">

**⭐ 如果这个项目对您有帮助，请给个星标支持！**

Made with ❤️ by [rootliuat](https://github.com/rootliuat)

</div>