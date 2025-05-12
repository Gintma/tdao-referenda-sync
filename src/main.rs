

mod config;
mod db;
mod models;
mod service;

use tokio::time::{interval, MissedTickBehavior};
use anyhow::Result;
use dotenv::dotenv;
use env_logger::Env;
use log::{info, error};
use reqwest::Client;
use std::time::Duration;
use config::Config;
use db::Db;
use service::run_sync;
use chrono::{Local, Duration as ChronoDuration};


#[tokio::main]
async fn main() -> Result<()> {
    // 先加载 .env，再加载环境变量
    dotenv().ok();

    // 初始化日志：从环境变量 RUST_LOG 读取过滤级别，默认为 info
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // 加载程序配置
    let cfg = Config::from_env()?;
    info!("🔧 使用的 OpenSquare 空间：{}", cfg.open_square_space);

    // 构建 HTTP 客户端
    let http = Client::builder()
        .timeout(cfg.http_timeout)
        .build()?;

    // 连接数据库
    let db = Db::connect(&cfg.postgres_url).await?;

  

    // 创建一个 Interval
    let mut ticker = interval(Duration::from_secs(60 * 30));


    // 如果错过执行，延迟到下一个周期，而不是立即补跑
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        // 2. 等待下一个 tick
        ticker.tick().await;

        // 3. 执行前日志
        let now = Local::now();
        info!("🔄 [{}] 开始定时同步...", now.format("%Y-%m-%d %H:%M:%S"));

        // 4. 真正的同步逻辑
        if let Err(err) = run_sync(&http, &db, &cfg).await {
            error!("❌ 定时同步失败: {:?}", err);
        } else {
            info!("✅ 定时同步完成");
        }

        // 5. 计算并打印下一次执行时间
        let next = now + ChronoDuration::minutes(30);
        info!("⏱ 下一次定时同步将于 {}", next.format("%Y-%m-%d %H:%M:%S"));
}
}
