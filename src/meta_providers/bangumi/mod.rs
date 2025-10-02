use crate::models::{AnimeWork, BangumiResult, BangumiSubject, BangumiInfoboxItem, AiConfig};
use crate::ai::object_matcher::{SourceWork, CandidateWork, match_works_with_ai};
use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone};
use indicatif::{ProgressBar, ProgressStyle};

pub async fn search_bangumi_for_works(
    works: &[AnimeWork],
) -> Result<Vec<BangumiResult>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut results = Vec::new();

    // 创建进度条
    let total_works = works.len();
    let pb = ProgressBar::new(total_works as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.yellow} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}"
        )
        .unwrap()
        .progress_chars("█▓▒░")
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(250));
    pb.set_message("Bangumi搜索中...");

    for (index, work) in works.iter().enumerate() {
        pb.set_message(format!(
            "搜索作品: {} ({}/{})",
            work.cleaned_title,
            index + 1,
            total_works
        ));

        let mut found = false;

        // 构建搜索关键词数组：包含cleaned_title和keywords，并去重
        let mut search_keywords: Vec<&str> = Vec::new();
        search_keywords.push(&work.cleaned_title);
        search_keywords.extend(work.keywords.iter().map(|s| s.as_str()));

        // 去重
        search_keywords.sort();
        search_keywords.dedup();

        // 尝试每个关键字
        for keyword in search_keywords {
            if let Some(subject) =
                search_bangumi_with_keyword(&client, keyword, &work.air_date).await?
            {
                let chinese_name = if !subject.name_cn.is_empty() {
                    Some(subject.name_cn.clone())
                } else {
                    None
                };

                let aliases = extract_aliases_from_infobox(&subject.infobox);

                results.push(BangumiResult {
                    original_title: work.original_title.clone(),
                    cleaned_title: work.cleaned_title.clone(),
                    bangumi_id: Some(subject.id),
                    chinese_name,
                    aliases,
                    air_date: work.air_date,
                    keywords: work.keywords.clone(),
                });

                found = true;
                break;
            }
        }

        if !found {
            results.push(BangumiResult {
                original_title: work.original_title.clone(),
                cleaned_title: work.cleaned_title.clone(),
                bangumi_id: None,
                chinese_name: None,
                aliases: Vec::new(),
                air_date: work.air_date,
                keywords: work.keywords.clone(),
            });
        }

        // 更新进度条
        pb.inc(1);
    }

    // 完成进度条
    pb.finish_with_message("Bangumi搜索完成");

    Ok(results)
}

pub async fn search_bangumi_with_keyword(
    client: &reqwest::Client,
    keyword: &str,
    air_date: &Option<NaiveDate>,
) -> Result<Option<BangumiSubject>, Box<dyn std::error::Error>> {
    let url = "https://api.bgm.tv/v0/search/subjects";

    // 构建日期范围查询
    let date_range = build_air_date_filter(air_date);

    // 构建POST请求体
    let mut request_body = serde_json::json!({
        "keyword": keyword,
        "sort": "rank",
        "filter": {
            "type": [2]  // 只搜索动画
        }
    });

    // 如果有日期范围，添加到过滤器中
    if let Some(ref date_filter) = date_range {
        request_body["filter"]["air_date"] = date_filter.clone();
    }

    // 特别调试：检查是否在搜索问题作品
    let problem_keywords = vec![
        "破産富豪",
        "ある日、お姫様になってしまった件について",
        "羅小黒戦記",
        "MUZIK TIGER In the Forest 第2期",
    ];

    let is_problem_work = problem_keywords.iter().any(|k| keyword.contains(k));

    if is_problem_work {
        println!("\n🔍 调试：正在搜索问题作品的关键字: '{}'", keyword);
        println!("   日期过滤器: {:?}", date_range);
        println!(
            "   Bangumi API 请求体: {}",
            serde_json::to_string_pretty(&request_body).unwrap()
        );
    }

    let response = client
        .post(url)
        .header("User-Agent", "smart_bangumi_qb_rule_generator/0.1.0")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if is_problem_work {
        println!("   Bangumi API 响应状态: {}", response.status());
    }

    if response.status().is_success() {
        let json_response: serde_json::Value = response.json().await?;

        // 调试输出搜索结果（仅针对问题作品）
        if is_problem_work {
            if let Some(data_array) = json_response["data"].as_array() {
                println!("   找到 {} 个搜索结果", data_array.len());
                if !data_array.is_empty() {
                    println!(
                        "   第一个结果: {}",
                        serde_json::to_string_pretty(&data_array[0]).unwrap()
                    );
                }
            }
        }

        if let Some(data_array) = json_response["data"].as_array() {
            // 将搜索结果转换为候选作品
            let candidate_works: Vec<CandidateWork> = data_array
                .iter()
                .filter_map(|subject_data| {
                    serde_json::from_value::<BangumiSubject>(subject_data.clone()).ok()
                })
                .map(|subject| CandidateWork::from(&subject))
                .collect();

            if candidate_works.is_empty() {
                if is_problem_work {
                    println!("🔍 调试：没有有效的候选作品");
                }
                return Ok(None);
            }

            // 创建源作品
            let source_work = SourceWork {
                original_title: keyword.to_string(),
                cleaned_title: keyword.to_string(),
                air_date: air_date.map(|d| d.to_string()),
                keywords: vec![keyword.to_string()],
            };

            // 使用AI进行匹配
            let ai_config = AiConfig::deepseek();
            if let Ok(matched_id) = match_works_with_ai(&source_work, &candidate_works, &ai_config).await {
                if let Some(bangumi_id) = matched_id {
                    // 找到匹配的作品
                    if is_problem_work {
                        println!("🔍 调试：AI匹配成功，作品ID: {}", bangumi_id);
                    }

                    // 返回匹配的BangumiSubject
                    for subject_data in data_array {
                        if let Ok(subject) = serde_json::from_value::<BangumiSubject>(subject_data.clone()) {
                            if subject.id == bangumi_id {
                                return Ok(Some(subject));
                            }
                        }
                    }
                } else if is_problem_work {
                    println!("🔍 调试：AI未找到匹配作品");
                }
            } else if is_problem_work {
                println!("🔍 调试：AI匹配失败");
            }
        }
    }

    if is_problem_work {
        println!("🔍 调试：未找到匹配结果");
    }

    Ok(None)
}

fn build_air_date_filter(air_date: &Option<NaiveDate>) -> Option<serde_json::Value> {
    // 根据放送时间构建日期范围过滤器
    if let Some(date) = air_date {
        // 将NaiveDate转换为JST时区，确保日期范围正确
        let jst_date = convert_to_jst_date(*date);

        // 对于具体日期，搜索前后1个月的范围
        let start_date = jst_date - chrono::Duration::days(30);
        let end_date = jst_date + chrono::Duration::days(30);

        return Some(serde_json::json!([
            format!(">={}", start_date.format("%Y-%m-%d")),
            format!("<{}", end_date.format("%Y-%m-%d"))
        ]));
    }

    None
}

fn convert_to_jst_date(naive_date: NaiveDate) -> DateTime<FixedOffset> {
    // 日本标准时间 (JST) 是 UTC+9
    let jst_offset = FixedOffset::east_opt(9 * 3600).unwrap();
    jst_offset
        .from_local_datetime(&naive_date.and_hms_opt(0, 0, 0).unwrap())
        .unwrap()
}


pub fn extract_aliases_from_infobox(infobox: &[BangumiInfoboxItem]) -> Vec<String> {
    let mut aliases = Vec::new();

    for item in infobox {
        if item.key == "别名" || item.key == "中文名" || item.key == "译名" {
            match &item.value {
                serde_json::Value::String(s) => {
                    aliases.push(s.clone());
                }
                serde_json::Value::Array(arr) => {
                    for val in arr {
                        // Handle both string values and object values with "v" key
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