// src/models.rs

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// SubSquare 公投结构体映射
#[derive(Debug, Deserialize)]
pub struct SubSquareReferendum {
    #[serde(rename = "referendumIndex")]
    pub referendum_index: u32,
    pub title: Option<String>,
    pub content: Option<String>,
    #[serde(rename = "track")]
    pub track_id: u16,
    #[serde(rename = "contentSummary")]
    pub content_summary: Option<ContentSummary>,
}

#[derive(Debug, Deserialize)]
pub struct ContentSummary {
    pub summary: Option<String>,
}

/// networksConfig 里的单个资产配置
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetConfig {
    pub symbol: String,
    pub decimals: u8,
    #[serde(rename = "votingThreshold")]
    pub voting_threshold: String,
    pub multiplier: u32,
}

/// networksConfig 里的单个网络详情
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDetail {
    pub network: String,
    #[serde(rename = "ss58Format")]
    pub ss58_format: u8,
    pub assets: Vec<AssetConfig>,
}

/// 完整的 networksConfig
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworksConfig {
    pub symbol: String,
    pub decimals: u8,
    pub networks: Vec<NetworkDetail>,
    pub strategies: Vec<String>,
    pub version: String,
}

/// data 字段里的完整提案结构
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalData {
    #[serde(rename = "space")]
    pub space: String,

    #[serde(rename = "title")]
    pub title: String,

    #[serde(rename = "content")]
    pub content: String,

    #[serde(rename = "contentType")]
    pub content_type: String,

    #[serde(rename = "choiceType")]
    pub choice_type: String,

    #[serde(rename = "choices")]
    pub choices: Vec<String>,

    #[serde(rename = "startDate")]
    pub start_date: u64,

    #[serde(rename = "endDate")]
    pub end_date: u64,

    #[serde(rename = "snapshotHeights")]
    pub snapshot_heights: HashMap<String, u64>,

    #[serde(rename = "realProposer")]
    pub real_proposer: Option<Value>,

    #[serde(rename = "proposerNetwork")]
    pub proposer_network: String,

    #[serde(rename = "version")]
    pub version: String,

    #[serde(rename = "timestamp")]
    pub timestamp: u64,

    #[serde(rename = "networksConfig")]
    pub networks_config: NetworksConfig,

    pub discussion: Option<String>,
}

/// 最终发送的请求体
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenSquareNewProposalRequest {
    pub data: ProposalData,
    pub address: String,
    pub signature: String,
}

/// Track 枚举及格式化，保持不变
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Track {
    Root = 0,
    WhitelistedCaller = 1,
    WishForChange = 2,
    StakingAdmin = 10,
    Treasurer = 11,
    LeaseAdmin = 12,
    FellowshipAdmin = 13,
    GeneralAdmin = 14,
    AuctionAdmin = 15,
    ReferendumCanceller = 20,
    ReferendumKiller = 21,
    SmallTipper = 30,
    BigTipper = 31,
    SmallSpender = 32,
    MediumSpender = 33,
    BigSpender = 34,
}

impl Track {
    pub fn short_name(&self) -> &str {
        match self {
            Track::Root => "R",
            Track::WhitelistedCaller => "WC",
            Track::WishForChange => "WFC",
            Track::StakingAdmin => "SA",
            Track::Treasurer => "T",
            Track::LeaseAdmin => "LA",
            Track::FellowshipAdmin => "FA",
            Track::GeneralAdmin => "GA",
            Track::AuctionAdmin => "AA",
            Track::ReferendumCanceller => "RC",
            Track::ReferendumKiller => "RK",
            Track::SmallTipper => "ST",
            Track::BigTipper => "BT",
            Track::SmallSpender => "SS",
            Track::MediumSpender => "MS",
            Track::BigSpender => "BS",
        }
    }

    pub fn from_id(id: u16) -> Option<Track> {
        match id {
            0 => Some(Track::Root),
            1 => Some(Track::WhitelistedCaller),
            2 => Some(Track::WishForChange),
            10 => Some(Track::StakingAdmin),
            11 => Some(Track::Treasurer),
            12 => Some(Track::LeaseAdmin),
            13 => Some(Track::FellowshipAdmin),
            14 => Some(Track::GeneralAdmin),
            15 => Some(Track::AuctionAdmin),
            20 => Some(Track::ReferendumCanceller),
            21 => Some(Track::ReferendumKiller),
            30 => Some(Track::SmallTipper),
            31 => Some(Track::BigTipper),
            32 => Some(Track::SmallSpender),
            33 => Some(Track::MediumSpender),
            34 => Some(Track::BigSpender),
            _ => None,
        }
    }


    pub fn format_title(track_id: u16, referendum_index: u32, title_text: &str) -> String {
        let short = Track::from_id(track_id)
            .map(|t| t.short_name().to_string())
            .unwrap_or_else(|| "OT".into());
        format!("[{}] #{} - {}", short, referendum_index, title_text)
    }
}
