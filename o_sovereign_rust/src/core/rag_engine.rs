// RAG Engine - 检索增强生成系统
// Retrieval-Augmented Generation for AI responses
//
// 核心功能：
// 1. 向量数据库集成（Qdrant/Milvus）
// 2. 文档分块和嵌入
// 3. 语义检索
// 4. 上下文注入
// 5. 混合检索（向量+关键词）

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// 文档分块策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkingStrategy {
    /// 固定大小分块
    FixedSize,
    /// 语义分块（按段落/句子）
    Semantic,
    /// 滑动窗口
    SlidingWindow,
    /// 递归分块
    Recursive,
}

/// 嵌入模型类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmbeddingModel {
    /// OpenAI text-embedding-3-small
    OpenAISmall,
    /// OpenAI text-embedding-3-large
    OpenAILarge,
    /// 本地模型（all-MiniLM-L6-v2）
    LocalMiniLM,
    /// 自定义模型
    Custom,
}

/// 检索模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrievalMode {
    /// 纯向量检索
    VectorOnly,
    /// 纯关键词检索
    KeywordOnly,
    /// 混合检索
    Hybrid,
}

/// 文档块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    /// 块ID
    pub chunk_id: String,
    /// 原始文档ID
    pub document_id: String,
    /// 块内容
    pub content: String,
    /// 块索引（在文档中的位置）
    pub chunk_index: usize,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 嵌入向量（base64编码）
    pub embedding: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// 文档ID
    pub document_id: String,
    /// 文档标题
    pub title: String,
    /// 文档内容
    pub content: String,
    /// 文档类型（markdown/pdf/txt/code）
    pub doc_type: String,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 检索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    /// 文档块
    pub chunk: DocumentChunk,
    /// 相似度分数（0.0-1.0）
    pub score: f64,
    /// 检索方法
    pub retrieval_method: String,
}

/// RAG配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    /// 嵌入模型
    pub embedding_model: EmbeddingModel,
    /// 分块策略
    pub chunking_strategy: ChunkingStrategy,
    /// 块大小（字符数）
    pub chunk_size: usize,
    /// 块重叠（字符数）
    pub chunk_overlap: usize,
    /// 检索模式
    pub retrieval_mode: RetrievalMode,
    /// Top-K结果数
    pub top_k: usize,
    /// 最小相似度阈值
    pub min_similarity: f64,
    /// 向量数据库URL
    pub vector_db_url: String,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            embedding_model: EmbeddingModel::OpenAISmall,
            chunking_strategy: ChunkingStrategy::Semantic,
            chunk_size: 512,
            chunk_overlap: 50,
            retrieval_mode: RetrievalMode::Hybrid,
            top_k: 5,
            min_similarity: 0.7,
            vector_db_url: "http://localhost:6333".to_string(), // Qdrant默认端口
        }
    }
}

/// RAG统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RagStats {
    pub total_documents: u64,
    pub total_chunks: u64,
    pub total_queries: u64,
    pub avg_query_time_ms: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// RAG引擎
pub struct RagEngine {
    config: RagConfig,
    /// 文档存储
    documents: Arc<RwLock<HashMap<String, Document>>>,
    /// 块存储
    chunks: Arc<RwLock<HashMap<String, DocumentChunk>>>,
    /// 查询缓存
    query_cache: Arc<RwLock<HashMap<String, Vec<RetrievalResult>>>>,
    /// 统计信息
    stats: Arc<RwLock<RagStats>>,
}

impl RagEngine {
    /// 创建新的RAG引擎
    pub fn new(config: RagConfig) -> Self {
        info!("🔍 Initializing RAG Engine");
        info!("    Embedding Model: {:?}", config.embedding_model);
        info!("    Chunking: {:?} (size: {}, overlap: {})",
            config.chunking_strategy, config.chunk_size, config.chunk_overlap);
        info!("    Retrieval Mode: {:?}", config.retrieval_mode);
        info!("    Top-K: {}", config.top_k);

        Self {
            config,
            documents: Arc::new(RwLock::new(HashMap::new())),
            chunks: Arc::new(RwLock::new(HashMap::new())),
            query_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(RagStats::default())),
        }
    }

    /// 索引文档
    pub async fn index_document(&self, document: Document) -> Result<Vec<String>> {
        info!("📄 Indexing document: {}", document.title);

        // 分块
        let chunks = self.chunk_document(&document).await?;
        let chunk_ids: Vec<String> = chunks.iter().map(|c| c.chunk_id.clone()).collect();

        // 生成嵌入
        for mut chunk in chunks {
            let embedding = self.generate_embedding(&chunk.content).await?;
            chunk.embedding = Some(embedding);

            // 存储块
            let mut chunks_store = self.chunks.write().await;
            chunks_store.insert(chunk.chunk_id.clone(), chunk);
        }

        // 存储文档
        let mut docs = self.documents.write().await;
        docs.insert(document.document_id.clone(), document);

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.total_documents += 1;
        stats.total_chunks += chunk_ids.len() as u64;

        info!("✅ Indexed {} chunks", chunk_ids.len());
        Ok(chunk_ids)
    }

    /// 检索相关文档
    pub async fn retrieve(&self, query: &str) -> Result<Vec<RetrievalResult>> {
        let start = std::time::Instant::now();

        // 检查缓存
        let cache_key = self.compute_cache_key(query);
        {
            let cache = self.query_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                let mut stats = self.stats.write().await;
                stats.cache_hits += 1;
                info!("💨 Cache hit for query");
                return Ok(cached.clone());
            }
        }

        // 生成查询嵌入
        let query_embedding = self.generate_embedding(query).await?;

        // 执行检索
        let results = match self.config.retrieval_mode {
            RetrievalMode::VectorOnly => {
                self.vector_search(&query_embedding).await?
            }
            RetrievalMode::KeywordOnly => {
                self.keyword_search(query).await?
            }
            RetrievalMode::Hybrid => {
                self.hybrid_search(query, &query_embedding).await?
            }
        };

        // 过滤低分结果
        let filtered: Vec<RetrievalResult> = results
            .into_iter()
            .filter(|r| r.score >= self.config.min_similarity)
            .take(self.config.top_k)
            .collect();

        // 缓存结果
        {
            let mut cache = self.query_cache.write().await;
            cache.insert(cache_key, filtered.clone());
        }

        // 更新统计
        let elapsed = start.elapsed().as_millis() as f64;
        let mut stats = self.stats.write().await;
        stats.total_queries += 1;
        stats.cache_misses += 1;
        stats.avg_query_time_ms =
            (stats.avg_query_time_ms * (stats.total_queries - 1) as f64 + elapsed)
            / stats.total_queries as f64;

        info!("🔍 Retrieved {} results in {:.2}ms", filtered.len(), elapsed);
        Ok(filtered)
    }

    /// 构建增强上下文
    pub async fn build_augmented_context(
        &self,
        query: &str,
        system_prompt: &str,
    ) -> Result<String> {
        let results = self.retrieve(query).await?;

        if results.is_empty() {
            return Ok(system_prompt.to_string());
        }

        let mut context = String::from(system_prompt);
        context.push_str("\n\n## 参考上下文\n\n");

        for (i, result) in results.iter().enumerate() {
            context.push_str(&format!(
                "### 来源 {} (相似度: {:.2}%)\n{}\n\n",
                i + 1,
                result.score * 100.0,
                result.chunk.content
            ));
        }

        context.push_str("请基于以上上下文回答问题，如果上下文不相关，请说明并给出你的最佳回答。");

        Ok(context)
    }

    /// 删除文档
    pub async fn delete_document(&self, document_id: &str) -> Result<()> {
        // 删除文档
        let mut docs = self.documents.write().await;
        docs.remove(document_id);

        // 删除相关块
        let mut chunks = self.chunks.write().await;
        chunks.retain(|_, chunk| chunk.document_id != document_id);

        // 清除缓存
        let mut cache = self.query_cache.write().await;
        cache.clear();

        info!("🗑️  Deleted document: {}", document_id);
        Ok(())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> RagStats {
        self.stats.read().await.clone()
    }

    // ===== 内部方法 =====

    /// 文档分块
    async fn chunk_document(&self, document: &Document) -> Result<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();

        match self.config.chunking_strategy {
            ChunkingStrategy::FixedSize => {
                let content = &document.content;
                let chunk_size = self.config.chunk_size;
                let overlap = self.config.chunk_overlap;

                let mut start = 0;
                let mut chunk_index = 0;

                while start < content.len() {
                    let end = (start + chunk_size).min(content.len());
                    let chunk_content = content[start..end].to_string();

                    chunks.push(DocumentChunk {
                        chunk_id: format!("{}_{}", document.document_id, chunk_index),
                        document_id: document.document_id.clone(),
                        content: chunk_content,
                        chunk_index,
                        metadata: document.metadata.clone(),
                        embedding: None,
                        created_at: Utc::now(),
                    });

                    start += chunk_size - overlap;
                    chunk_index += 1;
                }
            }

            ChunkingStrategy::Semantic => {
                // TODO: 实现语义分块（按段落/句子）
                // 简化实现：按段落分块
                let paragraphs: Vec<&str> = document.content.split("\n\n").collect();
                for (i, para) in paragraphs.iter().enumerate() {
                    if para.trim().is_empty() {
                        continue;
                    }

                    chunks.push(DocumentChunk {
                        chunk_id: format!("{}_{}", document.document_id, i),
                        document_id: document.document_id.clone(),
                        content: para.to_string(),
                        chunk_index: i,
                        metadata: document.metadata.clone(),
                        embedding: None,
                        created_at: Utc::now(),
                    });
                }
            }

            _ => {
                // 其他策略待实现
                warn!("⚠️  Chunking strategy {:?} not implemented, using FixedSize", self.config.chunking_strategy);
            }
        }

        Ok(chunks)
    }

    /// 生成嵌入向量
    async fn generate_embedding(&self, text: &str) -> Result<String> {
        // TODO: 实际调用嵌入API
        // 根据 embedding_model 调用对应API：
        // - OpenAI: openai.embeddings.create()
        // - Local: 使用本地模型（如 rust-bert）

        // Placeholder: 返回模拟嵌入（base64编码的随机向量）
        let placeholder = format!("embedding_{}", text.len());
        Ok(placeholder)
    }

    /// 向量检索
    async fn vector_search(&self, _query_embedding: &str) -> Result<Vec<RetrievalResult>> {
        // TODO: 实际向量数据库查询（Qdrant/Milvus）
        // 计算余弦相似度并排序

        // Placeholder
        Ok(Vec::new())
    }

    /// 关键词检索
    async fn keyword_search(&self, query: &str) -> Result<Vec<RetrievalResult>> {
        let chunks = self.chunks.read().await;
        let query_lower = query.to_lowercase();

        let mut results: Vec<RetrievalResult> = chunks
            .values()
            .filter(|chunk| chunk.content.to_lowercase().contains(&query_lower))
            .map(|chunk| RetrievalResult {
                chunk: chunk.clone(),
                score: 0.8, // 简化评分
                retrieval_method: "keyword".to_string(),
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(results)
    }

    /// 混合检索
    async fn hybrid_search(&self, query: &str, query_embedding: &str) -> Result<Vec<RetrievalResult>> {
        // 结合向量检索和关键词检索
        let vector_results = self.vector_search(query_embedding).await?;
        let keyword_results = self.keyword_search(query).await?;

        // TODO: 实现结果融合和重排序
        // 简化实现：合并结果
        let mut combined = vector_results;
        combined.extend(keyword_results);

        // 去重
        combined.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        combined.dedup_by(|a, b| a.chunk.chunk_id == b.chunk.chunk_id);

        Ok(combined)
    }

    fn compute_cache_key(&self, query: &str) -> String {
        // TODO: 使用更好的哈希（如blake3）
        format!("query_{}", query.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_document_indexing() {
        let engine = RagEngine::new(RagConfig::default());

        let doc = Document {
            document_id: "doc1".to_string(),
            title: "Test Document".to_string(),
            content: "This is a test document.\n\nIt has multiple paragraphs.".to_string(),
            doc_type: "txt".to_string(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let chunk_ids = engine.index_document(doc).await.unwrap();
        assert!(!chunk_ids.is_empty());
    }

    #[tokio::test]
    async fn test_keyword_search() {
        let engine = RagEngine::new(RagConfig {
            retrieval_mode: RetrievalMode::KeywordOnly,
            ..Default::default()
        });

        let doc = Document {
            document_id: "doc1".to_string(),
            title: "Rust Programming".to_string(),
            content: "Rust is a systems programming language.".to_string(),
            doc_type: "txt".to_string(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        engine.index_document(doc).await.unwrap();

        let results = engine.retrieve("Rust").await.unwrap();
        assert!(!results.is_empty());
    }
}
