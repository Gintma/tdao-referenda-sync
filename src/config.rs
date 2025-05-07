use std::env;
use std::time::Duration;



/// 全局配置，从环境变量中加载，允许 .env 文件覆盖
///
/// 可配置项:
/// - OPEN_SQUARE_SPACE: OpenSquare 空间名称
/// - POSTGRES_URL: PostgreSQL 连接串
/// - HTTP_TIMEOUT_SECS: HTTP 请求超时时间（秒）
/// - SNAPSHOT_OFFSET: 块高度偏移
/// - MNEMONIC: 用于签名的助记词
/// - SUBSCAN_API_KEY: Subscan API Key
/// - PAGE_SIZE: 每次拉取公投条数，默认 50
pub struct Config {
    pub open_square_space: String,
    pub postgres_url: String,
    pub http_timeout: Duration,
    pub snapshot_offset: u64,
    pub mnemonic: String,
    pub subscan_api_key: String,
    pub page_size: usize,
}

impl Config {
    /// 从环境变量加载配置，未设置时使用默认值
    pub fn from_env() -> anyhow::Result<Self> {
        // 如果存在 .env 文件，优先加载
        let _ = dotenv::dotenv();

        let open_square_space = env::var("OPEN_SQUARE_SPACE").unwrap_or_else(|_| "".into());
        let postgres_url = env::var("POSTGRES_URL")?;
        let http_timeout_secs: u64 = env::var("HTTP_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        let snapshot_offset: u64 = env::var("SNAPSHOT_OFFSET")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(50);
        let mnemonic = env::var("MNEMONIC")?;
        let subscan_api_key = env::var("SUBSCAN_API_KEY")?;
        let page_size: usize = env::var("PAGE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(50);

        Ok(Config {
            open_square_space,
            postgres_url,
            http_timeout: Duration::from_secs(http_timeout_secs),
            snapshot_offset,
            mnemonic,
            subscan_api_key,
            page_size,
        })
    }
}
