//! 服务器状态查询模块
//!
//! 本模块提供服务器状态查询功能，包括：
//! - 服务器运行状态
//! - 性能指标（文档数量、内存使用等）
//! - 错误统计
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use summer_lsp::status::ServerStatus;
//!
//! let status = ServerStatus::new();
//! status.increment_document_count();
//! status.record_error();
//!
//! let metrics = status.get_metrics();
//! println!("Documents: {}", metrics.document_count);
//! println!("Errors: {}", metrics.error_count);
//! ```

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 服务器状态跟踪器
///
/// 使用原子操作跟踪服务器状态和性能指标，确保线程安全
#[derive(Clone)]
pub struct ServerStatus {
    /// 服务器启动时间
    start_time: Arc<Instant>,
    /// 打开的文档数量
    document_count: Arc<AtomicUsize>,
    /// 总请求数
    request_count: Arc<AtomicU64>,
    /// 总错误数
    error_count: Arc<AtomicU64>,
    /// 补全请求数
    completion_count: Arc<AtomicU64>,
    /// 悬停请求数
    hover_count: Arc<AtomicU64>,
    /// 诊断发布数
    diagnostic_count: Arc<AtomicU64>,
}

impl ServerStatus {
    /// 创建新的服务器状态跟踪器
    pub fn new() -> Self {
        Self {
            start_time: Arc::new(Instant::now()),
            document_count: Arc::new(AtomicUsize::new(0)),
            request_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            completion_count: Arc::new(AtomicU64::new(0)),
            hover_count: Arc::new(AtomicU64::new(0)),
            diagnostic_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 增加文档计数
    pub fn increment_document_count(&self) {
        self.document_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 减少文档计数
    pub fn decrement_document_count(&self) {
        self.document_count.fetch_sub(1, Ordering::Relaxed);
    }

    /// 记录请求
    pub fn record_request(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录错误
    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录补全请求
    pub fn record_completion(&self) {
        self.completion_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录悬停请求
    pub fn record_hover(&self) {
        self.hover_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录诊断发布
    pub fn record_diagnostic(&self) {
        self.diagnostic_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 获取服务器运行时长
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// 获取性能指标
    pub fn get_metrics(&self) -> ServerMetrics {
        let uptime = self.uptime();

        ServerMetrics {
            uptime_seconds: uptime.as_secs(),
            document_count: self.document_count.load(Ordering::Relaxed),
            request_count: self.request_count.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            completion_count: self.completion_count.load(Ordering::Relaxed),
            hover_count: self.hover_count.load(Ordering::Relaxed),
            diagnostic_count: self.diagnostic_count.load(Ordering::Relaxed),
            requests_per_second: if uptime.as_secs() > 0 {
                self.request_count.load(Ordering::Relaxed) as f64 / uptime.as_secs() as f64
            } else {
                0.0
            },
            error_rate: if self.request_count.load(Ordering::Relaxed) > 0 {
                self.error_count.load(Ordering::Relaxed) as f64
                    / self.request_count.load(Ordering::Relaxed) as f64
            } else {
                0.0
            },
        }
    }

    /// 重置所有计数器（用于测试）
    #[cfg(test)]
    pub fn reset(&self) {
        self.document_count.store(0, Ordering::Relaxed);
        self.request_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.completion_count.store(0, Ordering::Relaxed);
        self.hover_count.store(0, Ordering::Relaxed);
        self.diagnostic_count.store(0, Ordering::Relaxed);
    }
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// 服务器性能指标
///
/// 包含服务器运行状态和性能统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    /// 运行时长（秒）
    pub uptime_seconds: u64,
    /// 当前打开的文档数量
    pub document_count: usize,
    /// 总请求数
    pub request_count: u64,
    /// 总错误数
    pub error_count: u64,
    /// 补全请求数
    pub completion_count: u64,
    /// 悬停请求数
    pub hover_count: u64,
    /// 诊断发布数
    pub diagnostic_count: u64,
    /// 每秒请求数
    pub requests_per_second: f64,
    /// 错误率（错误数/总请求数）
    pub error_rate: f64,
}

impl ServerMetrics {
    /// 格式化为人类可读的字符串
    pub fn format(&self) -> String {
        format!(
            "Server Status:\n\
             - Uptime: {}s\n\
             - Documents: {}\n\
             - Requests: {} ({:.2} req/s)\n\
             - Errors: {} ({:.2}% error rate)\n\
             - Completions: {}\n\
             - Hovers: {}\n\
             - Diagnostics: {}",
            self.uptime_seconds,
            self.document_count,
            self.request_count,
            self.requests_per_second,
            self.error_count,
            self.error_rate * 100.0,
            self.completion_count,
            self.hover_count,
            self.diagnostic_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_server_status_creation() {
        let status = ServerStatus::new();
        let metrics = status.get_metrics();

        assert_eq!(metrics.document_count, 0);
        assert_eq!(metrics.request_count, 0);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.completion_count, 0);
        assert_eq!(metrics.hover_count, 0);
        assert_eq!(metrics.diagnostic_count, 0);
    }

    #[test]
    fn test_document_count() {
        let status = ServerStatus::new();

        status.increment_document_count();
        status.increment_document_count();
        assert_eq!(status.get_metrics().document_count, 2);

        status.decrement_document_count();
        assert_eq!(status.get_metrics().document_count, 1);
    }

    #[test]
    fn test_request_tracking() {
        let status = ServerStatus::new();

        status.record_request();
        status.record_request();
        status.record_request();

        let metrics = status.get_metrics();
        assert_eq!(metrics.request_count, 3);
    }

    #[test]
    fn test_error_tracking() {
        let status = ServerStatus::new();

        status.record_request();
        status.record_request();
        status.record_error();

        let metrics = status.get_metrics();
        assert_eq!(metrics.error_count, 1);
        assert_eq!(metrics.error_rate, 0.5); // 1 error / 2 requests
    }

    #[test]
    fn test_completion_tracking() {
        let status = ServerStatus::new();

        status.record_completion();
        status.record_completion();

        let metrics = status.get_metrics();
        assert_eq!(metrics.completion_count, 2);
    }

    #[test]
    fn test_hover_tracking() {
        let status = ServerStatus::new();

        status.record_hover();
        status.record_hover();
        status.record_hover();

        let metrics = status.get_metrics();
        assert_eq!(metrics.hover_count, 3);
    }

    #[test]
    fn test_diagnostic_tracking() {
        let status = ServerStatus::new();

        status.record_diagnostic();

        let metrics = status.get_metrics();
        assert_eq!(metrics.diagnostic_count, 1);
    }

    #[test]
    fn test_uptime() {
        let status = ServerStatus::new();

        // 等待一小段时间
        thread::sleep(Duration::from_millis(100));

        let uptime = status.uptime();
        assert!(uptime.as_millis() >= 100);
    }

    #[test]
    fn test_requests_per_second() {
        let status = ServerStatus::new();

        // 等待至少 1 秒以获得有意义的 RPS
        thread::sleep(Duration::from_secs(1));

        status.record_request();
        status.record_request();

        let metrics = status.get_metrics();
        // RPS 应该接近 2（可能略小，因为等待了 1 秒多一点）
        assert!(metrics.requests_per_second > 0.0);
        assert!(metrics.requests_per_second <= 2.0);
    }

    #[test]
    fn test_metrics_format() {
        let status = ServerStatus::new();

        status.increment_document_count();
        status.record_request();
        status.record_completion();

        let metrics = status.get_metrics();
        let formatted = metrics.format();

        assert!(formatted.contains("Server Status:"));
        assert!(formatted.contains("Documents: 1"));
        assert!(formatted.contains("Requests: 1"));
        assert!(formatted.contains("Completions: 1"));
    }

    #[test]
    fn test_concurrent_updates() {
        let status = ServerStatus::new();
        let status_clone = status.clone();

        // 在多个线程中并发更新
        let handle1 = thread::spawn(move || {
            for _ in 0..100 {
                status_clone.record_request();
            }
        });

        let status_clone2 = status.clone();
        let handle2 = thread::spawn(move || {
            for _ in 0..100 {
                status_clone2.record_request();
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        let metrics = status.get_metrics();
        assert_eq!(metrics.request_count, 200);
    }

    #[test]
    fn test_zero_division_safety() {
        let status = ServerStatus::new();

        // 没有请求时，错误率应该是 0
        let metrics = status.get_metrics();
        assert_eq!(metrics.error_rate, 0.0);

        // 运行时间为 0 时，RPS 应该是 0
        // 注意：由于 Instant::now() 的精度，这个测试可能不稳定
        // 但代码应该能处理这种情况
        assert!(metrics.requests_per_second >= 0.0);
    }

    #[test]
    fn test_reset() {
        let status = ServerStatus::new();

        status.increment_document_count();
        status.record_request();
        status.record_error();

        status.reset();

        let metrics = status.get_metrics();
        assert_eq!(metrics.document_count, 0);
        assert_eq!(metrics.request_count, 0);
        assert_eq!(metrics.error_count, 0);
    }
}
