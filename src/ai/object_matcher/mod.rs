use crate::models::{AnimeWork, BangumiSubject};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceWork {
    pub original_title: String,
    pub cleaned_title: String,
    pub air_date: Option<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CandidateWork {
    pub bangumi_id: u32,
    pub japanese_title: String,
    pub chinese_title: String,
    pub aliases: Vec<String>,
    pub air_date: Option<String>,
    pub score: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiMatchRequest {
    pub source_work: SourceWork,
    pub candidate_works: Vec<CandidateWork>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiMatchResponse {
    pub matched_bangumi_id: Option<u32>,
    pub confidence: f32,
    pub reasoning: String,
}

impl From<&AnimeWork> for SourceWork {
    fn from(work: &AnimeWork) -> Self {
        SourceWork {
            original_title: work.original_title.clone(),
            cleaned_title: work.cleaned_title.clone(),
            air_date: work.air_date.map(|d| d.to_string()),
            keywords: work.keywords.clone(),
        }
    }
}

impl From<&BangumiSubject> for CandidateWork {
    fn from(subject: &BangumiSubject) -> Self {
        // 从infobox中提取放映时间和别名
        let air_date = extract_air_date_from_infobox(&subject.infobox);
        let aliases = extract_aliases_from_infobox(&subject.infobox);

        CandidateWork {
            bangumi_id: subject.id,
            japanese_title: subject.name.clone(),
            chinese_title: subject.name_cn.clone(),
            aliases,
            air_date: air_date.map(|d| d.to_string()),
            score: None,
        }
    }
}

fn extract_air_date_from_infobox(infobox: &[crate::models::BangumiInfoboxItem]) -> Option<chrono::NaiveDate> {
    for item in infobox {
        if item.key == "放送开始" || item.key == "开始" {
            match &item.value {
                serde_json::Value::String(date_str) => {
                    // 尝试解析日期格式
                    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                        return Some(date);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

fn extract_aliases_from_infobox(infobox: &[crate::models::BangumiInfoboxItem]) -> Vec<String> {
    let mut aliases = Vec::new();

    for item in infobox {
        if item.key == "别名" || item.key == "中文名" || item.key == "译名" {
            match &item.value {
                serde_json::Value::String(s) => {
                    aliases.push(s.clone());
                }
                serde_json::Value::Array(arr) => {
                    for val in arr {
                        match val {
                            serde_json::Value::String(s) => {
                                aliases.push(s.clone());
                            }
                            serde_json::Value::Object(obj) => {
                                if let Some(serde_json::Value::String(s)) = obj.get("v") {
                                    aliases.push(s.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    aliases
}

pub mod matcher;
pub use matcher::*;