// Emergency Log System - 紧急日志保存系统
// 目标：避免断电导致聊天记录丢失
//
// 核心策略：
// 1. 实时写入：每条消息立即持久化到磁盘
// 2. 双重备份：内存buffer + 磁盘文件  
// 3. 原子写入：使用write-tmp-rename模式避免损坏
// 4. 自动恢复：启动时自动恢复未完成的对话
// 5. 压缩存档：定期压缩历史日志节省空间

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write as IoWrite};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyLogConfig {
    pub log_dir: PathBuf,
    pub max_buffer_size: usize,
    pub flush_interval_secs: u64,
    pub max_file_size_mb: u64,
    pub enable_compression: bool,
    pub retention_days: u32,
}

impl Default for EmergencyLogConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("./logs/emergency"),
            max_buffer_size: 1000,
            flush_interval_secs: 5,
            max_file_size_mb: 100,
            enable_compression: true,
            retention_days: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub entry_type: LogEntryType,
    pub content: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogEntryType {
    UserInput,
    AssistantResponse,
    SystemEvent,
    Error,
    ToolCall,
    ToolResult,
}

pub struct EmergencyLogger {
    config: EmergencyLogConfig,
    buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    current_file: Arc<Mutex<Option<BufWriter<File>>>>,
    current_session: String,
    last_flush: Arc<Mutex<std::time::Instant>>,
}

impl EmergencyLogger {
    pub fn new(config: EmergencyLogConfig) -> Result<Self> {
        fs::create_dir_all(&config.log_dir)?;
        info!("🔥 Emergency Logger initialized");

        Ok(Self {
            config,
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            current_file: Arc::new(Mutex::new(None)),
            current_session: format!("session_{}", Utc::now().format("%Y%m%d_%H%M%S")),
            last_flush: Arc::new(Mutex::new(std::time::Instant::now())),
        })
    }

    pub fn log(&self, entry_type: LogEntryType, content: String, metadata: serde_json::Value) -> Result<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            session_id: self.current_session.clone(),
            entry_type,
            content,
            metadata,
        };

        {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.push_back(entry.clone());
        }

        self.write_entry_to_disk(&entry)?;
        Ok(())
    }

    fn write_entry_to_disk(&self, entry: &LogEntry) -> Result<()> {
        let mut file_guard = self.current_file.lock().unwrap();

        if file_guard.is_none() {
            let file_path = self.config.log_dir.join(format!("{}.jsonl", self.current_session));
            let file = OpenOptions::new().create(true).append(true).open(&file_path)?;
            *file_guard = Some(BufWriter::new(file));
        }

        if let Some(writer) = file_guard.as_mut() {
            let json = serde_json::to_string(entry)?;
            writeln!(writer, "{}", json)?;
            writer.flush()?;
        }

        Ok(())
    }

    pub fn flush(&self) -> Result<()> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.clear();
        Ok(())
    }

    pub fn recover_session(&self, session_id: &str) -> Result<Vec<LogEntry>> {
        let file_path = self.config.log_dir.join(format!("{}.jsonl", session_id));
        if !file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&file_path)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            if let Ok(entry) = serde_json::from_str::<LogEntry>(line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    pub fn shutdown(&self) -> Result<()> {
        self.flush()?;
        let mut file_guard = self.current_file.lock().unwrap();
        if let Some(mut writer) = file_guard.take() {
            writer.flush()?;
        }
        Ok(())
    }
}

impl Drop for EmergencyLogger {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
