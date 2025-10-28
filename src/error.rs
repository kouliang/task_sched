use thiserror::Error;

/// 调度器错误类型
#[derive(Debug, Clone, Error)]
pub enum SchedulerError {
    /// 调度器已关闭
    #[error("调度器已关闭")]
    Shutdown,
    /// 任务ID重复（正在运行中）
    #[error("任务ID重复: {0}")]
    DuplicateTaskId(String),
    /// 任务执行超时
    #[error("任务执行超时")]
    Timeout,
    /// 其他错误
    #[error("其他错误: {0}")]
    Other(String),
}

/// 任务执行错误类型
#[derive(Debug, Clone, Error)]
pub enum TaskError {
    /// 任务执行失败
    #[error("任务执行失败: {0}")]
    ExecutionFailed(String),
    /// 任务被取消
    #[error("任务被取消")]
    Cancelled,
    /// 任务超时
    #[error("任务超时")]
    Timeout,
}