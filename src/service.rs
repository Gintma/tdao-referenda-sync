
use anyhow::Result;
use log::{info, error};
use reqwest::Client;
use std::collections::{HashSet, HashMap};
use serde_json::to_string_pretty;
use chrono::{Utc, Datelike, Duration as ChronoDuration, TimeZone};

use sp_core::Pair;
use sp_core::sr25519;
use sp_core::crypto::{Ss58AddressFormat, Ss58AddressFormatRegistry, Ss58Codec};
use hex;

use crate::config::Config;
use crate::db::Db;
use crate::models::{
    SubSquareReferendum,
    ProposalData,
    OpenSquareNewProposalRequest,
    NetworksConfig,
    NetworkDetail,
    AssetConfig,
    Track,
};



/// 拉取 SubSquare 公投列表，数量由配置决定
pub async fn fetch_referenda(client: &Client, page_size: usize) -> Result<Vec<SubSquareReferendum>> {
    let url = format!(
        "https://polkadot-api.subsquare.io/gov2/referendums?page=1&page_size={}&simple=false",
        page_size
    );
    let resp = client.get(&url)
        .send().await?
        .json::<serde_json::Value>().await?;
    let items = serde_json::from_value::<Vec<SubSquareReferendum>>(resp["items"].clone())?;
    Ok(items)
}

/// 获取最新区块高度并应用偏移
pub async fn get_latest_block_height(client: &Client, offset: u64) -> Result<u64> {
    let resp = client
        .post("https://polkadot.api.subscan.io/api/scan/metadata")
        .header("Content-Type", "application/json")
        .header("X-API-Key", &Config::from_env()?.subscan_api_key)
        .body("{}")
        .send().await?
        .json::<serde_json::Value>().await?;

    let block_num_str = resp["data"]["blockNum"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("blockNum not found"))?;
    let height = block_num_str.parse::<u64>()?;
    Ok(height.saturating_sub(offset))
}


/// 核心同步流程：拉取、去重、签名并推送提案
pub async fn run_sync(client: &Client, db: &Db, cfg: &Config) -> Result<()> {
    // 1. 初始化 DB
    db.init_schema().await?;

    // 2. 打印已同步列表
    let existing = db.get_existing_indices().await?;
    info!("📚 当前已同步公投编号（{} 条）：{:?}", existing.len(), existing);

    // 3. 拉取并去重
    let referenda: Vec<SubSquareReferendum> = fetch_referenda(client, cfg.page_size).await?;
    info!("🔍 拉取 {} 条公投数据", referenda.len());
    let mut seen = HashSet::new();
    let unique: Vec<_> = referenda
        .into_iter()
        .filter(|r| seen.insert(r.referendum_index))
        .collect();
    info!("🔎 去重后剩余 {} 条", unique.len());

    // 4. 签名密钥对
    let keypair = sr25519::Pair::from_string(&cfg.mnemonic, None)?;
    // 5. 获取快照高度
    let snapshot = get_latest_block_height(client, cfg.snapshot_offset).await?;
    info!("⛏ 快照块高度：{}", snapshot);

    // 6. 逐条处理
    for r in unique {
        info!("➡️ 开始处理公投 #{}", r.referendum_index);
        if existing.contains(&(r.referendum_index as i32)) {
            info!("↩️ 公投 #{} 已存在，跳过", r.referendum_index);
            continue;
        }

        // 6.1 拼时间戳
        let now = Utc::now();
        let today = Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
            .single()
            .expect("有效的日期时间");
        let start_date = today.timestamp_millis() as u64;
        let end_date = (today + ChronoDuration::days(30)).timestamp_millis() as u64;

        // 6.2 拼标题和内容
        let title_text = r.title.clone().unwrap_or_default();
        let display_title = Track::format_title(r.track_id, r.referendum_index, &title_text);
        let content = format!(
            "https://polkadot.subsquare.io/referenda/{}\n\n{}",
            r.referendum_index,
            r.content_summary
                .as_ref().and_then(|c| c.summary.clone())
                .or_else(|| r.content.clone())
                .unwrap_or_default()
        );

        // 6.3 构造 networksConfig
        let networks_config = NetworksConfig {
            symbol: "DOT".into(),
            decimals: 10,
            networks: vec![
                NetworkDetail {
                    network: "polkadot".into(),
                    ss58_format: 0,
                    assets: vec![
                        AssetConfig {
                            symbol: "DOT".into(),
                            decimals: 10,
                            
                        }
                    ],
                },
            ],
            accessibility: "whitelist".into(),
            whitelist: vec![
                "12mP4sjCfKbDyMRAEyLpkeHeoYtS5USY4x34n9NMwQrcEyoh".to_string(),
                "167rjWHghVwBJ52mz8sNkqr5bKu5vpchbc9CBoieBhVX714h".to_string(),
                "16ap6fdqS2rqFsyYah35hX1FH6rPNWtLqqXZDQC9x6GW141C".to_string(),
                "14pa3BAYZLPvZfRDjWEfZXZWBVU45E67HUQEUxNCrdXGoata".to_string(),
                "14qwyVVvW4Tuhq4Fvt2AHZqhbCtGfVb8HUY2xM2PKrzKsmZT".to_string(),
            ],
            strategies: vec![
                "one-person-one-vote".into(),
            ],
            version: "4".into(),
        };

        // 6.4 构造 snapshotHeights
        let mut snapshot_heights = HashMap::new();
        snapshot_heights.insert("polkadot".into(), snapshot);

        // 6.5 构造 ProposalData
        let data = ProposalData {
            space:            cfg.open_square_space.clone(),
            // title:            display_title.clone(),
            title:            "test-test-test".into(),
            content:          content.clone(),
            content_type:     "markdown".into(),
            choice_type:      "single".into(),
            choices:          vec!["Aye".into(), "Nay".into(), "Abstain".into()],
            start_date,
            end_date,
            snapshot_heights,
            real_proposer:    None,
            proposer_network: "polkadot".into(),
            version:          "5".into(),
            timestamp:        now.timestamp() as u64,
            networks_config,
            discussion:       None,
        };

        // 6.6 签名 & 拼装请求
        let payload = serde_json::to_string(&data)?;
        let sig     = keypair.sign(payload.as_bytes());
        let address = sp_core::sr25519::Public::from_raw(keypair.public().0)
            .to_ss58check_with_version(
                Ss58AddressFormat::from(Ss58AddressFormatRegistry::PolkadotAccount)
            );
        let request = OpenSquareNewProposalRequest {
            data,
            address:   address.clone(),
            signature: format!("0x{}", hex::encode(sig)),
        };

        // 6.7 日志打印
        info!("📨 签名地址: {}", address);
        info!("🔗 请求 URL: https://voting.opensquare.io/api/{}/proposals", cfg.open_square_space);
        info!("📤 请求体: {}", to_string_pretty(&request)?);

        // 6.8 发送
        let res = client
            .post(&format!("https://voting.opensquare.io/api/{}/proposals", cfg.open_square_space))
            .json(&request)
            .send()
            .await?;
        let status = res.status();
        let body   = res.text().await.unwrap_or_default();
        if !status.is_success() {
            error!("❌ 发布失败 #{}：{} - {}", r.referendum_index, status, body);
            continue;
        }
        info!("✅ 发布成功 #{}：{}", r.referendum_index, status);

        // 6.9 插入 DB
        db.insert_referendum(r.referendum_index).await?;
        info!("🗄 已插入数据库 #{}", r.referendum_index);
    }

    Ok(())
}
