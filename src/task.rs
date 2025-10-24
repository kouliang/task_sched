use std::fmt;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, Mutex};
use tokio::time::timeout;

use crate::error::TaskError;

/// 任务状态
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    /// 等待执行
    Pending,
    /// 正在执行
    Running,
    /// 执行成功
    Completed,
    /// 执行失败
    Failed,
    /// 被跳过（重复ID）
    SkippedDuplicate,
    /// 被取消
    Cancelled,
}

/// 任务结果
#[derive(Debug, Clone)]
pub enum TaskResult {
    /// 成功
    Success(String),
    /// 失败
    Failure(TaskError),
    /// 跳过
    Skipped,
}

/// 任务定义
pub struct Task<F, R>
where
    F: Future<Output = Result<R, TaskError>> + Send + 'static,
    R: Send + 'static,
{
    pub id: String,
    pub future: F,
    pub timeout: Option<Duration>,
    pub status: Arc<Mutex<TaskStatus>>,
    pub result_sender: Option<oneshot::Sender<TaskResult>>,
}

impl<F, R> Task<F, R>
where
    F: Future<Output = Result<R, TaskError>> + Send + 'static,
    R: Send + 'static,
{
    pub fn new(id: String, future: F, timeout: Option<Duration>) -> Self {
        Self {
            id,
            future,
            timeout,
            status: Arc::new(Mutex::new(TaskStatus::Pending)),
            result_sender: None,
        }
    }

    /// 执行任务
    pub async fn execute(self) -> TaskResult {
        // 更新状态为运行中
        {
            let mut status = self.status.lock().await;
            *status = TaskStatus::Running;
        }

        let result = if let Some(timeout_duration) = self.timeout {
            // 带超时的执行
            match timeout(timeout_duration, self.future).await {
                Ok(Ok(_result)) => TaskResult::Success("任务执行成功".to_string()),
                Ok(Err(task_error)) => TaskResult::Failure(task_error),
                Err(_) => TaskResult::Failure(TaskError::Timeout),
            }
        } else {
            // 无超时的执行
            match self.future.await {
                Ok(_result) => TaskResult::Success("任务执行成功".to_string()),
                Err(task_error) => TaskResult::Failure(task_error),
            }
        };

        // 更新最终状态
        {
            let mut status = self.status.lock().await;
            *status = match &result {
                TaskResult::Success(_) => TaskStatus::Completed,
                TaskResult::Failure(_) => TaskStatus::Failed,
                TaskResult::Skipped => TaskStatus::SkippedDuplicate,
            };
        }

        result
    }

    /// 获取任务状态
    pub async fn get_status(&self) -> TaskStatus {
        self.status.lock().await.clone()
    }
}

impl<F, R> fmt::Debug for Task<F, R>
where
    F: Future<Output = Result<R, TaskError>> + Send + 'static,
    R: Send + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("timeout", &self.timeout)
            .finish()
    }
}
