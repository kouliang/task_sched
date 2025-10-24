use std::fmt;

/// 调度器错误类型
#[derive(Debug, Clone)]
pub enum SchedulerError {
    /// 调度器已关闭
    Shutdown,
    /// 任务ID重复（正在运行中）
    DuplicateTaskId(String),
    /// 任务执行超时
    Timeout,
    /// 其他错误
    Other(String),
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchedulerError::Shutdown => write!(f, "调度器已关闭"),
            SchedulerError::DuplicateTaskId(id) => write!(f, "任务ID重复: {}", id),
            SchedulerError::Timeout => write!(f, "任务执行超时"),
            SchedulerError::Other(msg) => write!(f, "其他错误: {}", msg),
        }
    }
}

impl std::error::Error for SchedulerError {}

/// 任务执行错误类型
#[derive(Debug, Clone)]
pub enum TaskError {
    /// 任务执行失败
    ExecutionFailed(String),
    /// 任务被取消
    Cancelled,
    /// 任务超时
    Timeout,
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskError::ExecutionFailed(msg) => write!(f, "任务执行失败: {}", msg),
            TaskError::Cancelled => write!(f, "任务被取消"),
            TaskError::Timeout => write!(f, "任务超时"),
        }
    }
}

impl std::error::Error for TaskError {}
