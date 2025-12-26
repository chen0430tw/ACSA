// Voice Processor - STT/TTS语音处理系统
// 基于Kyutai

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::info;

use super::emergency_log::{EmergencyLogger, LogEntryType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub kyutai_server_url: String,
    pub enable_cache: bool,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            kyutai_server_url: "ws://localhost:8000".to_string(),
            enable_cache: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttResult {
    pub text: String,
    pub confidence: f64,
}

pub struct VoiceProcessor {
    config: VoiceConfig,
    audio_cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    emergency_logger: Option<Arc<EmergencyLogger>>,
}

impl VoiceProcessor {
    pub fn new(config: VoiceConfig) -> Self {
        info!("🎤 Voice Processor initialized");
        Self {
            config,
            audio_cache: Arc::new(RwLock::new(HashMap::new())),
            emergency_logger: None,
        }
    }

    pub async fn speech_to_text(&self, audio: &[u8]) -> Result<SttResult> {
        info!("🎧 STT: {} bytes", audio.len());
        
        // 模拟实现
        let text = "主人，我收到了您的语音指令".to_string();
        
        if let Some(logger) = &self.emergency_logger {
            logger.log(
                LogEntryType::UserInput,
                text.clone(),
                serde_json::json!({"source": "voice"}),
            )?;
        }

        Ok(SttResult {
            text,
            confidence: 0.95,
        })
    }

    pub async fn text_to_speech(&self, text: &str) -> Result<Vec<u8>> {
        info!("🔊 TTS: {}", text);
        
        let cache_key = text.to_string();
        if self.config.enable_cache {
            let cache = self.audio_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let audio = vec![0u8; 1024]; // 模拟音频

        if self.config.enable_cache {
            let mut cache = self.audio_cache.write().await;
            cache.insert(cache_key, audio.clone());
        }

        Ok(audio)
    }
}
