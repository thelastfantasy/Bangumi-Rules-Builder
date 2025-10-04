use super::types::{CandidateWork, BatchMatchResponse};
use crate::models::{AnimeWork, AiConfig, AiProvider, AiRequest, AiMessage, AiResponse};
use std::env;


/// 批量匹配多个源作品与候选作品
/// 将多个匹配请求合并为一个AI请求，显著减少API调用次数
pub async fn batch_match_works_with_ai(
    source_works: &[&AnimeWork],
    candidate_works_map: &[&Vec<CandidateWork>],
    ai_config: &AiConfig,
) -> Result<Vec<Option<u32>>, Box<dyn std::error::Error>> {
    if source_works.len() != candidate_works_map.len() {
        return Err("源作品数量和候选作品映射数量不匹配".into());
    }

    let api_key = match ai_config.provider {
        AiProvider::DeepSeek => env::var("DEEPSEEK_API_KEY")?,
    };


    let batch_tasks_content = format_batch_match_tasks(source_works, candidate_works_map);

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
        batch_tasks_content
    );

    let request = AiRequest {
        model: ai_config.model.clone(),
        messages: vec![AiMessage {
            role: "user".to_string(),
            content: prompt,
        }],
    };

    let client = reqwest::Client::new();

    log::debug!("发送AI匹配请求，包含 {} 个任务", source_works.len());

    let response = match client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await {
            Ok(response) => {
                if response.status().is_success() {
                    log::debug!("AI API请求成功，状态码: {}", response.status());
                    response
                } else {
                    log::error!("AI API请求失败，状态码: {}", response.status());
                    return Ok(vec![None; source_works.len()]);
                }
            }
            Err(e) => {
                log::error!("AI API请求网络错误: {}", e);
                return Ok(vec![None; source_works.len()]);
            }
        };

    let api_response: AiResponse = match response.json().await {
        Ok(response) => response,
        Err(e) => {
            log::error!("解析AI API响应失败: {}", e);
            return Ok(vec![None; source_works.len()]);
        }
    };

    if let Some(choice) = api_response.choices.first() {
        let content = choice.message.content.trim();

        // 提取JSON内容，处理markdown代码块
        let json_content = if content.starts_with("```json") && content.ends_with("```") {
            content[7..content.len() - 3].trim()
        } else if content.starts_with("```") && content.ends_with("```") {
            content[3..content.len() - 3].trim()
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


    // 分批处理
    for (batch_index, chunk) in search_tasks.chunks(batch_size).enumerate() {
        if let Some(pb) = progress_bar {
            pb.set_message(format!(
                "处理批次 {}/{} ({}个搜索任务)",
                batch_index + 1,
                search_tasks.len().div_ceil(batch_size),
                chunk.len()
            ));
        }

        match batch_match_works_with_ai(
            &chunk.iter().map(|(source, _)| source).collect::<Vec<_>>(),
            &chunk.iter().map(|(_, candidates)| candidates).collect::<Vec<_>>(),
            ai_config
        ).await {
            Ok(batch_results) => {
                all_results.extend(batch_results);
                log::debug!("成功处理批次 {}，处理了 {} 个任务", batch_index + 1, chunk.len());
            }
            Err(e) => {
                log::error!("处理批次 {} 时发生错误: {}", batch_index + 1, e);
                // 如果AI匹配失败，为这个批次的所有任务返回None
                all_results.extend(vec![None; chunk.len()]);
            }
        }

        // 更新进度条
        if let Some(pb) = progress_bar {
            pb.inc(chunk.len() as u64);
        }

        // 添加延迟以避免API限制
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    Ok(all_results)
}

fn format_batch_match_tasks(source_works: &[&AnimeWork], candidate_works_map: &[&Vec<CandidateWork>]) -> String {
    source_works
        .iter()
        .enumerate()
        .map(|(i, &source_work)| {
            let candidate_works = candidate_works_map[i];
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
    if candidate_works.is_empty() {
        return "无候选作品".to_string();
    }

    candidate_works
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            let aliases_display = if candidate.aliases.is_empty() {
                "无别名".to_string()
            } else {
                candidate.aliases
                    .iter()
                    .map(|alias| format!("『{}』", alias))
                    .collect::<Vec<_>>()
                    .join("、")
            };

            format!(
                "{}. [ID: {}] 日文标题:『{}』 中文标题:『{}』 放映时间:『{}』 别名: {}",
                i + 1,
                candidate.bangumi_id,
                candidate.japanese_title,
                candidate.chinese_title,
                candidate.air_date.as_deref().unwrap_or("未知"),
                aliases_display
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// 单个作品匹配函数 - 保留用于特殊情况
#[allow(dead_code)]
pub async fn match_works_with_ai(
    source_work: &AnimeWork,
    candidate_works: &[CandidateWork],
    ai_config: &AiConfig,
) -> Result<Option<u32>, Box<dyn std::error::Error>> {
    let candidate_works_vec = candidate_works.to_vec();
    let results = batch_match_works_with_ai(&[source_work], &[&candidate_works_vec], ai_config).await?;
    Ok(results.first().copied().flatten())
}