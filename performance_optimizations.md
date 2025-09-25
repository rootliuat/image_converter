# 图片转换工具性能优化方案

## 🎯 核心优化策略

### 1. 智能预估算法替代试错法

**现状问题**: 当前使用二分法+多次编码来找到合适的压缩参数，效率低下

**优化方案**: 使用数学模型预估最佳参数

```rust
// 新增: 基于图片特征的快速预估
struct ImageMetrics {
    complexity_score: f32,    // 复杂度评分 (0.0-1.0)
    color_count: u32,         // 颜色数量
    has_transparency: bool,   // 是否有透明通道
    size_factor: f32,         // 尺寸因子
}

fn estimate_optimal_params(image: &DynamicImage, target_bytes: usize) -> (u8, f32) {
    let metrics = analyze_image_quickly(image);

    // JPEG质量预估公式 (基于经验数据拟合)
    let base_quality = match metrics.complexity_score {
        x if x > 0.8 => 85,  // 高复杂度图片
        x if x > 0.5 => 70,  // 中复杂度
        _ => 55,             // 低复杂度
    };

    // 根据目标大小调整
    let size_ratio = target_bytes as f32 / estimate_uncompressed_size(image);
    let quality = (base_quality as f32 * size_ratio.sqrt()).clamp(10.0, 95.0) as u8;

    // 缩放比例预估
    let scale = if size_ratio < 0.3 { 0.7 } else { 1.0 };

    (quality, scale)
}
```

### 2. 流水线并行处理架构

**现状问题**: 各种格式处理策略不统一，资源利用不充分

**优化方案**: 实现三级流水线

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct OptimizedBatchProcessor {
    // 限制并发的图片加载数量
    load_semaphore: Arc<Semaphore>,
    // 限制并发的处理数量
    process_semaphore: Arc<Semaphore>,
    // 限制并发的保存数量
    save_semaphore: Arc<Semaphore>,
}

impl OptimizedBatchProcessor {
    pub fn new() -> Self {
        let cpu_count = num_cpus::get();
        Self {
            load_semaphore: Arc::new(Semaphore::new(cpu_count * 2)),
            process_semaphore: Arc::new(Semaphore::new(cpu_count)),
            save_semaphore: Arc::new(Semaphore::new(cpu_count / 2)),
        }
    }

    async fn process_file_pipeline(&self, file_path: PathBuf) -> Result<()> {
        // 阶段1: 加载 (IO密集)
        let image = {
            let _permit = self.load_semaphore.acquire().await?;
            tokio::task::spawn_blocking(move || {
                image::open(&file_path)
            }).await??
        };

        // 阶段2: 处理 (CPU密集)
        let processed_data = {
            let _permit = self.process_semaphore.acquire().await?;
            tokio::task::spawn_blocking(move || {
                // 快速预估 + 单次压缩
                let (quality, scale) = estimate_optimal_params(&image, target_bytes);
                compress_with_params(&image, quality, scale)
            }).await??
        };

        // 阶段3: 保存 (IO密集)
        {
            let _permit = self.save_semaphore.acquire().await?;
            tokio::fs::write(output_path, processed_data).await?;
        }

        Ok(())
    }
}
```

### 3. 内存优化策略

**问题**: 大批量处理时内存使用过多

**方案**: 实现内存池和分块处理

```rust
use once_cell::sync::Lazy;
use std::sync::Mutex;

// 全局内存池，复用缓冲区
static BUFFER_POOL: Lazy<Mutex<Vec<Vec<u8>>>> = Lazy::new(|| {
    Mutex::new(Vec::new())
});

fn get_reusable_buffer(min_size: usize) -> Vec<u8> {
    let mut pool = BUFFER_POOL.lock().unwrap();
    pool.pop()
        .filter(|buf| buf.capacity() >= min_size)
        .unwrap_or_else(|| Vec::with_capacity(min_size))
}

fn return_buffer(mut buffer: Vec<u8>) {
    // 只保留合理大小的缓冲区
    if buffer.capacity() <= 50 * 1024 * 1024 { // 50MB以下
        buffer.clear();
        let mut pool = BUFFER_POOL.lock().unwrap();
        if pool.len() < 10 { // 最多保留10个
            pool.push(buffer);
        }
    }
}

// 大图片分块处理
fn process_large_image_in_chunks(image: &DynamicImage, chunk_size: u32) -> Result<Vec<u8>> {
    if image.width() <= chunk_size && image.height() <= chunk_size {
        return compress_single_pass(image); // 小图片直接处理
    }

    // 大图片分块处理并重新组合
    // 这里可以实现图片分割、并行处理、重组的逻辑
    todo!("实现分块处理逻辑")
}
```

### 4. SIMD向量化优化

**问题**: 未充分利用现代CPU的向量指令

**方案**: 在关键路径使用SIMD

```rust
// 在 Cargo.toml 中添加
// image = { version = "0.25", features = ["jpeg", "png", "webp", "bmp", "tiff", "simd"] }

// 启用SIMD优化的图片缩放
fn resize_with_simd(image: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    // 使用带SIMD优化的缩放算法
    image.resize_exact(width, height, image::imageops::FilterType::CatmullRom)
}

// 快速图片复杂度分析
fn analyze_image_quickly(image: &DynamicImage) -> ImageMetrics {
    let sample_pixels = sample_pixels_simd(image, 1000); // 采样1000个像素

    ImageMetrics {
        complexity_score: calculate_entropy_simd(&sample_pixels),
        color_count: estimate_color_count_simd(&sample_pixels),
        has_transparency: image.color().has_alpha(),
        size_factor: (image.width() * image.height()) as f32 / 1_000_000.0,
    }
}
```

### 5. 缓存和预处理优化

**问题**: 重复处理相似操作

**方案**: 智能缓存机制

```rust
use std::collections::HashMap;
use std::sync::RwLock;

static COMPRESSION_CACHE: Lazy<RwLock<HashMap<String, (u8, f32)>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

fn get_cached_params(file_path: &Path, target_size: usize) -> Option<(u8, f32)> {
    let key = format!("{}:{}", file_path.to_string_lossy(), target_size);
    COMPRESSION_CACHE.read().unwrap().get(&key).copied()
}

fn cache_params(file_path: &Path, target_size: usize, quality: u8, scale: f32) {
    let key = format!("{}:{}", file_path.to_string_lossy(), target_size);
    let mut cache = COMPRESSION_CACHE.write().unwrap();

    // LRU清理：保持缓存大小在1000以内
    if cache.len() >= 1000 {
        cache.clear(); // 简单清理策略
    }

    cache.insert(key, (quality, scale));
}
```

### 6. 配置文件优化

**新增高性能配置项**:

```toml
[performance]
# 内存限制 (MB)
max_memory_usage = 2048

# 并发策略
max_concurrent_files = 8
io_thread_count = 4
cpu_thread_count = 0  # 0 = auto detect

# 压缩优化
use_fast_preview = true       # 使用快速预览算法
enable_simd = true            # 启用SIMD优化
cache_compression_params = true # 缓存压缩参数

# 图片处理
chunk_size_mb = 100           # 大图片分块大小
quality_estimation = "fast"   # fast/accurate
```

## 📊 预期性能提升

1. **压缩速度**: 减少60-80%的编码次数，预计提速3-5倍
2. **内存使用**: 通过内存池和分块处理，减少50%内存占用
3. **并发效率**: 流水线架构预计提高40%的整体吞吐量
4. **大文件处理**: SIMD优化预计提速20-30%
5. **响应性**: 更好的资源管理，避免UI冻结

## 🛠 实施优先级

1. **高优先级** (立即实施):
   - 智能参数预估算法
   - 内存池优化

2. **中优先级** (1-2周内):
   - 流水线并行架构
   - SIMD优化

3. **低优先级** (长期):
   - 高级缓存策略
   - 分块处理大图片

## 🧪 性能测试建议

```bash
# 创建性能测试脚本
# 测试场景1: 100张中等大小图片 (2-5MB each)
# 测试场景2: 10张大图片 (20-50MB each)
# 测试场景3: 1000张小图片 (100KB-1MB each)
# 测试场景4: 混合PDF + 图片批处理

# 性能指标:
# - 总处理时间
# - 内存峰值使用量
# - CPU使用率
# - 磁盘IO效率
```