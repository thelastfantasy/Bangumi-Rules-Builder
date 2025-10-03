use crate::models::{AiConfig, AiProvider, AiRequest, AiMessage, AiResponse, TableInfo, AnimeWork, Statistics};
use std::env;
use indicatif::{ProgressBar, ProgressStyle};

pub async fn match_and_process_with_ai<'a>(
    description: &'a str,
    tables: &'a [TableInfo],
    ai_config: &AiConfig,
) -> Result<
    (
        Option<(&'a TableInfo, Vec<AnimeWork>)>,
        Vec<AnimeWork>,
        Statistics,
    ),
    Box<dyn std::error::Error>,
> {
    let api_key = match ai_config.provider {
        AiProvider::DeepSeek => env::var("DEEPSEEK_API_KEY")?,
        // 未来支持其他AI提供商
        // AiProvider::OpenAi => env::var("OPENAI_API_KEY")?,
        // AiProvider::Claude => env::var("CLAUDE_API_KEY")?,
    };

    let mut stats = Statistics {
        total_works_from_table: 0,
        works_with_undetermined_date: 0,
        works_processed_by_ai: 0,
        works_with_bangumi_info: 0,
        works_without_bangumi_info: 0,
        qb_rules_generated: 0,
        qb_rules_failed: 0,
        ai_requests_count: 0,
        ai_input_tokens: 0,
        ai_output_tokens: 0,
    };

    // 准备表格信息 - 只发送表格标题作为锚点
    let table_descriptions: Vec<String> = tables
        .iter()
        .map(|table| format!("标题: {}", table.title))
        .collect();

    // 步骤1: 先让AI选择表格
    let table_selection_prompt = format!(
        "用户提供的描述是: '{}'\n\n
        以下是网页中找到的表格标题:\n{}
        \n\n请根据用户描述，判断哪个表格标题最相关。请返回表格的序号（从0开始）。\n\n
        请返回JSON格式：{{\"table_index\": 数字}}",
        description,
        table_descriptions
            .iter()
            .enumerate()
            .map(|(i, desc)| format!("[{}] {}", i, desc))
            .collect::<Vec<_>>()
            .join("\n\n")
    );

    println!("发送给AI的表格选择提示: {}", table_selection_prompt);

    let table_selection_request = AiRequest {
        model: ai_config.model.clone(),
        messages: vec![AiMessage {
            role: "user".to_string(),
            content: table_selection_prompt,
        }],
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()?;
    let response = client
        .post(&ai_config.api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&table_selection_request)
        .send()
        .await?;

    let api_response: AiResponse = response.json().await?;

    // 记录第一次AI请求的token使用情况
    stats.ai_requests_count += 1;
    if let Some(usage) = &api_response.usage {
        stats.ai_input_tokens += usage.prompt_tokens;
        stats.ai_output_tokens += usage.completion_tokens;
    }

    let mut selected_table_index = 0;
    if let Some(choice) = api_response.choices.first() {
        let content = choice.message.content.trim();
        println!("AI表格选择返回内容: {}", content);

        // 提取JSON内容，处理markdown代码块
        let json_content = if content.starts_with("```json") && content.ends_with("```") {
            content[7..content.len() - 3].trim()
        } else if content.starts_with("```") && content.ends_with("```") {
            content[3..content.len() - 3].trim()
        } else {
            content
        };

        if let Ok(processed_data) = serde_json::from_str::<serde_json::Value>(json_content)
            && let Some(table_index) = processed_data["table_index"].as_u64()
        {
            selected_table_index = table_index as usize;
        }
    }

    if selected_table_index >= tables.len() {
        selected_table_index = 0;
    }

    let matched_table = &tables[selected_table_index];
    println!("选择的表格标题: {}", matched_table.title);

    // 步骤2: 解析表格获取实际作品
    let raw_works = crate::sites::kansou::parse_table_works(&matched_table.table_html)?;
    stats.total_works_from_table = raw_works.len();
    println!("从表格中解析出 {} 个作品", raw_works.len());

    // 步骤3: 将实际作品分批发送给AI进行清理和关键字生成
    let batch_size = 20; // 每批处理20个作品
    let mut processed_works = Vec::new();

    // 创建进度条 - 在整个AI处理过程中共享
    let total_works = raw_works.len();
    let pb = ProgressBar::new(total_works as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}"
        )
        .unwrap()
        .progress_chars("█▓▒░")
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(250));
    pb.set_message("AI处理中...");

    for (batch_index, batch) in raw_works.chunks(batch_size).enumerate() {
        let current_batch_start = batch_index * batch_size;
        let current_batch_end = (batch_index + 1) * batch_size;

        pb.set_message(format!(
            "处理第 {} 批作品 ({}-{}/{})",
            batch_index + 1,
            current_batch_start + 1,
            current_batch_end.min(total_works),
            total_works
        ));

        let works_for_processing: Vec<String> = batch
            .iter()
            .map(|work| {
                format!(
                    "原标题: {}, 放送日期: {:?}",
                    work.original_title, work.air_date
                )
            })
            .collect();

        let works_processing_prompt = format!(
            "以下是需要处理的动画作品列表：\n\n{}
            \n\n请为每个作品执行以下操作：\n1. 清理标题，去除无用信息如【日本語吹替版】等，但保留季号信息和副标题\n   - 重要：副标题如『』、【】、（）、《》、「」中的内容都是重要信息，必须保留\n   - 例如：'青のミブロ 第二期「芹沢暗殺編」' 中的 '「芹沢暗殺編」' 必须保留\n2. 生成5-8个搜索关键字 - 请包含：\n   - 日文原标题（包含中点・和空格变体）\n   - 常见中文译名\n   - 英文名称\n   - 其他常见搜索变体\n   特别提醒：\n   - 主标题和副标题同等重要，至少有一个关键字必须同时包含主副标题（用半角空格分割）\n   - 对于包含中点的日文标题，请同时生成去掉中点用空格替代的版本\n   - 对于可能使用特殊符号（如♥、☆等）的经典作品，请生成包含这些符号变体的关键字\n   - 对于经典作品的重制/新作，请包含原版作品的各种常见名称变体\n   - 对于经典作品如'キャッツ・アイ'，请包含'キャッツ アイ'（无中点）和'猫眼三姐妹'等常见中文译名\n   - 生成的关键字中尽量不要带特殊符号（如♥、☆等），以免干扰搜索结果\n   - 如果标题中有起到分隔作用的特殊符号（如日文中点・、爱心♥、星星☆等），在生成关键字时应替换成半角空格\n\n
            请返回JSON格式：{{\"works\": [{{\"original_title\": \"原标题\", \"cleaned_title\": \"清理后标题\", \"keywords\": [\"关键词1\", \"关键词2\"]}}, ...]}}",
            works_for_processing.join("\n")
        );

        let works_processing_request = AiRequest {
            model: ai_config.model.clone(),
            messages: vec![AiMessage {
                role: "user".to_string(),
                content: works_processing_prompt,
            }],
        };

        let response = client
            .post(&ai_config.api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&works_processing_request)
            .send()
            .await?;

        let api_response: AiResponse = response.json().await?;

        // 记录AI请求的token使用情况
        stats.ai_requests_count += 1;
        if let Some(usage) = &api_response.usage {
            stats.ai_input_tokens += usage.prompt_tokens;
            stats.ai_output_tokens += usage.completion_tokens;
        }

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

            // 尝试从JSON中提取处理后的信息
            if let Ok(processed_data) = serde_json::from_str::<serde_json::Value>(json_content)
                && let Some(works_array) = processed_data["works"].as_array()
            {
                for (i, work_data) in works_array.iter().enumerate() {
                    let batch_offset = batch_index * batch_size + i;
                    if batch_offset < raw_works.len()
                        && let (
                            Some(original_title),
                            Some(cleaned_title),
                            Some(keywords_array),
                        ) = (
                            work_data["original_title"].as_str(),
                            work_data["cleaned_title"].as_str(),
                            work_data["keywords"].as_array(),
                        )
                    {
                        let keywords: Vec<String> = keywords_array
                            .iter()
                            .filter_map(|k| k.as_str().map(|s| s.to_string()))
                            .collect();

                        // 保持原有的air_date
                        let original_work = &raw_works[batch_offset];
                        processed_works.push(AnimeWork {
                            original_title: original_title.to_string(),
                            cleaned_title: cleaned_title.to_string(),
                            air_date: original_work.air_date,
                            keywords,
                        });
                    }
                }
            }
        }

        // 更新进度条
        pb.inc(batch.len() as u64);

        // 短暂延迟避免API限制
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    // 完成进度条
    pb.finish_with_message("AI处理完成");

    // 如果AI处理失败，使用原始作品
    if processed_works.is_empty() {
        println!("AI作品处理失败，使用原始作品");
        processed_works = raw_works.clone();
    }

    stats.works_processed_by_ai = processed_works.len();
    let processed_works_clone = processed_works.clone();
    Ok((
        Some((matched_table, processed_works)),
        processed_works_clone,
        stats,
    ))
}