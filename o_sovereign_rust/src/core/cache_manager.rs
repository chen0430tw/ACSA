// Cache Manager - 避免系统卡死的清理系统
// Prevents system freezing by managing cache and log cleanup

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// 缓存清理策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPolicy {
    /// 最大缓存大小 (MB)
    pub max_cache_size_mb: u64,
    /// 最大日志大小 (MB)
    pub max_log_size_mb: u64,
    /// 保留天数
    pub retention_days: u32,
    /// 自动清理触发阈值 (0.0-1.0, 达到max的比例时触发)
    pub auto_cleanup_threshold: f64,
    /// 是否启用自动清理
    pub enable_auto_cleanup: bool,
}

impl Default for CleanupPolicy {
    fn default() -> Self {
        Self {
            max_cache_size_mb: 1024,      // 1GB cache
            max_log_size_mb: 512,          // 512MB logs
            retention_days: 7,             // 保留7天
            auto_cleanup_threshold: 0.8,   // 80%时触发
            enable_auto_cleanup: true,
        }
    }
}

/// 清理统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStats {
    /// 清理的文件数量
    pub files_removed: u32,
    /// 释放的空间 (bytes)
    pub space_freed_bytes: u64,
    /// 清理耗时 (ms)
    pub elapsed_ms: u64,
    /// 清理时间
    pub cleaned_at: DateTime<Utc>,
}

impl CleanupStats {
    pub fn space_freed_mb(&self) -> f64 {
        self.space_freed_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// 缓存类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheType {
    /// API响应缓存
    ApiResponse,
    /// 模型输出缓存
    ModelOutput,
    /// 临时文件
    TempFiles,
    /// 日志文件
    Logs,
}

impl CacheType {
    pub fn subdir(&self) -> &'static str {
        match self {
            CacheType::ApiResponse => "api_cache",
            CacheType::ModelOutput => "model_cache",
            CacheType::TempFiles => "temp",
            CacheType::Logs => "logs",
        }
    }
}

/// 缓存管理器
pub struct CacheManager {
    /// 缓存根目录
    cache_root: PathBuf,
    /// 清理策略
    policy: CleanupPolicy,
    /// 上次清理时间
    last_cleanup: Option<DateTime<Utc>>,
}

impl CacheManager {
    /// 创建缓存管理器
    pub fn new(cache_root: PathBuf, policy: CleanupPolicy) -> Result<Self> {
        // 确保缓存目录存在
        fs::create_dir_all(&cache_root)
            .context("Failed to create cache root directory")?;

        // 创建子目录
        for cache_type in &[
            CacheType::ApiResponse,
            CacheType::ModelOutput,
            CacheType::TempFiles,
            CacheType::Logs,
        ] {
            let subdir = cache_root.join(cache_type.subdir());
            fs::create_dir_all(&subdir)
                .context(format!("Failed to create {:?} directory", cache_type))?;
        }

        info!("🗄️ CacheManager initialized at {:?}", cache_root);
        info!("  Max cache: {} MB", policy.max_cache_size_mb);
        info!("  Max logs: {} MB", policy.max_log_size_mb);
        info!("  Retention: {} days", policy.retention_days);

        Ok(Self {
            cache_root,
            policy,
            last_cleanup: None,
        })
    }

    /// 使用默认策略创建
    pub fn with_defaults(cache_root: PathBuf) -> Result<Self> {
        Self::new(cache_root, CleanupPolicy::default())
    }

    /// 获取缓存类型的目录路径
    pub fn get_cache_dir(&self, cache_type: CacheType) -> PathBuf {
        self.cache_root.join(cache_type.subdir())
    }

    /// 计算目录大小 (bytes)
    fn calculate_dir_size(&self, path: &Path) -> Result<u64> {
        let mut total_size = 0u64;

        if !path.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += self.calculate_dir_size(&entry.path())?;
            }
        }

        Ok(total_size)
    }

    /// 获取缓存使用情况
    pub fn get_cache_usage(&self) -> Result<CacheUsage> {
        let api_cache_bytes = self.calculate_dir_size(&self.get_cache_dir(CacheType::ApiResponse))?;
        let model_cache_bytes = self.calculate_dir_size(&self.get_cache_dir(CacheType::ModelOutput))?;
        let temp_bytes = self.calculate_dir_size(&self.get_cache_dir(CacheType::TempFiles))?;
        let log_bytes = self.calculate_dir_size(&self.get_cache_dir(CacheType::Logs))?;

        let total_cache_bytes = api_cache_bytes + model_cache_bytes + temp_bytes;
        let total_bytes = total_cache_bytes + log_bytes;

        Ok(CacheUsage {
            total_bytes,
            cache_bytes: total_cache_bytes,
            log_bytes,
            api_cache_bytes,
            model_cache_bytes,
            temp_bytes,
        })
    }

    /// 检查是否需要清理
    pub fn needs_cleanup(&self) -> Result<bool> {
        if !self.policy.enable_auto_cleanup {
            return Ok(false);
        }

        let usage = self.get_cache_usage()?;

        let cache_threshold = (self.policy.max_cache_size_mb * 1024 * 1024) as f64 * self.policy.auto_cleanup_threshold;
        let log_threshold = (self.policy.max_log_size_mb * 1024 * 1024) as f64 * self.policy.auto_cleanup_threshold;

        let needs_cache_cleanup = usage.cache_bytes as f64 >= cache_threshold;
        let needs_log_cleanup = usage.log_bytes as f64 >= log_threshold;

        Ok(needs_cache_cleanup || needs_log_cleanup)
    }

    /// 清理过期文件
    pub fn cleanup_expired(&mut self) -> Result<CleanupStats> {
        info!("🧹 Starting cleanup of expired files...");
        let start = std::time::Instant::now();

        let cutoff_time = Utc::now() - Duration::days(self.policy.retention_days as i64);
        let mut files_removed = 0u32;
        let mut space_freed = 0u64;

        // 清理所有缓存类型
        for cache_type in &[
            CacheType::ApiResponse,
            CacheType::ModelOutput,
            CacheType::TempFiles,
            CacheType::Logs,
        ] {
            let dir = self.get_cache_dir(*cache_type);
            let (removed, freed) = self.cleanup_old_files(&dir, cutoff_time)?;
            files_removed += removed;
            space_freed += freed;

            if removed > 0 {
                info!("  {:?}: removed {} files, freed {:.2} MB",
                    cache_type, removed, freed as f64 / (1024.0 * 1024.0));
            }
        }

        let elapsed_ms = start.elapsed().as_millis() as u64;
        self.last_cleanup = Some(Utc::now());

        let stats = CleanupStats {
            files_removed,
            space_freed_bytes: space_freed,
            elapsed_ms,
            cleaned_at: Utc::now(),
        };

        info!("✅ Cleanup completed: {} files, {:.2} MB freed in {} ms",
            files_removed, stats.space_freed_mb(), elapsed_ms);

        Ok(stats)
    }

    /// 清理特定目录下的旧文件
    fn cleanup_old_files(&self, dir: &Path, cutoff_time: DateTime<Utc>) -> Result<(u32, u64)> {
        let mut files_removed = 0u32;
        let mut space_freed = 0u64;

        if !dir.exists() {
            return Ok((0, 0));
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let metadata = entry.metadata()?;

                if let Ok(modified) = metadata.modified() {
                    let modified_time: DateTime<Utc> = modified.into();

                    if modified_time < cutoff_time {
                        let file_size = metadata.len();

                        match fs::remove_file(&path) {
                            Ok(_) => {
                                files_removed += 1;
                                space_freed += file_size;
                                debug!("Removed old file: {:?}", path);
                            }
                            Err(e) => {
                                warn!("Failed to remove {:?}: {}", path, e);
                            }
                        }
                    }
                }
            } else if path.is_dir() {
                // 递归清理子目录
                let (sub_removed, sub_freed) = self.cleanup_old_files(&path, cutoff_time)?;
                files_removed += sub_removed;
                space_freed += sub_freed;

                // 如果目录为空，删除它
                if fs::read_dir(&path)?.count() == 0 {
                    if let Err(e) = fs::remove_dir(&path) {
                        debug!("Failed to remove empty dir {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok((files_removed, space_freed))
    }

    /// 强制清理到目标大小
    pub fn cleanup_to_size(&mut self, target_mb: u64) -> Result<CleanupStats> {
        info!("🧹 Force cleanup to target size: {} MB", target_mb);
        let start = std::time::Instant::now();

        let target_bytes = target_mb * 1024 * 1024;
        let usage = self.get_cache_usage()?;

        if usage.total_bytes <= target_bytes {
            info!("✅ Already below target size, no cleanup needed");
            return Ok(CleanupStats {
                files_removed: 0,
                space_freed_bytes: 0,
                elapsed_ms: start.elapsed().as_millis() as u64,
                cleaned_at: Utc::now(),
            });
        }

        let mut files_removed = 0u32;
        let mut space_freed = 0u64;
        let space_to_free = usage.total_bytes - target_bytes;

        // 优先清理临时文件和旧缓存
        let cleanup_order = vec![
            CacheType::TempFiles,
            CacheType::ApiResponse,
            CacheType::ModelOutput,
            CacheType::Logs,
        ];

        for cache_type in cleanup_order {
            if space_freed >= space_to_free {
                break;
            }

            let dir = self.get_cache_dir(cache_type);
            let (removed, freed) = self.cleanup_oldest_files(&dir, space_to_free - space_freed)?;

            files_removed += removed;
            space_freed += freed;

            if removed > 0 {
                info!("  {:?}: removed {} files, freed {:.2} MB",
                    cache_type, removed, freed as f64 / (1024.0 * 1024.0));
            }
        }

        let elapsed_ms = start.elapsed().as_millis() as u64;
        self.last_cleanup = Some(Utc::now());

        let stats = CleanupStats {
            files_removed,
            space_freed_bytes: space_freed,
            elapsed_ms,
            cleaned_at: Utc::now(),
        };

        info!("✅ Force cleanup completed: {} files, {:.2} MB freed",
            files_removed, stats.space_freed_mb());

        Ok(stats)
    }

    /// 清理最旧的文件直到释放足够空间
    fn cleanup_oldest_files(&self, dir: &Path, space_needed: u64) -> Result<(u32, u64)> {
        if !dir.exists() {
            return Ok((0, 0));
        }

        // 收集所有文件及其修改时间
        let mut files = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        files.push((path, modified, metadata.len()));
                    }
                }
            }
        }

        // 按修改时间排序（最旧的在前）
        files.sort_by_key(|(_, modified, _)| *modified);

        let mut files_removed = 0u32;
        let mut space_freed = 0u64;

        for (path, _, size) in files {
            if space_freed >= space_needed {
                break;
            }

            match fs::remove_file(&path) {
                Ok(_) => {
                    files_removed += 1;
                    space_freed += size;
                    debug!("Removed old file: {:?}", path);
                }
                Err(e) => {
                    warn!("Failed to remove {:?}: {}", path, e);
                }
            }
        }

        Ok((files_removed, space_freed))
    }

    /// 清空特定类型的缓存
    pub fn clear_cache_type(&mut self, cache_type: CacheType) -> Result<CleanupStats> {
        info!("🧹 Clearing {:?} cache...", cache_type);
        let start = std::time::Instant::now();

        let dir = self.get_cache_dir(cache_type);
        let (files_removed, space_freed) = self.remove_all_files(&dir)?;

        let stats = CleanupStats {
            files_removed,
            space_freed_bytes: space_freed,
            elapsed_ms: start.elapsed().as_millis() as u64,
            cleaned_at: Utc::now(),
        };

        info!("✅ {:?} cache cleared: {} files, {:.2} MB",
            cache_type, files_removed, stats.space_freed_mb());

        Ok(stats)
    }

    /// 删除目录下所有文件
    fn remove_all_files(&self, dir: &Path) -> Result<(u32, u64)> {
        if !dir.exists() {
            return Ok((0, 0));
        }

        let mut files_removed = 0u32;
        let mut space_freed = 0u64;

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let size = entry.metadata()?.len();

                match fs::remove_file(&path) {
                    Ok(_) => {
                        files_removed += 1;
                        space_freed += size;
                    }
                    Err(e) => {
                        warn!("Failed to remove {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok((files_removed, space_freed))
    }

    /// 自动清理检查（如果启用且超过阈值）
    pub fn auto_cleanup_check(&mut self) -> Result<Option<CleanupStats>> {
        if !self.needs_cleanup()? {
            return Ok(None);
        }

        info!("⚠️ Auto cleanup triggered");
        let stats = self.cleanup_expired()?;

        // 如果清理后仍然超标，强制清理到目标大小
        if self.needs_cleanup()? {
            warn!("⚠️ Still over threshold after cleanup, forcing size reduction");
            let target_cache_mb = (self.policy.max_cache_size_mb as f64 * 0.7) as u64;
            let force_stats = self.cleanup_to_size(target_cache_mb)?;

            Ok(Some(CleanupStats {
                files_removed: stats.files_removed + force_stats.files_removed,
                space_freed_bytes: stats.space_freed_bytes + force_stats.space_freed_bytes,
                elapsed_ms: stats.elapsed_ms + force_stats.elapsed_ms,
                cleaned_at: Utc::now(),
            }))
        } else {
            Ok(Some(stats))
        }
    }

    /// 获取清理策略
    pub fn get_policy(&self) -> &CleanupPolicy {
        &self.policy
    }

    /// 更新清理策略
    pub fn update_policy(&mut self, policy: CleanupPolicy) {
        info!("📝 Updating cleanup policy");
        self.policy = policy;
    }

    /// 打印缓存使用情况
    pub fn print_usage(&self) -> Result<()> {
        let usage = self.get_cache_usage()?;

        println!("\n┌─────────────────────────────────────────┐");
        println!("│        Cache Usage Report               │");
        println!("├─────────────────────────────────────────┤");
        println!("│  Total:        {:.2} MB", usage.total_mb());
        println!("│  Cache:        {:.2} MB", usage.cache_mb());
        println!("│    - API:      {:.2} MB", usage.api_cache_mb());
        println!("│    - Model:    {:.2} MB", usage.model_cache_mb());
        println!("│    - Temp:     {:.2} MB", usage.temp_mb());
        println!("│  Logs:         {:.2} MB", usage.log_mb());
        println!("├─────────────────────────────────────────┤");
        println!("│  Limits:                                │");
        println!("│    Cache:      {} MB", self.policy.max_cache_size_mb);
        println!("│    Logs:       {} MB", self.policy.max_log_size_mb);
        println!("│  Retention:    {} days", self.policy.retention_days);
        println!("└─────────────────────────────────────────┘");

        if let Some(last_cleanup) = self.last_cleanup {
            println!("\nLast cleanup: {}", last_cleanup.format("%Y-%m-%d %H:%M:%S"));
        }

        Ok(())
    }
}

/// 缓存使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheUsage {
    pub total_bytes: u64,
    pub cache_bytes: u64,
    pub log_bytes: u64,
    pub api_cache_bytes: u64,
    pub model_cache_bytes: u64,
    pub temp_bytes: u64,
}

impl CacheUsage {
    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn cache_mb(&self) -> f64 {
        self.cache_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn log_mb(&self) -> f64 {
        self.log_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn api_cache_mb(&self) -> f64 {
        self.api_cache_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn model_cache_mb(&self) -> f64 {
        self.model_cache_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn temp_mb(&self) -> f64 {
        self.temp_bytes as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_cache_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CacheManager::with_defaults(temp_dir.path().to_path_buf());

        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(manager.get_cache_dir(CacheType::ApiResponse).exists());
        assert!(manager.get_cache_dir(CacheType::Logs).exists());
    }

    #[test]
    fn test_cache_usage_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CacheManager::with_defaults(temp_dir.path().to_path_buf()).unwrap();

        // 创建测试文件
        let test_file = manager.get_cache_dir(CacheType::ApiResponse).join("test.txt");
        let mut file = File::create(test_file).unwrap();
        file.write_all(b"Hello, world!").unwrap();

        let usage = manager.get_cache_usage().unwrap();
        assert!(usage.api_cache_bytes > 0);
    }

    #[test]
    fn test_cleanup_expired() {
        let temp_dir = TempDir::new().unwrap();
        let policy = CleanupPolicy {
            retention_days: 0, // 立即过期
            ..Default::default()
        };

        let mut manager = CacheManager::new(temp_dir.path().to_path_buf(), policy).unwrap();

        // 创建测试文件
        let test_file = manager.get_cache_dir(CacheType::TempFiles).join("old.txt");
        File::create(test_file).unwrap();

        // 等待1秒确保文件被标记为旧
        std::thread::sleep(std::time::Duration::from_secs(1));

        let stats = manager.cleanup_expired().unwrap();
        assert!(stats.files_removed > 0);
    }
}
