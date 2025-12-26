// Multimodal Support Module
// 多模态支持 - 图片、文件、PDF等

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tracing::{debug, info, warn};

/// 多模态输入类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModalityType {
    /// 文本
    Text,
    /// 图片 (JPEG, PNG, GIF, WebP)
    Image { mime_type: String },
    /// 文件 (代码、文档等)
    File { extension: String },
    /// PDF文档
    Pdf,
    /// 音频 (暂不支持)
    Audio,
    /// 视频 (暂不支持)
    Video,
}

/// 多模态输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalInput {
    /// 输入类型
    pub modality: ModalityType,
    /// 内容 (文本或Base64编码)
    pub content: String,
    /// 元数据
    pub metadata: MultimodalMetadata,
}

/// 多模态元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalMetadata {
    /// 原始文件路径
    pub file_path: Option<String>,
    /// 文件大小 (bytes)
    pub size: u64,
    /// 是否已Base64编码
    pub is_base64: bool,
    /// 附加信息
    pub extra: std::collections::HashMap<String, String>,
}

/// 多模态处理器
pub struct MultimodalProcessor {
    /// 最大文件大小 (默认 10MB)
    max_file_size: u64,
    /// 支持的图片格式
    supported_image_formats: Vec<String>,
    /// 支持的文本文件格式
    supported_text_formats: Vec<String>,
}

impl Default for MultimodalProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl MultimodalProcessor {
    pub fn new() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            supported_image_formats: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "webp".to_string(),
                "bmp".to_string(),
            ],
            supported_text_formats: vec![
                "txt".to_string(),
                "md".to_string(),
                "rs".to_string(),
                "py".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "go".to_string(),
                "java".to_string(),
                "c".to_string(),
                "cpp".to_string(),
                "h".to_string(),
                "hpp".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "toml".to_string(),
                "xml".to_string(),
                "html".to_string(),
                "css".to_string(),
                "sh".to_string(),
                "bash".to_string(),
                "sql".to_string(),
            ],
        }
    }

    /// 处理文件输入
    pub async fn process_file(&self, file_path: &Path) -> Result<MultimodalInput> {
        info!("📂 Processing file: {}", file_path.display());

        // 检查文件是否存在
        if !file_path.exists() {
            return Err(anyhow!("File not found: {}", file_path.display()));
        }

        // 获取文件元数据
        let metadata = fs::metadata(file_path).await?;
        let size = metadata.len();

        // 检查文件大小
        if size > self.max_file_size {
            return Err(anyhow!(
                "File too large: {} bytes (max: {} bytes)",
                size,
                self.max_file_size
            ));
        }

        // 获取文件扩展名
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        debug!("  Extension: {}, Size: {} bytes", extension, size);

        // 判断文件类型并处理
        if self.is_image(&extension) {
            self.process_image(file_path, &extension, size).await
        } else if extension == "pdf" {
            self.process_pdf(file_path, size).await
        } else if self.is_text_file(&extension) {
            self.process_text_file(file_path, &extension, size).await
        } else {
            // 默认作为二进制文件处理
            self.process_binary_file(file_path, &extension, size)
                .await
        }
    }

    /// 处理图片文件
    async fn process_image(
        &self,
        file_path: &Path,
        extension: &str,
        size: u64,
    ) -> Result<MultimodalInput> {
        info!("🖼️  Processing image: {}", extension);

        // 读取图片并转为Base64
        let bytes = fs::read(file_path).await?;
        let base64_content = general_purpose::STANDARD.encode(&bytes);

        // 确定MIME类型
        let mime_type = match extension {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "bmp" => "image/bmp",
            _ => "image/unknown",
        }
        .to_string();

        info!("  ✓ Encoded to Base64 ({} bytes -> {} chars)", size, base64_content.len());

        Ok(MultimodalInput {
            modality: ModalityType::Image { mime_type },
            content: base64_content,
            metadata: MultimodalMetadata {
                file_path: Some(file_path.display().to_string()),
                size,
                is_base64: true,
                extra: std::collections::HashMap::new(),
            },
        })
    }

    /// 处理文本文件
    async fn process_text_file(
        &self,
        file_path: &Path,
        extension: &str,
        size: u64,
    ) -> Result<MultimodalInput> {
        info!("📄 Processing text file: {}", extension);

        // 读取文本内容
        let content = fs::read_to_string(file_path).await?;
        let line_count = content.lines().count();

        info!(
            "  ✓ Read {} lines ({} bytes)",
            line_count, size
        );

        let mut extra = std::collections::HashMap::new();
        extra.insert("line_count".to_string(), line_count.to_string());
        extra.insert("language".to_string(), extension.to_string());

        Ok(MultimodalInput {
            modality: ModalityType::File {
                extension: extension.to_string(),
            },
            content,
            metadata: MultimodalMetadata {
                file_path: Some(file_path.display().to_string()),
                size,
                is_base64: false,
                extra,
            },
        })
    }

    /// 处理PDF文件
    async fn process_pdf(&self, file_path: &Path, size: u64) -> Result<MultimodalInput> {
        info!("📕 Processing PDF file");

        // 读取PDF并转为Base64
        let bytes = fs::read(file_path).await?;
        let base64_content = general_purpose::STANDARD.encode(&bytes);

        info!("  ✓ Encoded PDF to Base64");

        Ok(MultimodalInput {
            modality: ModalityType::Pdf,
            content: base64_content,
            metadata: MultimodalMetadata {
                file_path: Some(file_path.display().to_string()),
                size,
                is_base64: true,
                extra: std::collections::HashMap::new(),
            },
        })
    }

    /// 处理二进制文件
    async fn process_binary_file(
        &self,
        file_path: &Path,
        extension: &str,
        size: u64,
    ) -> Result<MultimodalInput> {
        warn!("⚠️  Processing as binary file: {}", extension);

        // 读取并转为Base64
        let bytes = fs::read(file_path).await?;
        let base64_content = general_purpose::STANDARD.encode(&bytes);

        Ok(MultimodalInput {
            modality: ModalityType::File {
                extension: extension.to_string(),
            },
            content: base64_content,
            metadata: MultimodalMetadata {
                file_path: Some(file_path.display().to_string()),
                size,
                is_base64: true,
                extra: std::collections::HashMap::new(),
            },
        })
    }

    /// 处理文本输入
    pub fn process_text(&self, text: &str) -> MultimodalInput {
        MultimodalInput {
            modality: ModalityType::Text,
            content: text.to_string(),
            metadata: MultimodalMetadata {
                file_path: None,
                size: text.len() as u64,
                is_base64: false,
                extra: std::collections::HashMap::new(),
            },
        }
    }

    /// 判断是否为图片文件
    fn is_image(&self, extension: &str) -> bool {
        self.supported_image_formats.contains(&extension.to_string())
    }

    /// 判断是否为文本文件
    fn is_text_file(&self, extension: &str) -> bool {
        self.supported_text_formats.contains(&extension.to_string())
    }

    /// 格式化多模态输入为AI Prompt
    pub fn format_for_ai(&self, inputs: &[MultimodalInput]) -> String {
        let mut prompt = String::new();

        for (i, input) in inputs.iter().enumerate() {
            match &input.modality {
                ModalityType::Text => {
                    prompt.push_str(&format!("\n### Input {}: Text\n", i + 1));
                    prompt.push_str(&input.content);
                    prompt.push('\n');
                }
                ModalityType::Image { mime_type } => {
                    prompt.push_str(&format!("\n### Input {}: Image ({})\n", i + 1, mime_type));
                    prompt.push_str("[Image encoded as Base64]\n");
                    if let Some(path) = &input.metadata.file_path {
                        prompt.push_str(&format!("Source: {}\n", path));
                    }
                    prompt.push_str(&format!("Size: {} bytes\n", input.metadata.size));
                    // Note: 实际发送给支持vision的模型时需要特殊格式
                }
                ModalityType::File { extension } => {
                    prompt.push_str(&format!(
                        "\n### Input {}: File (.{})\n",
                        i + 1, extension
                    ));
                    if let Some(path) = &input.metadata.file_path {
                        prompt.push_str(&format!("File: {}\n", path));
                    }
                    prompt.push_str("```");
                    prompt.push_str(extension);
                    prompt.push('\n');
                    prompt.push_str(&input.content);
                    prompt.push_str("\n```\n");
                }
                ModalityType::Pdf => {
                    prompt.push_str(&format!("\n### Input {}: PDF Document\n", i + 1));
                    prompt.push_str("[PDF encoded as Base64]\n");
                    if let Some(path) = &input.metadata.file_path {
                        prompt.push_str(&format!("Source: {}\n", path));
                    }
                }
                ModalityType::Audio | ModalityType::Video => {
                    prompt.push_str(&format!("\n### Input {}: Unsupported modality\n", i + 1));
                }
            }
        }

        prompt
    }

    /// 设置最大文件大小
    pub fn set_max_file_size(&mut self, size: u64) {
        self.max_file_size = size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_text_processing() {
        let processor = MultimodalProcessor::new();
        let input = processor.process_text("Hello, world!");

        assert!(matches!(input.modality, ModalityType::Text));
        assert_eq!(input.content, "Hello, world!");
        assert!(!input.metadata.is_base64);
    }

    #[tokio::test]
    async fn test_format_for_ai() {
        let processor = MultimodalProcessor::new();
        let inputs = vec![
            processor.process_text("Analyze this code:"),
            MultimodalInput {
                modality: ModalityType::File {
                    extension: "rs".to_string(),
                },
                content: "fn main() {}".to_string(),
                metadata: MultimodalMetadata {
                    file_path: Some("main.rs".to_string()),
                    size: 12,
                    is_base64: false,
                    extra: std::collections::HashMap::new(),
                },
            },
        ];

        let formatted = processor.format_for_ai(&inputs);
        assert!(formatted.contains("Text"));
        assert!(formatted.contains("File (.rs)"));
        assert!(formatted.contains("```rs"));
    }
}
