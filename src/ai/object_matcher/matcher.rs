use super::types::{AiMatchRequest, AiMatchResponse, CandidateWork, BatchMatchRequest, BatchMatchResponse};
use crate::models::{AnimeWork, AiConfig, AiProvider, AiRequest, AiMessage, AiResponse};
use std::env;

pub async fn match_works_with_ai(
    source_work: &AnimeWork,
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
        source_work.air_date.map(|d| d.to_string()).as_deref().unwrap_or("未知"),
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

/// 批量匹配多个源作品与候选作品
/// 将多个匹配请求合并为一个AI请求，显著减少API调用次数
pub async fn batch_match_works_with_ai(
    source_works: &[AnimeWork],
    candidate_works_map: &[Vec<CandidateWork>],
    ai_config: &AiConfig,
) -> Result<Vec<Option<u32>>, Box<dyn std::error::Error>> {
    if source_works.len() != candidate_works_map.len() {
        return Err("源作品数量和候选作品映射数量不匹配".into());
    }

    let api_key = match ai_config.provider {
        AiProvider::DeepSeek => env::var("DEEPSEEK_API_KEY")?,
    };

    let _batch_request = BatchMatchRequest {
        source_works: source_works.to_vec(),
        candidate_works_map: candidate_works_map.to_vec(),
    };

    let prompt = format!(
        r#"请为以下多个独立的匹配任务找到最合适的Bangumi作品。每个任务都是完全独立的，请不要混淆不同任务之间的信息。

重要规则：
1. 每个任务都是独立的，只考虑该任务内的源作品和候选作品
2. 不要将任务A的关键词与任务B的候选作品匹配
3. 每个任务必须单独评估，互不影响

任务列表：
{}

匹配标准（对每个任务独立应用）：
- 标题语义相似性（包括特殊符号、季度表示差异）
- 放映时间的接近程度（前后30天内）
- 关键词与候选作品标题/别名的匹配度
- 是否为同一作品的不同季度

返回格式要求：
- 必须为每个任务返回一个结果，即使没有匹配也要返回null
- confidence必须基于该任务内的信息独立计算
- reasoning必须说明为什么选择这个匹配（或为什么不匹配）

请返回JSON格式：
{{
  "matches": [
    {{"source_index": 0, "matched_bangumi_id": <ID或null>, "confidence": <0-1的置信度>, "reasoning": "匹配理由"}},
    {{"source_index": 1, "matched_bangumi_id": <ID或null>, "confidence": <0-1的置信度>, "reasoning": "匹配理由"}},
    // ... 确保每个任务都有对应的结果
  ]
}}

注意：如果没有高度匹配（confidence > 0.7），请返回null。"#,
        format_batch_match_tasks(source_works, candidate_works_map)
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

        if let Ok(batch_response) = serde_json::from_str::<BatchMatchResponse>(json_content) {
            // 将匹配结果按源作品索引排序
            let mut results = vec![None; source_works.len()];
            for match_result in batch_response.matches {
                if match_result.source_index < source_works.len() && match_result.confidence > 0.7 {
                    results[match_result.source_index] = match_result.matched_bangumi_id;
                }
            }
            return Ok(results);
        }
    }

    // 如果解析失败，返回所有None
    Ok(vec![None; source_works.len()])
}

/// 批量处理多个搜索任务，自动分批处理以避免token超限
pub async fn batch_process_searches(
    search_tasks: &[(AnimeWork, Vec<CandidateWork>)],
    ai_config: &AiConfig,
    batch_size: usize,
    progress_bar: Option<&indicatif::ProgressBar>,
) -> Result<Vec<Option<u32>>, Box<dyn std::error::Error>> {
    let mut all_results = Vec::new();

    // 输出开始智能匹配的文本
    println!("🚀 开始智能匹配，共 {} 个搜索任务，分批大小: {}", search_tasks.len(), batch_size);

    // 分批处理
    for (batch_index, chunk) in search_tasks.chunks(batch_size).enumerate() {
        if let Some(pb) = progress_bar {
            pb.set_message(format!(
                "处理批次 {}/{} ({}个搜索任务)",
                batch_index + 1,
                (search_tasks.len() + batch_size - 1) / batch_size,
                chunk.len()
            ));
        }

        let source_works: Vec<AnimeWork> = chunk.iter().map(|(source, _)| source.clone()).collect();
        let candidate_works_map: Vec<Vec<CandidateWork>> = chunk.iter().map(|(_, candidates)| candidates.clone()).collect();

        let batch_results = batch_match_works_with_ai(&source_works, &candidate_works_map, ai_config).await?;
        all_results.extend(batch_results);

        // 更新进度条
        if let Some(pb) = progress_bar {
            pb.inc(chunk.len() as u64);
        }

        // 添加延迟以避免API限制
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    Ok(all_results)
}

fn format_batch_match_tasks(source_works: &[AnimeWork], candidate_works_map: &[Vec<CandidateWork>]) -> String {
    source_works
        .iter()
        .enumerate()
        .map(|(i, source_work)| {
            let candidate_works = &candidate_works_map[i];
            format!(
                "\n=== 任务 {} ===\n[源作品信息]\n- 原标题: {}\n- 清理标题: {}\n- 放映时间: {}\n- 关键词: {:?}\n\n[候选作品列表]\n{}\n=== 任务 {} 结束 ===",
                i,
                source_work.original_title,
                source_work.cleaned_title,
                source_work.air_date.map(|d| d.to_string()).as_deref().unwrap_or("未知"),
                source_work.keywords,
                format_candidate_works(candidate_works),
                i
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
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