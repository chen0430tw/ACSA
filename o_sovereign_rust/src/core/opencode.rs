// OpenCode Executor Module
// DeepSeek的"执行之手" (Execution Hands)
//
// OpenCode = 超级工头 (Super Foreman)
// ACSA = 影子政府 (Shadow Government)
// DeepSeek = 大脑 (Brain)

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};

/// OpenCode执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub files_created: Vec<String>,
    pub files_modified: Vec<String>,
}

/// OpenCode执行器配置
#[derive(Debug, Clone)]
pub struct OpenCodeConfig {
    /// 工作目录
    pub workspace: PathBuf,
    /// 是否使用真实OpenCode CLI（false则使用模拟模式）
    pub use_real_cli: bool,
    /// OpenCode CLI路径
    pub cli_path: Option<PathBuf>,
}

impl Default for OpenCodeConfig {
    fn default() -> Self {
        Self {
            workspace: PathBuf::from("/tmp/opencode_workspace"),
            use_real_cli: false,
            cli_path: None,
        }
    }
}

/// OpenCode执行器
///
/// 两种工作模式：
/// 1. **CLI模式**：调用真实的OpenCode CLI（需要安装）
/// 2. **文件模式**：直接操作文件系统（默认，更可靠）
pub struct OpenCodeExecutor {
    config: OpenCodeConfig,
}

impl OpenCodeExecutor {
    /// 创建新的OpenCode执行器
    pub fn new(config: OpenCodeConfig) -> Self {
        Self { config }
    }

    /// 执行代码生成任务
    ///
    /// # Arguments
    /// * `task` - DeepSeek生成的任务描述
    /// * `code` - 要执行/写入的代码
    /// * `language` - 编程语言（用于文件扩展名）
    pub async fn execute_task(
        &self,
        task: &str,
        code: &str,
        language: &str,
    ) -> Result<OpenCodeResult> {
        info!("🔧 OpenCode Executor: {}", task);

        if self.config.use_real_cli {
            self.execute_via_cli(task, code, language).await
        } else {
            self.execute_via_filesystem(task, code, language).await
        }
    }

    /// 通过CLI执行（需要安装OpenCode）
    async fn execute_via_cli(
        &self,
        task: &str,
        code: &str,
        language: &str,
    ) -> Result<OpenCodeResult> {
        let cli_path = self
            .config
            .cli_path
            .as_ref()
            .ok_or_else(|| anyhow!("OpenCode CLI path not configured"))?;

        info!("  Using OpenCode CLI: {:?}", cli_path);

        // 创建临时输入文件
        let input_file = self.config.workspace.join("task.txt");
        fs::create_dir_all(&self.config.workspace).await?;
        fs::write(&input_file, format!("{}\n\n```{}\n{}\n```", task, language, code)).await?;

        // 调用OpenCode CLI
        let output = Command::new(cli_path)
            .arg("execute")
            .arg("--input")
            .arg(&input_file)
            .arg("--workspace")
            .arg(&self.config.workspace)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| anyhow!("Failed to execute OpenCode CLI: {}", e))?;

        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(OpenCodeResult {
            success,
            output: stdout.clone(),
            error: if stderr.is_empty() {
                None
            } else {
                Some(stderr)
            },
            files_created: self.extract_created_files(&stdout),
            files_modified: Vec::new(),
        })
    }

    /// 通过文件系统直接执行（默认模式，更可靠）
    async fn execute_via_filesystem(
        &self,
        task: &str,
        code: &str,
        language: &str,
    ) -> Result<OpenCodeResult> {
        info!("  Using filesystem mode (direct write)");

        // 确保工作目录存在
        fs::create_dir_all(&self.config.workspace).await?;

        // 生成文件名
        let ext = match language {
            "rust" => "rs",
            "python" => "py",
            "javascript" | "js" => "js",
            "typescript" | "ts" => "ts",
            "go" => "go",
            "java" => "java",
            "cpp" | "c++" => "cpp",
            "c" => "c",
            _ => "txt",
        };

        let filename = format!(
            "generated_{}.{}",
            chrono::Utc::now().format("%Y%m%d_%H%M%S"),
            ext
        );
        let filepath = self.config.workspace.join(&filename);

        // 写入代码
        let mut file = tokio::fs::File::create(&filepath).await?;
        file.write_all(code.as_bytes()).await?;
        file.flush().await?;

        info!("  ✓ Code written to: {}", filepath.display());

        // 写入任务描述（元数据）
        let meta_filename = format!("{}.meta.txt", filename);
        let meta_filepath = self.config.workspace.join(&meta_filename);
        fs::write(&meta_filepath, task).await?;

        Ok(OpenCodeResult {
            success: true,
            output: format!(
                "Successfully wrote code to: {}\nTask: {}",
                filepath.display(),
                task
            ),
            error: None,
            files_created: vec![filename.clone()],
            files_modified: Vec::new(),
        })
    }

    /// 修改已有文件
    pub async fn modify_file(
        &self,
        filepath: &Path,
        new_content: &str,
    ) -> Result<OpenCodeResult> {
        info!("✏️  Modifying file: {}", filepath.display());

        // 检查文件是否存在
        if !filepath.exists() {
            return Err(anyhow!("File not found: {}", filepath.display()));
        }

        // 备份原文件
        let backup_path = filepath.with_extension("backup");
        fs::copy(filepath, &backup_path).await?;
        debug!("  Backup created: {}", backup_path.display());

        // 写入新内容
        fs::write(filepath, new_content).await?;

        info!("  ✓ File modified successfully");

        Ok(OpenCodeResult {
            success: true,
            output: format!("Modified: {}", filepath.display()),
            error: None,
            files_created: Vec::new(),
            files_modified: vec![filepath.display().to_string()],
        })
    }

    /// 创建新项目结构
    pub async fn create_project(
        &self,
        project_name: &str,
        language: &str,
    ) -> Result<OpenCodeResult> {
        info!("🏗️  Creating new project: {} ({})", project_name, language);

        let project_path = self.config.workspace.join(project_name);
        fs::create_dir_all(&project_path).await?;

        let mut files_created = Vec::new();

        match language {
            "rust" => {
                // 创建基本Rust项目结构
                let cargo_toml = project_path.join("Cargo.toml");
                fs::write(
                    &cargo_toml,
                    format!(
                        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
                        project_name
                    ),
                )
                .await?;
                files_created.push("Cargo.toml".to_string());

                let src_dir = project_path.join("src");
                fs::create_dir_all(&src_dir).await?;

                let main_rs = src_dir.join("main.rs");
                fs::write(
                    &main_rs,
                    r#"fn main() {
    println!("Hello from O-Sovereign!");
}
"#,
                )
                .await?;
                files_created.push("src/main.rs".to_string());
            }
            "python" => {
                // 创建Python项目结构
                let main_py = project_path.join("main.py");
                fs::write(&main_py, "# O-Sovereign Python Project\n\nprint('Hello from O-Sovereign!')\n").await?;
                files_created.push("main.py".to_string());

                let requirements = project_path.join("requirements.txt");
                fs::write(&requirements, "# Dependencies\n").await?;
                files_created.push("requirements.txt".to_string());
            }
            _ => {
                warn!("Unknown language: {}, creating basic structure", language);
                let readme = project_path.join("README.md");
                fs::write(&readme, format!("# {}\n\nGenerated by O-Sovereign ACSA", project_name)).await?;
                files_created.push("README.md".to_string());
            }
        }

        info!("  ✓ Project created with {} files", files_created.len());

        Ok(OpenCodeResult {
            success: true,
            output: format!("Created project: {}", project_path.display()),
            error: None,
            files_created,
            files_modified: Vec::new(),
        })
    }

    /// 列出工作目录中的文件
    pub async fn list_files(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();

        if !self.config.workspace.exists() {
            return Ok(files);
        }

        let mut entries = fs::read_dir(&self.config.workspace).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Ok(file_type) = entry.file_type().await {
                if file_type.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        files.push(name.to_string());
                    }
                }
            }
        }

        Ok(files)
    }

    /// 读取文件内容
    pub async fn read_file(&self, filename: &str) -> Result<String> {
        let filepath = self.config.workspace.join(filename);
        let content = fs::read_to_string(&filepath).await?;
        Ok(content)
    }

    /// 从CLI输出中提取创建的文件列表
    fn extract_created_files(&self, output: &str) -> Vec<String> {
        // 简单的启发式提取
        // 真实的OpenCode CLI可能会有结构化输出
        output
            .lines()
            .filter_map(|line| {
                if line.contains("Created:") || line.contains("created") {
                    line.split_whitespace().last().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// 获取工作目录
    pub fn workspace(&self) -> &Path {
        &self.config.workspace
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_filesystem_execution() {
        let config = OpenCodeConfig {
            workspace: PathBuf::from("/tmp/opencode_test"),
            use_real_cli: false,
            cli_path: None,
        };

        let executor = OpenCodeExecutor::new(config);

        let result = executor
            .execute_task(
                "Test code generation",
                "fn main() { println!(\"test\"); }",
                "rust",
            )
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.files_created.len(), 1);
    }

    #[tokio::test]
    async fn test_project_creation() {
        let config = OpenCodeConfig {
            workspace: PathBuf::from("/tmp/opencode_test_project"),
            use_real_cli: false,
            cli_path: None,
        };

        let executor = OpenCodeExecutor::new(config);

        let result = executor
            .create_project("test_project", "rust")
            .await
            .unwrap();

        assert!(result.success);
        assert!(result.files_created.contains(&"Cargo.toml".to_string()));
        assert!(result.files_created.contains(&"src/main.rs".to_string()));
    }

    #[tokio::test]
    async fn test_list_files() {
        let config = OpenCodeConfig {
            workspace: PathBuf::from("/tmp/opencode_test_list"),
            use_real_cli: false,
            cli_path: None,
        };

        let executor = OpenCodeExecutor::new(config);

        // Create test file
        executor
            .execute_task("test", "print('hello')", "python")
            .await
            .unwrap();

        let files = executor.list_files().await.unwrap();
        assert!(!files.is_empty());
    }
}
