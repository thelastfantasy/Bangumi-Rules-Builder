use crate::models::{AnimeWork, BangumiResult, BangumiSubject, BangumiInfoboxItem, AiConfig};
use crate::ai::object_matcher::{SourceWork, CandidateWork, batch_process_searches};
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
    pb.set_message("准备批量搜索...");

    // 准备批量搜索任务
    let mut search_tasks = Vec::new();
    let mut work_indices = Vec::new();

    for (index, work) in works.iter().enumerate() {
        pb.set_message(format!(
            "准备搜索任务: {} ({}/{})",
            work.cleaned_title,
            index + 1,
            total_works
        ));

        // 构建搜索关键词数组：包含cleaned_title和keywords，并去重
        let mut search_keywords: Vec<&str> = Vec::new();
        search_keywords.push(&work.cleaned_title);
        search_keywords.extend(work.keywords.iter().map(|s| s.as_str()));

        // 去重
        search_keywords.sort();
        search_keywords.dedup();

        // 为每个关键词创建搜索任务
        for keyword in search_keywords {
            // 先搜索Bangumi获取候选作品
            let subjects = search_bangumi_with_keyword(&client, keyword, &work.air_date).await?;

            if !subjects.is_empty() {
                // 创建源作品
                let source_work = SourceWork {
                    original_title: work.original_title.clone(),
                    cleaned_title: work.cleaned_title.clone(),
                    air_date: work.air_date.map(|d| d.to_string()),
                    keywords: work.keywords.clone(),
                };

                // 创建候选作品列表
                let candidate_works: Vec<CandidateWork> = subjects
                    .iter()
                    .map(|subject| CandidateWork::from(subject))
                    .collect();

                search_tasks.push((source_work, candidate_works));
                work_indices.push(index);

                // 每个作品只使用第一个成功的关键词
                break;
            }
        }

        // 更新进度条
        pb.inc(1);
    }

    pb.set_message("使用AI进行批量匹配...");

    // 使用批量AI匹配
    let ai_config = AiConfig::deepseek();
    let batch_size = 5; // 每批次5个任务
    let matched_ids = batch_process_searches(&search_tasks, &ai_config, batch_size).await?;

    // 处理匹配结果
    for (index, work) in works.iter().enumerate() {
        let mut found = false;

        // 查找该作品的匹配结果
        for (task_index, &work_index) in work_indices.iter().enumerate() {
            if work_index == index {
                if let Some(bangumi_id) = matched_ids[task_index] {
                    // 找到匹配，创建BangumiResult
                    // 从候选作品中提取详细信息
                    let search_task = &search_tasks[task_index];
                    let candidate_works = &search_task.1;

                    // 查找匹配的候选作品
                    if let Some(matched_candidate) = candidate_works.iter().find(|c| c.bangumi_id == bangumi_id) {
                        let chinese_name = if !matched_candidate.chinese_title.is_empty() {
                            Some(matched_candidate.chinese_title.clone())
                        } else {
                            None
                        };

                        results.push(BangumiResult {
                            original_title: work.original_title.clone(),
                            cleaned_title: work.cleaned_title.clone(),
                            bangumi_id: Some(bangumi_id),
                            chinese_name,
                            aliases: matched_candidate.aliases.clone(),
                            air_date: work.air_date,
                            keywords: work.keywords.clone(),
                        });

                        found = true;
                        break;
                    }
                }
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
    }

    // 完成进度条
    pb.finish_with_message("Bangumi批量搜索完成");

    Ok(results)
}

pub async fn search_bangumi_with_keyword(
    client: &reqwest::Client,
    keyword: &str,
    air_date: &Option<NaiveDate>,
) -> Result<Vec<BangumiSubject>, Box<dyn std::error::Error>> {
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
            // 返回所有搜索结果，让批量处理来处理匹配
            let subjects: Vec<BangumiSubject> = data_array
                .iter()
                .filter_map(|subject_data| {
                    serde_json::from_value::<BangumiSubject>(subject_data.clone()).ok()
                })
                .collect();

            if is_problem_work {
                println!("🔍 调试：找到 {} 个搜索结果", subjects.len());
            }

            return Ok(subjects);
        }
    }

    if is_problem_work {
        println!("🔍 调试：未找到搜索结果");
    }

    Ok(Vec::new())
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