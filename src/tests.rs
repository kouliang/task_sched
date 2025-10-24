use crate::{SchedulerError, Task, TaskError, TaskScheduler};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_basic_task_execution() {
    let scheduler = TaskScheduler::new(2);

    let task = Task::new(
        "test-task".to_string(),
        async {
            sleep(Duration::from_millis(100)).await;
            Ok("success")
        },
        Some(Duration::from_secs(1)),
    );

    assert!(scheduler.submit(task).await.is_ok());

    // 等待任务完成
    sleep(Duration::from_millis(200)).await;

    assert_eq!(scheduler.running_count().await, 0);
}

#[tokio::test]
async fn test_duplicate_task_id() {
    let scheduler = TaskScheduler::new(2);

    let task1 = Task::new(
        "duplicate-task".to_string(),
        async {
            sleep(Duration::from_millis(100)).await;
            Ok("first")
        },
        Some(Duration::from_secs(1)),
    );

    let task2 = Task::new(
        "duplicate-task".to_string(),
        async {
            sleep(Duration::from_millis(100)).await;
            Ok("second")
        },
        Some(Duration::from_secs(1)),
    );

    // 第一个任务应该成功提交
    assert!(scheduler.submit(task1).await.is_ok());

    // 第二个任务应该被拒绝
    match scheduler.submit(task2).await {
        Err(SchedulerError::DuplicateTaskId(id)) => {
            assert_eq!(id, "duplicate-task");
        }
        _ => panic!("应该检测到重复任务ID"),
    }
}

#[tokio::test]
async fn test_concurrency_limit() {
    let scheduler = TaskScheduler::new(2);

    // 提交3个任务，但并发限制为2
    for i in 0..3 {
        let task = Task::new(
            format!("task-{}", i),
            async move {
                sleep(Duration::from_millis(200)).await;
                Ok(format!("task-{}", i))
            },
            Some(Duration::from_secs(1)),
        );
        assert!(scheduler.submit(task).await.is_ok());
    }

    // 给一些时间让任务开始执行
    sleep(Duration::from_millis(50)).await;

    // 检查，应该最多有2个任务在运行
    let running_count = scheduler.running_count().await;
    assert!(
        running_count <= 2,
        "运行任务数: {}, 应该 <= 2",
        running_count
    );

    // 等待所有任务完成
    sleep(Duration::from_millis(500)).await;
    assert_eq!(scheduler.running_count().await, 0);
}

#[tokio::test]
async fn test_task_failure() {
    let scheduler = TaskScheduler::new(1);

    let task: Task<_, ()> = Task::new(
        "failing-task".to_string(),
        async { Err(TaskError::ExecutionFailed("测试失败".to_string())) },
        Some(Duration::from_secs(1)),
    );

    assert!(scheduler.submit(task).await.is_ok());

    // 等待任务完成
    sleep(Duration::from_millis(100)).await;
    assert_eq!(scheduler.running_count().await, 0);
}

#[tokio::test]
async fn test_shutdown() {
    let scheduler = TaskScheduler::new(1);

    let task: Task<_, &'static str> = Task::new(
        "long-task".to_string(),
        async {
            sleep(Duration::from_millis(500)).await;
            Ok("completed")
        },
        Some(Duration::from_secs(1)),
    );

    assert!(scheduler.submit(task).await.is_ok());

    // 立即关闭，应该等待任务完成
    let start = std::time::Instant::now();
    assert!(scheduler
        .shutdown(Some(Duration::from_secs(2)))
        .await
        .is_ok());
    let elapsed = start.elapsed();

    // 应该等待了至少500ms
    assert!(elapsed >= Duration::from_millis(500));
}

#[tokio::test]
async fn test_shutdown_after_shutdown() {
    let scheduler = TaskScheduler::new(1);

    // 第一次关闭
    assert!(scheduler
        .shutdown(Some(Duration::from_secs(1)))
        .await
        .is_ok());

    // 尝试提交任务到已关闭的调度器
    let task: Task<_, &'static str> = Task::new(
        "task-after-shutdown".to_string(),
        async { Ok("should not run") },
        Some(Duration::from_secs(1)),
    );

    match scheduler.submit(task).await {
        Err(SchedulerError::Shutdown) => {
            // 正确，调度器已关闭
        }
        _ => panic!("应该返回Shutdown错误"),
    }
}
