use std::time::Duration;
use task_sched::{Task, TaskError, TaskScheduler};
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("=== 基本使用示例 ===");

    // 创建调度器，最大并发数为2
    let scheduler = TaskScheduler::new(2);

    // 示例1: 提交多个任务
    info!("\n1. 提交多个任务测试并发控制");
    for i in 1..=5 {
        let task = Task::new(
            format!("task-{}", i),
            async move {
                info!("任务 {} 开始执行", i);
                sleep(Duration::from_secs(1)).await;
                info!("任务 {} 完成", i);
                Ok(format!("结果-{}", i))
            },
            Some(Duration::from_secs(10)),
        );
        scheduler.submit(task).await?;
    }

    // 等待一段时间
    sleep(Duration::from_millis(100)).await;
    info!("当前运行任务数: {}", scheduler.running_count().await);

    // 等待所有任务完成
    sleep(Duration::from_secs(3)).await;

    // 示例2: 任务去重
    info!("\n2. 测试任务去重");
    let task1 = Task::new(
        "unique-task".to_string(),
        async {
            sleep(Duration::from_secs(2)).await;
            Ok("第一个任务")
        },
        None,
    );
    scheduler.submit(task1).await?;

    let task2 = Task::new(
        "unique-task".to_string(),
        async { Ok("第二个任务（应该被拒绝）") },
        None,
    );

    match scheduler.submit(task2).await {
        Ok(_) => info!("意外：第二个任务被接受了"),
        Err(e) => info!("预期：第二个任务被拒绝 - {}", e),
    }

    // 等待第一个任务完成
    sleep(Duration::from_secs(3)).await;

    // 示例3: 错误处理
    info!("\n3. 测试错误处理");
    let task_error: Task<_, ()> = Task::new(
        "error-task".to_string(),
        async {
            sleep(Duration::from_millis(500)).await;
            Err(TaskError::ExecutionFailed("模拟的错误".to_string()))
        },
        None,
    );
    scheduler.submit(task_error).await?;

    sleep(Duration::from_secs(1)).await;

    // 优雅关闭
    info!("\n4. 优雅关闭调度器");
    scheduler.shutdown(Some(Duration::from_secs(5))).await?;

    info!("\n示例程序完成！");
    Ok(())
}
