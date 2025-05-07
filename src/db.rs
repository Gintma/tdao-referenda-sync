
use tokio_postgres::{Client, NoTls};
use tokio::task;
use anyhow::Result;
use log::error; 

/// 数据库客户端封装
pub struct Db {
    client: Client,
}

impl Db {
    /// 连接数据库并启动后台连接任务
    pub async fn connect(db_url: &str) -> Result<Self> {
        let (client, conn) = tokio_postgres::connect(db_url, NoTls).await?;
        task::spawn(async move {
            if let Err(e) = conn.await {
                error!("❗️ Postgres 连接错误：{}", e);
            }
        });
        Ok(Db { client })
    }

    /// 初始化数据库表和索引
    pub async fn init_schema(&self) -> Result<()> {
        self.client.execute(
            "CREATE TABLE IF NOT EXISTS referenda (
                id SERIAL PRIMARY KEY,
                referendum_index INTEGER UNIQUE
            )",
            &[],
        ).await?;
        self.client.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_referendum_index \
             ON referenda (referendum_index)",
            &[],
        ).await?;
        Ok(())
    }

    /// 获取已同步的所有公投编号（按编号升序）
    pub async fn get_existing_indices(&self) -> Result<Vec<i32>> {
        let rows = self.client
            .query("SELECT referendum_index FROM referenda ORDER BY referendum_index", &[])
            .await?;
        Ok(rows.iter().map(|r| r.get(0)).collect())
    }

    /// 插入新的公投编号记录
    pub async fn insert_referendum(&self, referendum_index: u32) -> Result<u64> {
        let idx = referendum_index as i32;
        let count = self.client
            .execute(
                "INSERT INTO referenda (referendum_index) VALUES ($1)",
                &[&idx],
            )
            .await?;
        Ok(count)
    }
}
