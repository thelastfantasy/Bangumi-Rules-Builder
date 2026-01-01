use crate::models::{AnimeWork, TableInfo, Task};
use crate::utils::{cache_results, extract_season_name_from_table_title};
use scraper::{Html, Selector};

pub async fn process_kansou_site(task: &Task) -> Result<(), Box<dyn std::error::Error>> {
    // 获取网页内容
    let url = "https://www.kansou.me/";
    log::info!("正在获取网页内容: {}", url);

    let client = reqwest::Client::new();
    let response = match client.get(url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                log::info!("成功获取网页内容，状态码: {}", response.status());
                response
            } else {
                log::error!("获取网页内容失败，状态码: {}", response.status());
                return Err(format!("HTTP请求失败，状态码: {}", response.status()).into());
            }
        }
        Err(e) => {
            log::error!("网络请求失败: {}", e);
            return Err(e.into());
        }
    };

    let html_content = match response.text().await {
        Ok(content) => {
            log::debug!("成功获取HTML内容，长度: {} 字节", content.len());
            content
        }
        Err(e) => {
            log::error!("读取响应内容失败: {}", e);
            return Err(e.into());
        }
    };

    // 解析HTML提取表格和标题
    let tables = extract_tables_with_titles(&html_content)?;
    log::info!("找到 {} 个表格", tables.len());

    // 使用AI API智能匹配表格并处理作品
    let ai_config = crate::models::AiConfig::deepseek();
    let (matched_table, _processed_works, mut stats) =
        crate::ai::deepseek::match_and_process_with_ai(&task.description, &tables, &ai_config)
            .await?;

    if let Some((table, works)) = matched_table {
        log::info!("匹配到的表格标题: {}", table.title);
        log::info!("表格内容已暂存");
        log::info!("提取到 {} 个作品", works.len());

        // 搜索Bangumi API
        let bangumi_results =
            crate::meta_providers::bangumi::search_bangumi_for_works(&works).await?;

        // 统计Bangumi搜索结果
        stats.works_with_bangumi_info = bangumi_results
            .iter()
            .filter(|r| r.bangumi_id.is_some())
            .count();
        stats.works_without_bangumi_info = bangumi_results.len() - stats.works_with_bangumi_info;

        // 缓存结果
        cache_results(&bangumi_results)?;

        // 从表格标题中提取季节信息
        let season_name = extract_season_name_from_table_title(&table.title);

        // 生成qBittorrent规则
        let rule_result =
            crate::rules::q_bittorrent::generate_qb_rules(&bangumi_results, task, &season_name)?;
        let rules_file = "qb_download_rules.json";
        std::fs::write(
            rules_file,
            serde_json::to_string_pretty(&rule_result.rules)?,
        )?;
        stats.qb_rules_generated = rule_result.rules.as_object().unwrap().len();
        stats.qb_rules_failed = rule_result.failed_works.len();
        log::info!("qBittorrent规则已生成到: {}", rules_file);

        // 生成统计报告
        crate::utils::generate_statistics_report(
            &stats,
            &bangumi_results,
            &rule_result.failed_works,
        );
    } else {
        log::warn!("未找到匹配的表格");
    }

    Ok(())
}

pub fn extract_tables_with_titles(
    html: &str,
) -> Result<Vec<TableInfo>, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);

    // 1. 收集所有h2标题（只包含有效的动画季节标题）
    let h2_selector = Selector::parse("h2").unwrap();
    let mut h2_titles = Vec::new();

    for h2_element in document.select(&h2_selector) {
        let title_text = h2_element.text().collect::<String>().trim().to_string();

        // 检查是否是有效的动画季节标题
        if title_text.contains("年") &&
           (title_text.contains("月") || title_text.contains("春") ||
            title_text.contains("夏") || title_text.contains("秋") ||
            title_text.contains("冬") || title_text.contains("放送")) {
            h2_titles.push(title_text);
        }
    }

    log::debug!("找到 {} 个有效的h2标题: {:?}", h2_titles.len(), h2_titles);

    // 2. 收集所有表格
    let table_selector = Selector::parse("table").unwrap();
    let mut tables = Vec::new();

    for (table_index, table_element) in document.select(&table_selector).enumerate() {
        let title;

        // 3. 为表格分配标题：按索引配对
        if table_index < h2_titles.len() {
            title = h2_titles[table_index].clone();
            log::debug!("表格[{}] 分配标题: {}", table_index, title);
        } else {
            // 如果h2标题不够，尝试查找其他标题或使用默认值
            if let Some(id) = table_element.value().attr("id") {
                title = format!("表格ID: {}", id);
            } else if let Some(class) = table_element.value().attr("class") {
                title = format!("表格类: {}", class);
            } else {
                title = format!("未命名表格-{}", table_index);
            }
            log::debug!("表格[{}] 使用默认标题: {}", table_index, title);
        }

        let table_html = table_element.html();
        tables.push(TableInfo { title, table_html });
    }

    log::info!("提取了 {} 个表格", tables.len());
    Ok(tables)
}

pub fn parse_table_works(
    table_html: &str,
) -> Result<(Vec<AnimeWork>, usize), Box<dyn std::error::Error>> {
    let document = Html::parse_fragment(table_html);
    let tr_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let th_selector = Selector::parse("th").unwrap();
    let mut works = Vec::new();
    let mut undetermined_date_count = 0;

    // 首先找到表头，确定列的位置
    let mut title_col_index = None;
    let mut date_col_index = None;

    if let Some(header_row) = document.select(&tr_selector).next() {
        for (i, cell) in header_row.select(&th_selector).enumerate() {
            let cell_text = cell.text().collect::<String>().trim().to_string();
            if cell_text.contains("作品名") || cell_text.contains("タイトル") {
                title_col_index = Some(i);
            } else if cell_text.contains("放送開始日") {
                date_col_index = Some(i);
            }
        }
    }

    // 如果没有找到表头，使用默认位置
    let title_col_index = title_col_index.unwrap_or(0);
    let date_col_index = date_col_index.unwrap_or(1);

    // 处理数据行
    for row in document.select(&tr_selector).skip(1) {
        // 跳过表头
        let cells: Vec<_> = row.select(&td_selector).collect();

        if cells.len() > title_col_index && cells.len() > date_col_index {
            let title_cell = cells[title_col_index]
                .text()
                .collect::<String>()
                .trim()
                .to_string();
            let date_cell = cells[date_col_index]
                .text()
                .collect::<String>()
                .trim()
                .to_string();

            // 统计日期未定的项目
            if !title_cell.is_empty() {
                if crate::utils::is_undetermined_date(&date_cell) {
                    undetermined_date_count += 1;
                } else {
                    let air_date = crate::utils::parse_air_date(&date_cell);

                    works.push(AnimeWork {
                        original_title: title_cell.clone(),
                        cleaned_title: title_cell, // 暂时使用原标题，后面会清理
                        air_date,
                        keywords: Vec::new(),
                    });
                }
            }
        }
    }

    Ok((works, undetermined_date_count))
}
