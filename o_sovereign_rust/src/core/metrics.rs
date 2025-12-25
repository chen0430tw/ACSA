// Metrics - 系统监控和可观测性
// Prometheus指标导出和健康检查
//
// 核心功能：
// 1. Prometheus指标收集
// 2. 健康检查端点
// 3. 系统性能监控
// 4. 自定义指标
// 5. OpenTelemetry集成（可选）

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// 指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// 计数器（只增不减）
    Counter,
    /// 仪表盘（可增可减）
    Gauge,
    /// 直方图（分布统计）
    Histogram,
    /// 摘要（分位数）
    Summary,
}

/// 指标值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// 指标名称
    pub name: String,
    /// 指标类型
    pub metric_type: MetricType,
    /// 当前值
    pub value: f64,
    /// 标签
    pub labels: HashMap<String, String>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

/// 健康检查状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 降级（部分功能不可用）
    Degraded,
    /// 不健康
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// 整体状态
    pub status: HealthStatus,
    /// 版本信息
    pub version: String,
    /// 启动时间
    pub started_at: DateTime<Utc>,
    /// 运行时长（秒）
    pub uptime_secs: u64,
    /// 组件健康状态
    pub components: HashMap<String, ComponentHealth>,
}

/// 组件健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// 组件名称
    pub name: String,
    /// 状态
    pub status: HealthStatus,
    /// 详细信息
    pub details: Option<String>,
    /// 最后检查时间
    pub last_check: DateTime<Utc>,
}

/// 系统指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU使用率 (0.0-1.0)
    pub cpu_usage: f64,
    /// 内存使用率 (0.0-1.0)
    pub memory_usage: f64,
    /// 已用内存（MB）
    pub memory_used_mb: u64,
    /// 总内存（MB）
    pub memory_total_mb: u64,
    /// 磁盘使用率 (0.0-1.0)
    pub disk_usage: f64,
    /// 网络接收字节数
    pub network_rx_bytes: u64,
    /// 网络发送字节数
    pub network_tx_bytes: u64,
}

/// 应用指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    /// 活跃连接数
    pub active_connections: u64,
    /// AI调用次数
    pub ai_api_calls: u64,
    /// AI调用成本
    pub ai_api_cost: f64,
}

/// 指标收集器
pub struct MetricsCollector {
    /// 自定义指标
    metrics: Arc<RwLock<HashMap<String, MetricValue>>>,
    /// 系统指标
    system_metrics: Arc<RwLock<SystemMetrics>>,
    /// 应用指标
    app_metrics: Arc<RwLock<ApplicationMetrics>>,
    /// 服务启动时间
    started_at: DateTime<Utc>,
    /// 版本信息
    version: String,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    pub fn new(version: String) -> Self {
        info!("📊 Initializing Metrics Collector (v{})", version);

        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            app_metrics: Arc::new(RwLock::new(ApplicationMetrics::default())),
            started_at: Utc::now(),
            version,
        }
    }

    /// 记录计数器
    pub async fn increment_counter(&self, name: &str, labels: HashMap<String, String>) {
        let mut metrics = self.metrics.write().await;

        let key = self.build_metric_key(name, &labels);
        let metric = metrics
            .entry(key.clone())
            .or_insert_with(|| MetricValue {
                name: name.to_string(),
                metric_type: MetricType::Counter,
                value: 0.0,
                labels: labels.clone(),
                updated_at: Utc::now(),
            });

        metric.value += 1.0;
        metric.updated_at = Utc::now();
    }

    /// 设置仪表盘值
    pub async fn set_gauge(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        let mut metrics = self.metrics.write().await;

        let key = self.build_metric_key(name, &labels);
        metrics.insert(
            key,
            MetricValue {
                name: name.to_string(),
                metric_type: MetricType::Gauge,
                value,
                labels,
                updated_at: Utc::now(),
            },
        );
    }

    /// 记录直方图
    pub async fn observe_histogram(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        // TODO: 实现真实的直方图统计（需要维护桶）
        let mut metrics = self.metrics.write().await;

        let key = self.build_metric_key(name, &labels);
        metrics.insert(
            key,
            MetricValue {
                name: name.to_string(),
                metric_type: MetricType::Histogram,
                value,
                labels,
                updated_at: Utc::now(),
            },
        );
    }

    /// 更新系统指标
    pub async fn update_system_metrics(&self) {
        // TODO: 使用 sysinfo crate 获取真实的系统信息
        let mut metrics = self.system_metrics.write().await;

        // Placeholder值
        metrics.cpu_usage = 0.0;
        metrics.memory_usage = 0.0;
        metrics.memory_used_mb = 0;
        metrics.memory_total_mb = 0;
        metrics.disk_usage = 0.0;
    }

    /// 记录请求
    pub async fn record_request(&self, success: bool, response_time_ms: f64) {
        let mut metrics = self.app_metrics.write().await;

        metrics.total_requests += 1;
        if success {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }

        // 更新平均响应时间
        let total = metrics.total_requests as f64;
        metrics.avg_response_time_ms =
            (metrics.avg_response_time_ms * (total - 1.0) + response_time_ms) / total;
    }

    /// 记录AI调用
    pub async fn record_ai_call(&self, cost: f64) {
        let mut metrics = self.app_metrics.write().await;
        metrics.ai_api_calls += 1;
        metrics.ai_api_cost += cost;
    }

    /// 设置活跃连接数
    pub async fn set_active_connections(&self, count: u64) {
        let mut metrics = self.app_metrics.write().await;
        metrics.active_connections = count;
    }

    /// 获取健康检查结果
    pub async fn get_health_check(&self, components: HashMap<String, ComponentHealth>) -> HealthCheck {
        // 计算运行时长
        let uptime_secs = (Utc::now() - self.started_at).num_seconds() as u64;

        // 确定整体状态
        let status = if components.values().all(|c| c.status == HealthStatus::Healthy) {
            HealthStatus::Healthy
        } else if components.values().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Degraded
        };

        HealthCheck {
            status,
            version: self.version.clone(),
            started_at: self.started_at,
            uptime_secs,
            components,
        }
    }

    /// 获取系统指标
    pub async fn get_system_metrics(&self) -> SystemMetrics {
        self.system_metrics.read().await.clone()
    }

    /// 获取应用指标
    pub async fn get_app_metrics(&self) -> ApplicationMetrics {
        self.app_metrics.read().await.clone()
    }

    /// 导出Prometheus格式
    pub async fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // 导出自定义指标
        let metrics = self.metrics.read().await;
        for (_, metric) in metrics.iter() {
            let labels = self.format_labels(&metric.labels);
            output.push_str(&format!(
                "# TYPE {} {}\n",
                metric.name,
                self.metric_type_to_string(metric.metric_type)
            ));
            output.push_str(&format!("{}{} {}\n", metric.name, labels, metric.value));
        }

        // 导出系统指标
        let sys_metrics = self.system_metrics.read().await;
        output.push_str(&format!("# TYPE acsa_cpu_usage gauge\n"));
        output.push_str(&format!("acsa_cpu_usage {}\n", sys_metrics.cpu_usage));
        output.push_str(&format!("# TYPE acsa_memory_usage gauge\n"));
        output.push_str(&format!("acsa_memory_usage {}\n", sys_metrics.memory_usage));

        // 导出应用指标
        let app_metrics = self.app_metrics.read().await;
        output.push_str(&format!("# TYPE acsa_total_requests counter\n"));
        output.push_str(&format!("acsa_total_requests {}\n", app_metrics.total_requests));
        output.push_str(&format!("# TYPE acsa_avg_response_time gauge\n"));
        output.push_str(&format!(
            "acsa_avg_response_time {}\n",
            app_metrics.avg_response_time_ms
        ));

        output
    }

    // ===== 内部辅助方法 =====

    fn build_metric_key(&self, name: &str, labels: &HashMap<String, String>) -> String {
        let mut key = name.to_string();
        let mut label_pairs: Vec<_> = labels.iter().collect();
        label_pairs.sort_by_key(|(k, _)| *k);

        for (k, v) in label_pairs {
            key.push_str(&format!(":{}={}", k, v));
        }
        key
    }

    fn format_labels(&self, labels: &HashMap<String, String>) -> String {
        if labels.is_empty() {
            return String::new();
        }

        let mut pairs: Vec<String> = labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect();
        pairs.sort();

        format!("{{{}}}", pairs.join(","))
    }

    fn metric_type_to_string(&self, metric_type: MetricType) -> &str {
        match metric_type {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge",
            MetricType::Histogram => "histogram",
            MetricType::Summary => "summary",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new("0.1.0".to_string());

        collector
            .increment_counter("test_counter", HashMap::new())
            .await;

        collector
            .set_gauge("test_gauge", 42.0, HashMap::new())
            .await;

        let prometheus = collector.export_prometheus().await;
        assert!(prometheus.contains("test_counter"));
        assert!(prometheus.contains("test_gauge"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let collector = MetricsCollector::new("0.1.0".to_string());

        let mut components = HashMap::new();
        components.insert(
            "database".to_string(),
            ComponentHealth {
                name: "database".to_string(),
                status: HealthStatus::Healthy,
                details: None,
                last_check: Utc::now(),
            },
        );

        let health = collector.get_health_check(components).await;
        assert_eq!(health.status, HealthStatus::Healthy);
    }
}
