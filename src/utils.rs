use crate::models::{BangumiResult, Statistics};
use chrono::{Datelike, NaiveDate};

pub fn is_undetermined_date(date_str: &str) -> bool {
    // 检查是否包含具体到日一级的日期格式：YYYY/MM/DD
    let specific_date_pattern = regex::Regex::new(r"\d{4}/\d{1,2}/\d{1,2}").unwrap();

    // 如果包含具体日期，就不是未定日期
    if specific_date_pattern.is_match(date_str) {
        return false;
    }

    // 其他情况都视为未定日期
    true
}

pub fn parse_air_date(date_str: &str) -> Option<NaiveDate> {
    // 解析日文日期格式，如 "2025/09/01(火)"

    // 首先尝试匹配 "YYYY/MM/DD(曜日)" 格式
    if let Some(caps) = regex::Regex::new(r"(\d{4})/(\d{1,2})/(\d{1,2})\([月火水木金土日]\)")
        .unwrap()
        .captures(date_str)
    {
        let year = caps[1].parse::<i32>().ok()?;
        let month = caps[2].parse::<u32>().ok()?;
        let day = caps[3].parse::<u32>().ok()?;
        return NaiveDate::from_ymd_opt(year, month, day);
    }

    // 尝试匹配 "YYYY/MM/DD" 格式
    if let Some(caps) = regex::Regex::new(r"(\d{4})/(\d{1,2})/(\d{1,2})")
        .unwrap()
        .captures(date_str)
    {
        let year = caps[1].parse::<i32>().ok()?;
        let month = caps[2].parse::<u32>().ok()?;
        let day = caps[3].parse::<u32>().ok()?;
        return NaiveDate::from_ymd_opt(year, month, day);
    }

    // 尝试匹配 "YYYY年MM月" 格式
    if let Some(caps) = regex::Regex::new(r"(\d{4})年(\d{1,2})月")
        .unwrap()
        .captures(date_str)
    {
        let year = caps[1].parse::<i32>().ok()?;
        let month = caps[2].parse::<u32>().ok()?;
        // 对于只有年月的情况，使用该月的第一天
        return NaiveDate::from_ymd_opt(year, month, 1);
    }

    // 尝试匹配 "YYYY/MM" 格式
    if let Some(caps) = regex::Regex::new(r"(\d{4})/(\d{1,2})")
        .unwrap()
        .captures(date_str)
    {
        let year = caps[1].parse::<i32>().ok()?;
        let month = caps[2].parse::<u32>().ok()?;
        // 对于只有年月的情况，使用该月的第一天
        return NaiveDate::from_ymd_opt(year, month, 1);
    }

    None
}

pub fn extract_season_name_from_table_title(table_title: &str) -> String {
    // 从表格标题中提取季节信息
    // 常见的表格标题格式如："2025年秋アニメ", "2025年10月新番"等

    // 首先尝试匹配 "yyyy年mm月新番" 格式
    if let Some(caps) = regex::Regex::new(r"(\d{4})年(\d{1,2})月新番")
        .unwrap()
        .captures(table_title)
    {
        let year = &caps[1];
        let month = &caps[2];
        return format!("{}年{}月新番", year, month);
    }

    // 尝试匹配 "yyyy年mm月" 格式
    if let Some(caps) = regex::Regex::new(r"(\d{4})年(\d{1,2})月")
        .unwrap()
        .captures(table_title)
    {
        let year = &caps[1];
        let month = &caps[2];
        return format!("{}年{}月新番", year, month);
    }

    // 尝试匹配 "yyyy年 秋/冬/春/夏" 格式
    if let Some(caps) = regex::Regex::new(r"(\d{4})年\s*(秋|冬|春|夏)")
        .unwrap()
        .captures(table_title)
    {
        let year = &caps[1];
        let season = &caps[2];
        // 将季节转换为月份
        let month = match season {
            "春" => "04",
            "夏" => "07",
            "秋" => "10",
            "冬" => "01",
            _ => "01",
        };
        return format!("{}年{}月新番", year, month);
    }

    // 如果无法从表格标题中提取，使用默认格式
    // 这里可以根据当前日期生成默认的季节名称
    let now = chrono::Local::now();
    let current_year = now.year();
    let current_month = now.month();

    // 根据当前月份确定季节
    let season_month = match current_month {
        1..=3 => "01", // 冬季
        4..=6 => "04", // 春季
        7..=9 => "07", // 夏季
        _ => "10",     // 秋季
    };

    format!("{}年{}月新番", current_year, season_month)
}

pub fn cache_results(results: &[BangumiResult]) -> Result<(), Box<dyn std::error::Error>> {
    let cache_file = "bangumi_results.json";
    let json_content = serde_json::to_string_pretty(results)?;
    std::fs::write(cache_file, json_content)?;
    log::info!("结果已缓存到: {}", cache_file);
    Ok(())
}

pub fn generate_statistics_report(stats: &Statistics, bangumi_results: &[BangumiResult], failed_works: &[(String, String)]) {
    log::info!("{}", "=".repeat(60));
    log::info!("📊 程序运行统计报告");
    log::info!("{}", "=".repeat(60));
    log::info!("表格处理统计:");
    log::info!(
        "  - 从表格中解析出的作品总数: {}",
        stats.total_works_from_table
    );
    log::info!(
        "  - 日期未定的作品数: {}",
        stats.works_with_undetermined_date
    );
    log::info!(
        "  - 经过AI处理的作品数: {}",
        stats.works_processed_by_ai
    );

    log::info!("Bangumi API搜索结果:");
    log::info!(
        "  - 成功找到Bangumi信息的作品: {}",
        stats.works_with_bangumi_info
    );
    log::info!(
        "  - 未找到Bangumi信息的作品: {}",
        stats.works_without_bangumi_info
    );

    log::info!("qBittorrent规则生成:");
    log::info!("  - 生成的下载规则数量: {}", stats.qb_rules_generated);
    log::info!("  - 规则生成失败数量: {}", stats.qb_rules_failed);

    // 显示规则生成失败的作品和原因
    if !failed_works.is_empty() {
        log::info!("规则生成失败的作品列表:");
        for (work_name, reason) in failed_works {
            log::info!("  - {} (原因: {})", work_name, reason);
        }
    }

    // 显示重复作品统计（如果有的话）
    let total_bangumi_works = stats.works_with_bangumi_info + stats.works_without_bangumi_info;
    let expected_rules = total_bangumi_works - stats.qb_rules_failed;
    if stats.qb_rules_generated < expected_rules {
        let duplicate_count = expected_rules - stats.qb_rules_generated;
        log::info!("重复作品处理:");
        log::info!("  - 检测到 {} 个重复作品（相同作品名称）", duplicate_count);
        log::info!("  - 重复作品已自动合并，只生成一个下载规则");
    }

    log::info!("AI API使用统计:");
    log::info!("  - AI请求次数: {}", stats.ai_requests_count);
    log::info!("  - 输入Token总数: {}", stats.ai_input_tokens);
    log::info!("  - 输出Token总数: {}", stats.ai_output_tokens);
    log::info!(
        "  - Token总计: {}",
        stats.ai_input_tokens + stats.ai_output_tokens
    );

    log::info!("未找到Bangumi信息的作品列表:");
    let mut not_found_count = 0;
    for result in bangumi_results {
        if result.bangumi_id.is_none() {
            log::info!(
                "  - {} (原标题: {})",
                result.cleaned_title, result.original_title
            );
            not_found_count += 1;
        }
    }
    if not_found_count == 0 {
        log::info!("  - 无");
    }

    log::info!("{}", "=".repeat(60));
    log::info!("🎉 处理完成！");
    log::info!("{}", "=".repeat(60));
}