# 使用指南

## 目录

- [快速开始](#快速开始)
- [核心概念](#核心概念)
- [功能示例](#功能示例)

## 快速开始

### 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
task_sched = { path = https://github.com/kouliang/task_sched }
```

### 最简示例

```rust
use task_sched::{Task, TaskScheduler};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建调度器，最大并发数为3
    let scheduler = TaskScheduler::new(3);
    
    // 创建并提交任务
    let task = Task::new(
        "my-first-task".to_string(),
        async {
            println!("任务执行中...");
            Ok("完成")
        },
        Some(Duration::from_secs(10)), // 超时设置
    );
    
    scheduler.submit(task).await?;
    
    // 优雅关闭
    scheduler.shutdown(Some(Duration::from_secs(30))).await?;
    
    Ok(())
}
```

## 核心概念

### TaskScheduler

任务调度器是系统的核心组件，负责管理和调度所有异步任务。

**创建调度器：**
```rust
let scheduler = TaskScheduler::new(max_concurrency);
```

- `max_concurrency`: 最大并发任务数

**关键方法：**
- `submit()`: 提交任务
- `shutdown()`: 优雅关闭
- `running_count()`: 获取运行中的任务数量

### Task

Task 代表一个可执行的异步任务。

**创建任务：**
```rust
let task = Task::new(
    task_id,        // 唯一标识符
    async { ... },  // 异步函数
    timeout         // 可选的超时时间
);
```

## 功能示例

### 1. 并发控制

```rust
let scheduler = TaskScheduler::new(5); // 最多5个任务并发

for i in 0..10 {
    let task = Task::new(
        format!("task-{}", i),
        async move {
            // 执行耗时操作
            tokio::time::sleep(Duration::from_secs(1)).await;
            Ok(i)
        },
        None,
    );
    scheduler.submit(task).await?;
}

// 即使提交了10个任务，最多只有5个同时运行
```

### 2. 任务去重

```rust
let scheduler = TaskScheduler::new(2);

// 第一个任务
let task1 = Task::new(
    "unique-id".to_string(),
    async {
        tokio::time::sleep(Duration::from_secs(5)).await;
        Ok("first")
    },
    None,
);
scheduler.submit(task1).await?;

// 相同ID的第二个任务会被拒绝
let task2 = Task::new(
    "unique-id".to_string(),
    async { Ok("second") },
    None,
);

match scheduler.submit(task2).await {
    Err(SchedulerError::DuplicateTaskId(id)) => {
        println!("任务ID重复: {}", id);
    }
    _ => {}
}
```

### 3. 错误处理

```rust
use task_sched::TaskError;

let task: Task<_, ()> = Task::new(
    "error-task".to_string(),
    async {
        // 模拟错误
        Err(task_sched::TaskError::ExecutionFailed("数据库连接失败".to_string()))
    },
    None,
);

scheduler.submit(task).await?;

// 错误会被自动记录到日志
```

### 4. 超时控制

```rust
let task = Task::new(
    "long-task".to_string(),
    async {
        // 这个任务需要10秒
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok("完成")
    },
    Some(Duration::from_secs(5)), // 5秒超时
);

scheduler.submit(task).await?;
// 任务会在5秒后超时并被终止
```

### 5. 优雅关闭

```rust
// 开始关闭过程
scheduler.shutdown(Some(Duration::from_secs(30))).await?;

// 调度器会：
// 1. 拒绝新的任务提交
// 2. 等待所有运行中的任务完成
// 3. 如果30秒后还有任务未完成，强制取消
```

## 最佳实践

### 1. 合理设置并发数

```rust
// CPU密集型任务
let cpu_scheduler = TaskScheduler::new(num_cpus::get());

// IO密集型任务
let io_scheduler = TaskScheduler::new(num_cpus::get() * 4);

// 外部API调用（避免触发限流）
let api_scheduler = TaskScheduler::new(10);
```

### 2. 使用有意义的任务ID

```rust
// ✓ 好的做法
let task_id = format!("user-{}-email", user_id);

// ✗ 不好的做法
let task_id = uuid::Uuid::new_v4().to_string(); // 无法去重
```

### 3. 设置合理的超时时间

```rust
// 快速API调用
let task = Task::new(id, future, Some(Duration::from_secs(5)));

// 文件处理
let task = Task::new(id, future, Some(Duration::from_secs(60)));

// 长时间运行的任务
let task = Task::new(id, future, Some(Duration::from_secs(3600)));
```

### 4. 日志配置

```rust
use tracing::Level;
use tracing_subscriber;

fn init_logging() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)  // 生产环境使用INFO
        .with_target(false)
        .init();
}
```

### 5. 错误处理

```rust
let task: Task<_, Result<String, MyError>> = Task::new(
    task_id,
    async {
        match risky_operation().await {
            Ok(result) => Ok(result),
            Err(e) => {
                // 记录详细错误信息
            Err(task_sched::TaskError::ExecutionFailed(
                format!("操作失败: {:?}", e)
            ))
            }
        }
    },
    timeout,
);
```

