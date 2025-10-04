use crate::models::BangumiSubject;
use serde::{Deserialize, Serialize};

// SourceWork已被移除，直接使用AnimeWork

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
pub struct BatchMatchResponse {
    pub matches: Vec<BatchMatchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchMatchResult {
    pub source_index: usize,
    pub matched_bangumi_id: Option<u32>,
    pub confidence: f32,
    pub reasoning: String,
}

// SourceWork已被移除，直接使用AnimeWork

impl From<&BangumiSubject> for CandidateWork {
    fn from(subject: &BangumiSubject) -> Self {
        // 从infobox中提取放映时间和别名
        let air_date = super::utils::extract_air_date_from_subject(subject);
        let aliases = crate::meta_providers::bangumi::extract_aliases_from_infobox(&subject.infobox);

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