use crate::models::BangumiSubject;

/// 从Bangumi主题中提取放映日期
/// 优先从顶层的date字段提取，如果失败则从infobox中提取
pub fn extract_air_date_from_subject(subject: &BangumiSubject) -> Option<chrono::NaiveDate> {
    // 首先尝试从顶层的date字段提取
    if let Some(ref date_str) = subject.date {
        // 尝试多种日期格式
        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            return Some(date);
        }
        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y年%m月%d日") {
            return Some(date);
        }
    }

    // 如果顶层date字段没有或格式不匹配，尝试从infobox中提取
    extract_air_date_from_infobox(&subject.infobox)
}

/// 从Bangumi infobox中提取放映日期
pub fn extract_air_date_from_infobox(infobox: &[crate::models::BangumiInfoboxItem]) -> Option<chrono::NaiveDate> {
    for item in infobox {
        if (item.key == "放送开始" || item.key == "开始")
            && let serde_json::Value::String(date_str) = &item.value
        {
            // 尝试多种日期格式
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                return Some(date);
            }
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y年%m月%d日") {
                return Some(date);
            }
        }
    }
    None
}