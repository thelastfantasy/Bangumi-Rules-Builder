use crate::models::{AnimeWork, TableInfo, Task};
use crate::utils::{extract_season_name_from_table_title, cache_results};
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
    println!("找到 {} 个表格", tables.len());

    // 使用AI API智能匹配表格并处理作品
    let ai_config = crate::models::AiConfig::deepseek();
    let (matched_table, _processed_works, mut stats) =
        crate::ai::deepseek::match_and_process_with_ai(&task.description, &tables, &ai_config).await?;

    if let Some((table, works)) = matched_table {
        println!("匹配到的表格标题: {}", table.title);
        println!("表格内容已暂存");
        println!("提取到 {} 个作品", works.len());

        // 搜索Bangumi API
        let bangumi_results = crate::meta_providers::bangumi::search_bangumi_for_works(&works).await?;

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
        let rule_result = crate::rules::q_bittorrent::generate_qb_rules(&bangumi_results, &task.root_path, &season_name)?;
        let rules_file = "qb_download_rules.json";
        std::fs::write(rules_file, serde_json::to_string_pretty(&rule_result.rules)?)?;
        stats.qb_rules_generated = rule_result.rules.as_object().unwrap().len();
        stats.qb_rules_failed = rule_result.failed_works.len();
        println!("qBittorrent规则已生成到: {}", rules_file);

        // 生成统计报告
        crate::utils::generate_statistics_report(&stats, &bangumi_results, &rule_result.failed_works);
    } else {
        println!("未找到匹配的表格");
    }

    Ok(())
}

pub fn extract_tables_with_titles(html: &str) -> Result<Vec<TableInfo>, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);
    let table_selector = Selector::parse("table").unwrap();
    let mut tables = Vec::new();

    for table_element in document.select(&table_selector) {
        // 获取表格前面的文本作为标题
        let mut title = String::new();

        // 查找表格前面的标题元素（h1-h6, strong, b等）
        let heading_selectors = [
            "h1", "h2", "h3", "h4", "h5", "h6", "strong", "b", ".title", ".heading",
        ];

        for selector_str in heading_selectors {
            if let Ok(selector) = Selector::parse(selector_str)
                && let Some(heading) = document.select(&selector).find(|_el| {
                    // 简化逻辑：先尝试找到任何标题
                    true
                })
            {
                title = heading.text().collect::<String>().trim().to_string();
                if !title.is_empty() {
                    break;
                }
            }
        }

        // 如果没找到标题，查找表格前面的文本节点
        if title.is_empty() {
            // 简化方法：查找表格前面的兄弟元素
            let mut prev_elements = Vec::new();

            // 查找表格前面的元素
            let all_elements: Vec<_> = document.select(&Selector::parse("*").unwrap()).collect();

            for element in &all_elements {
                if element.value().name() == "table" {
                    break;
                }
                prev_elements.push(element);
            }

            // 从后往前查找第一个有文本的元素
            for element in prev_elements.iter().rev() {
                let element_text = element.text().collect::<String>().trim().to_string();
                if !element_text.is_empty() {
                    title = element_text;
                    break;
                }
            }
        }

        let table_html = table_element.html();
        tables.push(TableInfo { title, table_html });
    }

    Ok(tables)
}

pub fn parse_table_works(table_html: &str) -> Result<Vec<AnimeWork>, Box<dyn std::error::Error>> {
    let document = Html::parse_fragment(table_html);
    let tr_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let th_selector = Selector::parse("th").unwrap();
    let mut works = Vec::new();

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

            // 过滤掉日期未定的项目
            if !title_cell.is_empty() && !crate::utils::is_undetermined_date(&date_cell) {
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

    Ok(works)
}


