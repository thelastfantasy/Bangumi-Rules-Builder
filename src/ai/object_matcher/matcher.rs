use super::types::{AiMatchRequest, AiMatchResponse, SourceWork, CandidateWork};
use crate::models::{AiConfig, AiProvider, AiRequest, AiMessage, AiResponse};
use std::env;

pub async fn match_works_with_ai(
    source_work: &SourceWork,
    candidate_works: &[CandidateWork],
    ai_config: &AiConfig,
) -> Result<Option<u32>, Box<dyn std::error::Error>> {
    let api_key = match ai_config.provider {
        AiProvider::DeepSeek => env::var("DEEPSEEK_API_KEY")?,
    };

    let _match_request = AiMatchRequest {
        source_work: source_work.clone(),
        candidate_works: candidate_works.to_vec(),
    };

    let prompt = format!(
        r#"请从以下候选作品中找到与源作品最匹配的项：

源作品：
- 原标题: {}
- 清理标题: {}
- 放映时间: {}
- 关键词: {:?}

候选作品：
{}

请考虑：
- 标题语义相似性（包括特殊符号、季度表示差异）
- 放映时间的接近程度
- 关键词与候选作品标题/别名的匹配度
- 是否为同一作品的不同季度

请返回JSON格式：{{"matched_bangumi_id": <ID或null>, "confidence": <0-1的置信度>, "reasoning": "匹配理由"}}

如果没有完美匹配则返回null。"#,
        source_work.original_title,
        source_work.cleaned_title,
        source_work.air_date.as_deref().unwrap_or("未知"),
        source_work.keywords,
        format_candidate_works(candidate_works)
    );

    let request = AiRequest {
        model: ai_config.model.clone(),
        messages: vec![AiMessage {
            role: "user".to_string(),
            content: prompt,
        }],
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let api_response: AiResponse = response.json().await?;

    if let Some(choice) = api_response.choices.first() {
        let content = choice.message.content.trim();

        // 提取JSON内容，处理markdown代码块
        let json_content = if content.starts_with("```json") && content.ends_with("```") {
            &content[7..content.len() - 3].trim()
        } else if content.starts_with("```") && content.ends_with("```") {
            &content[3..content.len() - 3].trim()
        } else {
            content
        };

        if let Ok(match_response) = serde_json::from_str::<AiMatchResponse>(json_content) {
            // 如果置信度高于0.7，认为匹配成功
            if match_response.confidence > 0.7 {
                return Ok(match_response.matched_bangumi_id);
            }
        }
    }

    Ok(None)
}

fn format_candidate_works(candidate_works: &[CandidateWork]) -> String {
    candidate_works
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            format!(
                "{}. [ID: {}] {} (中文: {}) (放映时间: {}) (别名: {:?})",
                i + 1,
                candidate.bangumi_id,
                candidate.japanese_title,
                candidate.chinese_title,
                candidate.air_date.as_deref().unwrap_or("未知"),
                candidate.aliases
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}