use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SiteType {
    Kansou,
    // 预留未来支持的站点
    // ModelScope,
    // AnimeList,
}

impl std::fmt::Display for SiteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SiteType::Kansou => write!(f, "kansou"),
            // SiteType::ModelScope => write!(f, "modelscope"),
            // SiteType::AnimeList => write!(f, "animelist"),
        }
    }
}

impl std::str::FromStr for SiteType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "kansou" => Ok(SiteType::Kansou),
            // "modelscope" => Ok(SiteType::ModelScope),
            // "animelist" => Ok(SiteType::AnimeList),
            _ => Err(format!("不支持的站点类型: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AiProvider {
    DeepSeek,
    // 预留未来支持的AI提供商
    // OpenAi,
    // Claude,
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::DeepSeek => write!(f, "deepseek"),
            // AiProvider::OpenAi => write!(f, "openai"),
            // AiProvider::Claude => write!(f, "claude"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Task {
    pub description: String,
    pub site: SiteType,
    pub root_path: String,
}

#[derive(Debug)]
pub struct TableInfo {
    pub title: String,
    pub table_html: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnimeWork {
    pub original_title: String,
    pub cleaned_title: String,
    pub air_date: Option<NaiveDate>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiResult {
    pub original_title: String,
    pub cleaned_title: String,
    pub bangumi_id: Option<u32>,
    pub chinese_name: Option<String>,
    pub aliases: Vec<String>,
    pub air_date: Option<NaiveDate>,
    pub keywords: Vec<String>,
}

#[derive(Debug)]
pub struct Statistics {
    pub total_works_from_table: usize,
    pub works_with_undetermined_date: usize,
    pub works_processed_by_ai: usize,
    pub works_with_bangumi_info: usize,
    pub works_without_bangumi_info: usize,
    pub qb_rules_generated: usize,
    pub qb_rules_failed: usize,
    pub ai_requests_count: usize,
    pub ai_input_tokens: usize,
    pub ai_output_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct AiConfig {
    pub provider: AiProvider,
    pub model: String,
    pub api_url: String,
}

impl AiConfig {
    pub fn deepseek() -> Self {
        Self {
            provider: AiProvider::DeepSeek,
            model: "deepseek-chat".to_string(),
            api_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
        }
    }

    // 预留其他AI提供商的配置
    // pub fn openai() -> Self {
    //     Self {
    //         provider: AiProvider::OpenAi,
    //         model: "gpt-4".to_string(),
    //         api_url: "https://api.openai.com/v1/chat/completions".to_string(),
    //     }
    // }

    // pub fn claude() -> Self {
    //     Self {
    //         provider: AiProvider::Claude,
    //         model: "claude-3-sonnet".to_string(),
    //         api_url: "https://api.anthropic.com/v1/messages".to_string(),
    //     }
    // }
}

#[derive(Serialize)]
pub struct AiRequest {
    pub model: String,
    pub messages: Vec<AiMessage>,
}

#[derive(Serialize)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct AiResponse {
    pub choices: Vec<AiChoice>,
    pub usage: Option<AiUsage>,
}

#[derive(Debug, Deserialize)]
pub struct AiUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
}

#[derive(Debug, Deserialize)]
pub struct AiChoice {
    pub message: AiChoiceMessage,
}

#[derive(Debug, Deserialize)]
pub struct AiChoiceMessage {
    pub content: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
pub struct BangumiSearchRequest {
    pub keyword: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub air_date: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct BangumiSubject {
    pub id: u32,
    pub name: String,
    pub name_cn: String,
    #[serde(default)]
    pub infobox: Vec<BangumiInfoboxItem>,
}

#[derive(Debug, Deserialize)]
pub struct BangumiInfoboxItem {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QBRule {
    #[serde(rename = "addPaused")]
    pub add_paused: Option<serde_json::Value>,
    #[serde(rename = "affectedFeeds")]
    pub affected_feeds: Vec<String>,
    #[serde(rename = "assignedCategory")]
    pub assigned_category: String,
    pub enabled: bool,
    #[serde(rename = "episodeFilter")]
    pub episode_filter: String,
    #[serde(rename = "ignoreDays")]
    pub ignore_days: u32,
    #[serde(rename = "lastMatch")]
    pub last_match: String,
    #[serde(rename = "mustContain")]
    pub must_contain: String,
    #[serde(rename = "mustNotContain")]
    pub must_not_contain: String,
    #[serde(rename = "previouslyMatchedEpisodes")]
    pub previously_matched_episodes: Vec<String>,
    pub priority: i32,
    #[serde(rename = "savePath")]
    pub save_path: String,
    #[serde(rename = "smartFilter")]
    pub smart_filter: bool,
    #[serde(rename = "torrentContentLayout")]
    pub torrent_content_layout: Option<serde_json::Value>,
    #[serde(rename = "torrentParams")]
    pub torrent_params: TorrentParams,
    #[serde(rename = "useRegex")]
    pub use_regex: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TorrentParams {
    pub category: String,
    pub download_limit: i32,
    pub download_path: String,
    pub inactive_seeding_time_limit: i32,
    pub operating_mode: String,
    pub ratio_limit: i32,
    pub save_path: String,
    pub seeding_time_limit: i32,
    pub share_limit_action: String,
    pub skip_checking: bool,
    pub ssl_certificate: String,
    pub ssl_dh_params: String,
    pub ssl_private_key: String,
    pub tags: Vec<String>,
    pub upload_limit: i32,
    pub use_auto_tmm: bool,
}

// 用于跟踪规则生成结果
pub struct RuleGenerationResult {
    pub rules: serde_json::Value,
    pub failed_works: Vec<(String, String)>, // (作品名称, 失败原因)
}