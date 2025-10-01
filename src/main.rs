use std::fs;

mod ai;
mod meta_providers;
mod models;
mod rules;
mod sites;
mod utils;

use crate::models::Task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 读取tasks.json文件
    let json_content = fs::read_to_string("tasks.json")?;
    let task: Task = serde_json::from_str(&json_content)?;

    println!("任务描述: {}", task.description);
    println!("站点: {}", task.site);

    match task.site {
        models::SiteType::Kansou => {
            println!("处理kansou站点...");
            sites::kansou::process_kansou_site(&task).await?;
        } // 未来添加其他站点支持
          // models::SiteType::ModelScope => {
          //     println!("处理modelscope站点...");
          //     sites::modelscope::process_modelscope_site(&task).await?;
          // }
          // models::SiteType::AnimeList => {
          //     println!("处理animelist站点...");
          //     sites::animelist::process_animelist_site(&task).await?;
          // }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::meta_providers::bangumi::{search_bangumi_for_works, search_bangumi_with_keyword};
    use crate::models::AnimeWork;

    #[tokio::test]
    async fn test_specific_work() -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🧪 测试特定作品的Bangumi搜索...");

        // 创建测试作品数据
        let test_work = AnimeWork {
            original_title: "青のミブロ 第二期「芹沢暗殺編」".to_string(),
            cleaned_title: "青のミブロ 第二期 芹沢暗殺編".to_string(),
            air_date: Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 20).unwrap()),
            keywords: vec![
                "青のミブロ 第二期".to_string(),
                "青のミブロ2".to_string(),
                "青のミブロ 芹沢暗殺編".to_string(),
                "Ao no Miburo Season 2".to_string(),
                "青色火焰 第二季".to_string(),
                "青之炎 第二期".to_string(),
                "青之壬生狼 第二季".to_string(),
            ],
        };

        println!("测试作品: {}", test_work.cleaned_title);
        println!("关键词数量: {}", test_work.keywords.len());
        println!("关键词列表: {:?}", test_work.keywords);

        // 测试搜索
        let results = search_bangumi_for_works(&[test_work]).await?;

        if let Some(result) = results.first() {
            println!("搜索结果: {:?}", result);
            if result.bangumi_id.is_some() {
                println!("✅ 成功找到Bangumi信息!");
                println!("   Bangumi ID: {}", result.bangumi_id.unwrap());
                println!("   中文名称: {:?}", result.chinese_name);
                println!("   别名: {:?}", result.aliases);
            } else {
                println!("❌ 未找到Bangumi信息");
                println!("⚠️ 问题分析:");
                println!("   - 关键词测试显示 '青のミブロ' 能找到作品 (ID: 454630)");
                println!("   - 但完整作品搜索时没有匹配成功");
                println!("   - 可能原因: 匹配阈值过高或日期过滤问题");
            }
        } else {
            println!("❌ 没有搜索结果");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_specific_keywords() -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🧪 测试特定关键词的Bangumi搜索...");
        let client = reqwest::Client::new();

        // 测试 "青のミブロ" 关键词
        println!("\n📝 测试关键词: 青のミブロ 第二期 芹沢暗殺編");
        if let Some(subject) =
            search_bangumi_with_keyword(&client, "青のミブロ 第二期 芹沢暗殺編", &None).await?
        {
            println!("✅ 成功找到作品: {}", subject.name);
            println!("   Bangumi ID: {}", subject.id);
            println!("   中文名称: {}", subject.name_cn);
        } else {
            println!("❌ 未找到作品");
        }

        // 测试 "SHIBUYA HACHI" 关键词
        println!("\n📝 测试关键词: SHIBUYA HACHI 第4クール");
        if let Some(subject) =
            search_bangumi_with_keyword(&client, "SHIBUYA HACHI 第4クール", &None).await?
        {
            println!("✅ 成功找到作品: {}", subject.name);
            println!("   Bangumi ID: {}", subject.id);
            println!("   中文名称: {}", subject.name_cn);
        } else {
            println!("❌ 未找到作品");
        }

        // 测试 "Ao no Miburo" 关键词
        println!("\n📝 测试关键词: Ao no Miburo");
        if let Some(subject) = search_bangumi_with_keyword(&client, "Ao no Miburo", &None).await? {
            println!("✅ 成功找到作品: {}", subject.name);
            println!("   Bangumi ID: {}", subject.id);
            println!("   中文名称: {}", subject.name_cn);
        } else {
            println!("❌ 未找到作品");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_without_date_filter() -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🧪 测试无日期过滤的Bangumi搜索...");

        // 创建测试作品数据，但不设置日期
        let test_work = AnimeWork {
            original_title: "青のミブロ 第二期「芹沢暗殺編」".to_string(),
            cleaned_title: "青のミブロ 第二期".to_string(),
            air_date: None, // 不设置日期
            keywords: vec![
                "青のミブロ 第二期".to_string(),
                "青のミブロ2".to_string(),
                "青のミブロ 芹沢暗殺編".to_string(),
                "Ao no Miburo Season 2".to_string(),
                "青色火焰 第二季".to_string(),
                "青之炎 第二期".to_string(),
                "青之壬生狼 第二季".to_string(),
            ],
        };

        println!("测试作品: {}", test_work.cleaned_title);
        println!("关键词数量: {}", test_work.keywords.len());
        println!("无日期过滤");

        // 测试搜索
        let results = search_bangumi_for_works(&[test_work]).await?;

        if let Some(result) = results.first() {
            println!("搜索结果: {:?}", result);
            if result.bangumi_id.is_some() {
                println!("✅ 成功找到Bangumi信息!");
                println!("   Bangumi ID: {}", result.bangumi_id.unwrap());
                println!("   中文名称: {:?}", result.chinese_name);
                println!("   别名: {:?}", result.aliases);
            } else {
                println!("❌ 未找到Bangumi信息");
            }
        } else {
            println!("❌ 没有搜索结果");
        }

        Ok(())
    }
}
