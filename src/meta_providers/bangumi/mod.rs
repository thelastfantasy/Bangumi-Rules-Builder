use crate::models::{AnimeWork, BangumiResult, BangumiSubject, BangumiInfoboxItem};
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
            let mut best_match: Option<BangumiSubject> = None;
            let mut best_score = 0.0;

            for subject_data in data_array {
                if let Ok(subject) = serde_json::from_value::<BangumiSubject>(subject_data.clone()) {
                    let mut score = 0.0;

                    // 1. 检查日文名称匹配（权重最高）
                    if is_title_matching(&subject.name, keyword) {
                        score += 0.5;
                    }

                    // 2. 检查中文名称匹配
                    if !subject.name_cn.is_empty() && is_title_matching(&subject.name_cn, keyword) {
                        score += 0.3;
                    }

                    // 3. 检查别名匹配
                    let aliases = extract_aliases_from_infobox(&subject.infobox);
                    for alias in &aliases {
                        if is_title_matching(alias, keyword) {
                            score += 0.2;
                            break; // 只加一次分
                        }
                    }

                    // 特别调试：输出问题作品的详细评分（在移动subject之前）
                    if is_problem_work && score > 0.0 {
                        println!("🔍 调试：匹配评分详情");
                        println!("   搜索关键字: '{}'", keyword);
                        println!("   作品ID: {}", subject.id);
                        println!("   作品名称: '{}'", subject.name);
                        println!("   中文名称: '{}'", subject.name_cn);
                        println!("   最终评分: {}", score);
                    }

                    // 如果分数高于当前最佳匹配，更新最佳匹配
                    if score > best_score {
                        best_score = score;
                        best_match = Some(subject);
                    }
                }
            }

            // 只有当匹配分数达到阈值时才返回结果
            if best_score >= 0.5 {
                if is_problem_work {
                    println!("🔍 调试：匹配成功，最佳评分: {}", best_score);
                }
                return Ok(best_match);
            } else if is_problem_work {
                println!("🔍 调试：匹配失败，最佳评分: {} (未达到阈值0.5)", best_score);
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

fn is_title_matching(bangumi_title: &str, search_keyword: &str) -> bool {
    // 改进的名称匹配逻辑
    let title_lower = bangumi_title.to_lowercase();
    let keyword_lower = search_keyword.to_lowercase();

    // 1. 如果标题完全包含关键词，认为是强匹配
    if title_lower.contains(&keyword_lower) {
        return true;
    }

    // 2. 如果关键词完全包含标题，也认为是匹配
    if keyword_lower.contains(&title_lower) {
        return true;
    }

    // 3. 对于较长的关键词，检查是否有显著的重叠部分
    if keyword_lower.len() > 5 {
        // 计算最长公共子串长度
        let common_length = longest_common_substring(&title_lower, &keyword_lower);
        let min_length = std::cmp::min(title_lower.len(), keyword_lower.len());

        // 如果公共子串长度超过较短字符串的60%，认为是匹配
        if common_length as f32 / min_length as f32 > 0.6 {
            return true;
        }
    }

    false
}

fn longest_common_substring(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let mut max_len = 0;

    for i in 0..s1_chars.len() {
        for j in 0..s2_chars.len() {
            let mut k = 0;
            while i + k < s1_chars.len() && j + k < s2_chars.len() && s1_chars[i + k] == s2_chars[j + k] {
                k += 1;
            }
            if k > max_len {
                max_len = k;
            }
        }
    }

    max_len
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