//! 任务扫描器模块
//!
//! 扫描项目中的所有定时任务定义（带有 #[cron], #[fix_delay], #[fix_rate] 的函数）

use crate::analysis::rust::macro_analyzer::{JobMacro, MacroAnalyzer, SummerMacro};
use crate::protocol::types::{LocationResponse, PositionResponse, RangeResponse};
use lsp_types::Url;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// 任务扫描器
pub struct JobScanner {
    macro_analyzer: MacroAnalyzer,
}

impl JobScanner {
    /// 创建新的任务扫描器
    pub fn new() -> Self {
        Self {
            macro_analyzer: MacroAnalyzer::new(),
        }
    }

    /// 扫描项目中的所有任务
    ///
    /// # Arguments
    ///
    /// * `project_path` - 项目根目录路径
    ///
    /// # Returns
    ///
    /// 返回扫描到的所有任务信息
    pub fn scan_jobs(&self, project_path: &Path) -> Result<Vec<JobInfoResponse>, ScanError> {
        let mut jobs = Vec::new();

        // 查找 src 目录
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Err(ScanError::InvalidProject(
                "src directory not found".to_string(),
            ));
        }

        // 遍历所有 Rust 文件
        for entry in WalkDir::new(&src_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            let file_path = entry.path();

            // 读取文件内容
            let content = match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => {
                    tracing::warn!("Failed to read file {:?}: {}", file_path, e);
                    continue;
                }
            };

            // 解析文件
            let file_url = match Url::from_file_path(file_path) {
                Ok(url) => url,
                Err(_) => {
                    tracing::warn!("Failed to convert path to URL: {:?}", file_path);
                    continue;
                }
            };

            let rust_doc = match self.macro_analyzer.parse(file_url.clone(), content) {
                Ok(doc) => doc,
                Err(e) => {
                    tracing::warn!("Failed to parse file {:?}: {}", file_path, e);
                    continue;
                }
            };

            // 提取宏信息
            let rust_doc = match self.macro_analyzer.extract_macros(rust_doc) {
                Ok(doc) => doc,
                Err(e) => {
                    tracing::warn!("Failed to extract macros from {:?}: {}", file_path, e);
                    continue;
                }
            };

            // 提取任务信息
            for summer_macro in &rust_doc.macros {
                if let SummerMacro::Job(job_macro) = summer_macro {
                    let (job_type, schedule) = match job_macro {
                        JobMacro::Cron { expression, .. } => (JobType::Cron, expression.clone()),
                        JobMacro::FixDelay { seconds, .. } => {
                            (JobType::FixDelay, format!("{} seconds", seconds))
                        }
                        JobMacro::FixRate { seconds, .. } => {
                            (JobType::FixRate, format!("{} seconds", seconds))
                        }
                    };

                    let range = match job_macro {
                        JobMacro::Cron { range, .. }
                        | JobMacro::FixDelay { range, .. }
                        | JobMacro::FixRate { range, .. } => range,
                    };

                    jobs.push(JobInfoResponse {
                        name: "job_function".to_string(), // TODO: 从函数名提取
                        job_type,
                        schedule,
                        location: LocationResponse {
                            uri: file_url.to_string(),
                            range: RangeResponse {
                                start: PositionResponse {
                                    line: range.start.line,
                                    character: range.start.character,
                                },
                                end: PositionResponse {
                                    line: range.end.line,
                                    character: range.end.character,
                                },
                            },
                        },
                    });
                }
            }
        }

        Ok(jobs)
    }
}

impl Default for JobScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    /// Cron 表达式任务
    Cron,
    /// 固定延迟任务
    FixDelay,
    /// 固定频率任务
    FixRate,
}

/// 任务信息响应（用于 JSON 序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfoResponse {
    /// 任务名称（函数名）
    pub name: String,
    /// 任务类型
    #[serde(rename = "jobType")]
    pub job_type: JobType,
    /// 调度表达式
    pub schedule: String,
    /// 源代码位置
    pub location: LocationResponse,
}

/// summer/jobs 请求参数
#[derive(Debug, Deserialize)]
pub struct JobsRequest {
    /// 应用路径
    #[serde(rename = "appPath")]
    pub app_path: String,
}

/// summer/jobs 响应
#[derive(Debug, Serialize)]
pub struct JobsResponse {
    /// 任务列表
    pub jobs: Vec<JobInfoResponse>,
}

/// 扫描错误
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("Failed to read file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Invalid project structure: {0}")]
    InvalidProject(String),

    #[error("No jobs found")]
    NoJobs,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_scanner_new() {
        let _scanner = JobScanner::new();
        // 验证扫描器创建成功（不会 panic）
    }

    #[test]
    fn test_job_scanner_default() {
        let _scanner = JobScanner::default();
        // 验证默认扫描器创建成功（不会 panic）
    }
}
