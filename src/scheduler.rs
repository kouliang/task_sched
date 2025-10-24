use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};
use tracing::{error, info, warn};

use crate::error::{SchedulerError, TaskError};
use crate::task::{Task, TaskResult, TaskStatus};

/// 任务调度器
pub struct TaskScheduler {
    /// 最大并发数
    max_concurrency: usize,
    /// 信号量控制并发
    semaphore: Arc<Semaphore>,
    /// 正在运行的任务
    running_tasks: Arc<RwLock<HashMap<String, JoinHandle<TaskResult>>>>,
    /// 调度器状态
    is_shutdown: Arc<Mutex<bool>>,
}

impl TaskScheduler {
    /// 创建新的任务调度器
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            max_concurrency,
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            is_shutdown: Arc::new(Mutex::new(false)),
        }
    }

    /// 提交任务
    pub async fn submit<F, R>(&self, task: Task<F, R>) -> Result<(), SchedulerError>
    where
        F: std::future::Future<Output = Result<R, TaskError>> + Send + 'static,
        R: Send + 'static,
    {
        // 检查调度器是否已关闭
        {
            let is_shutdown = self.is_shutdown.lock().await;
            if *is_shutdown {
                return Err(SchedulerError::Shutdown);
            }
        }

        let task_id = task.id.clone();

        // 检查是否有相同ID的任务正在运行
        {
            let running_tasks = self.running_tasks.read().await;
            if running_tasks.contains_key(&task_id) {
                warn!("任务ID重复，跳过执行: {}", task_id);
                return Err(SchedulerError::DuplicateTaskId(task_id));
            }
        }

        // 获取信号量许可
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| SchedulerError::Other("无法获取信号量许可".to_string()))?;

        // 生成任务执行
        self.spawn_task(task, permit).await;

        Ok(())
    }

    /// 生成任务执行
    async fn spawn_task<F, R>(&self, task: Task<F, R>, permit: tokio::sync::OwnedSemaphorePermit)
    where
        F: std::future::Future<Output = Result<R, TaskError>> + Send + 'static,
        R: Send + 'static,
    {
        let task_id = task.id.clone();
        let task_id_for_map = task_id.clone();
        let running_tasks = self.running_tasks.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit; // 持有permit直到任务完成
            let result = task.execute().await;

            // 记录任务结果
            match &result {
                TaskResult::Success(msg) => {
                    info!("任务执行成功: {} - {}", task_id, msg);
                }
                TaskResult::Failure(err) => {
                    error!("任务执行失败: {} - {}", task_id, err);
                }
                TaskResult::Skipped => {
                    info!("任务被跳过: {}", task_id);
                }
            }

            // 从运行任务中移除
            {
                let mut running = running_tasks.write().await;
                running.remove(&task_id);
            }

            result
        });

        // 将任务加入运行列表
        {
            let mut running = self.running_tasks.write().await;
            running.insert(task_id_for_map, handle);
        }
    }

    /// 获取当前运行任务数量
    pub async fn running_count(&self) -> usize {
        self.running_tasks.read().await.len()
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        let running_tasks = self.running_tasks.read().await;
        if running_tasks.contains_key(task_id) {
            Some(TaskStatus::Running)
        } else {
            None
        }
    }

    /// 获取所有运行中的任务ID
    pub async fn get_running_task_ids(&self) -> Vec<String> {
        let running_tasks = self.running_tasks.read().await;
        running_tasks.keys().cloned().collect()
    }

    /// 优雅关闭调度器
    pub async fn shutdown(&self, timeout_duration: Option<Duration>) -> Result<(), SchedulerError> {
        // 设置关闭状态
        {
            let mut is_shutdown = self.is_shutdown.lock().await;
            *is_shutdown = true;
        }

        info!("开始关闭任务调度器...");

        // 等待所有任务完成或超时
        if let Some(duration) = timeout_duration {
            match timeout(duration, self.wait_for_completion()).await {
                Ok(_) => {
                    info!("所有任务已完成");
                }
                Err(_) => {
                    warn!("等待任务完成超时，强制关闭");
                    self.cancel_all_tasks().await;
                }
            }
        } else {
            self.wait_for_completion().await;
        }

        info!("任务调度器已关闭");
        Ok(())
    }

    /// 等待所有任务完成
    async fn wait_for_completion(&self) {
        loop {
            let running_count = self.running_count().await;

            if running_count == 0 {
                break;
            }

            info!("等待任务完成 - 运行中: {}", running_count);
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// 取消所有任务
    async fn cancel_all_tasks(&self) {
        let mut running_tasks = self.running_tasks.write().await;
        for (task_id, handle) in running_tasks.drain() {
            handle.abort();
            info!("已取消任务: {}", task_id);
        }
    }
}

impl Clone for TaskScheduler {
    fn clone(&self) -> Self {
        Self {
            max_concurrency: self.max_concurrency,
            semaphore: self.semaphore.clone(),
            running_tasks: self.running_tasks.clone(),
            is_shutdown: self.is_shutdown.clone(),
        }
    }
}
