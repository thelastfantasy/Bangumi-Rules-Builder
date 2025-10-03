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
        let subjects =
            search_bangumi_with_keyword(&client, "青のミブロ 第二期 芹沢暗殺編", &None).await?;
        if !subjects.is_empty() {
            let subject = &subjects[0];
            println!("✅ 成功找到作品: {}", subject.name);
            println!("   Bangumi ID: {}", subject.id);
            println!("   中文名称: {}", subject.name_cn);
        } else {
            println!("❌ 未找到作品");
        }

        // 测试 "SHIBUYA HACHI" 关键词
        println!("\n📝 测试关键词: SHIBUYA HACHI 第4クール");
        let subjects =
            search_bangumi_with_keyword(&client, "SHIBUYA HACHI 第4クール", &None).await?;
        if !subjects.is_empty() {
            let subject = &subjects[0];
            println!("✅ 成功找到作品: {}", subject.name);
            println!("   Bangumi ID: {}", subject.id);
            println!("   中文名称: {}", subject.name_cn);
        } else {
            println!("❌ 未找到作品");
        }

        // 测试 "Ao no Miburo" 关键词
        println!("\n📝 测试关键词: Ao no Miburo");
        let subjects = search_bangumi_with_keyword(&client, "Ao no Miburo", &None).await?;
        if !subjects.is_empty() {
            let subject = &subjects[0];
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

    #[tokio::test]
    async fn test_ai_individual_matching() -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🧪 测试AI单个匹配...");

        use crate::ai::object_matcher::{CandidateWork, match_works_with_ai};
        use crate::meta_providers::bangumi::search_bangumi_for_works;
        use crate::models::{AiConfig, AnimeWork};

        let ai_config = AiConfig::deepseek();

        // 测试案例1: 破产富豪
        println!("\n📝 测试案例1: 破产富豪");
        let anime_work1 = AnimeWork {
            original_title: "破産富豪".to_string(),
            cleaned_title: "破产富豪".to_string(),
            air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 1),
            keywords: vec!["破产富豪".to_string(), "Bankrupt Billionaire".to_string()],
        }; // 该数据应该匹配不到任何结果

        let bangumi_results1 = search_bangumi_for_works(&[anime_work1.clone()]).await?;

        // 从Bangumi结果中提取候选作品信息
        let candidate_works1: Vec<CandidateWork> = bangumi_results1
            .iter()
            .filter_map(|result| {
                if let Some(bangumi_id) = result.bangumi_id {
                    Some(CandidateWork {
                        bangumi_id,
                        japanese_title: result.original_title.clone(),
                        chinese_title: result.chinese_name.clone().unwrap_or_default(),
                        air_date: result.air_date.map(|d| d.to_string()),
                        aliases: result.aliases.clone(),
                        score: None,
                    })
                } else {
                    None
                }
            })
            .collect();

        println!("找到 {} 个候选作品", candidate_works1.len());
        let result1 = match_works_with_ai(&anime_work1, &candidate_works1, &ai_config).await?;
        println!("匹配结果: {:?}", result1);
        println!("预期结果: None");
        assert_eq!(result1, None, "破产富豪应该匹配不到任何结果");
        println!("✅ 匹配结果符合预期");

        // 测试案例
        println!("\n📝 测试案例");
        let anime_work2 = AnimeWork {
            original_title: "ある日、お姫様になってしまった件について".to_string(),
            cleaned_title: "某天成为公主".to_string(),
            air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 1),
            keywords: vec![
                "某天成为公主".to_string(),
                "有一天我变成了公主".to_string(),
                "One Day I Became a Princess".to_string(),
            ],
        }; // 该数据应该匹配魔法公主的小烦恼，bangumi_id: 434807

        let bangumi_results2 = search_bangumi_for_works(&[anime_work2.clone()]).await?;

        let candidate_works2: Vec<CandidateWork> = bangumi_results2
            .iter()
            .filter_map(|result| {
                if let Some(bangumi_id) = result.bangumi_id {
                    Some(CandidateWork {
                        bangumi_id,
                        japanese_title: result.original_title.clone(),
                        chinese_title: result.chinese_name.clone().unwrap_or_default(),
                        air_date: result.air_date.map(|d| d.to_string()),
                        aliases: result.aliases.clone(),
                        score: None,
                    })
                } else {
                    None
                }
            })
            .collect();

        println!("找到 {} 个候选作品", candidate_works2.len());
        let result2 = match_works_with_ai(&anime_work2, &candidate_works2, &ai_config).await?;
        println!("匹配结果: {:?}", result2);
        println!("预期结果: Some(434807)");
        assert_eq!(result2, Some(434807), "某天成为公主应该匹配到魔法公主的小烦恼 (ID: 434807)");
        println!("✅ 匹配结果符合预期");

        // 测试案例3: 罗小黑战记
        println!("\n📝 测试案例3: 罗小黑战记");
        let anime_work3 = AnimeWork {
            original_title: "羅小黒戦記".to_string(),
            cleaned_title: "罗小黑战记".to_string(),
            air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 1),
            keywords: vec![
                "罗小黑战记".to_string(),
                "The Legend of Luo Xiao Hei".to_string(),
            ],
        }; // 该数据应该匹配不到任何结果

        let bangumi_results3 = search_bangumi_for_works(&[anime_work3.clone()]).await?;

        let candidate_works3: Vec<CandidateWork> = bangumi_results3
            .iter()
            .filter_map(|result| {
                if let Some(bangumi_id) = result.bangumi_id {
                    Some(CandidateWork {
                        bangumi_id,
                        japanese_title: result.original_title.clone(),
                        chinese_title: result.chinese_name.clone().unwrap_or_default(),
                        air_date: result.air_date.map(|d| d.to_string()),
                        aliases: result.aliases.clone(),
                        score: None,
                    })
                } else {
                    None
                }
            })
            .collect();

        println!("找到 {} 个候选作品", candidate_works3.len());
        let result3 = match_works_with_ai(&anime_work3, &candidate_works3, &ai_config).await?;
        println!("匹配结果: {:?}", result3);
        println!("预期结果: None");
        assert_eq!(result3, None, "罗小黑战记应该匹配不到任何结果");
        println!("✅ 匹配结果符合预期");

        // 测试案例4: 异世界四重奏3
        println!("\n📝 测试案例4: 异世界四重奏3");
        let anime_work4 = AnimeWork {
            original_title: "異世界かるてっと3".to_string(),
            cleaned_title: "異世界かるてっと3".to_string(),
            air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 13),
            keywords: vec![
                "異世界かるてっと3".to_string(),
                "異世界かるてっと 3".to_string(),
                "Isekai Quartet 3".to_string(),
                "Isekai Quartet Season 3".to_string(),
                "异世界四重奏3".to_string(),
                "异世界四重奏 第三季".to_string(),
                "Isekai Quartet 第三季".to_string(),
            ],
        }; // 该数据应该匹配到 异世界四重奏 第三季，bangumi_id: 564421

        let bangumi_results4 = search_bangumi_for_works(&[anime_work4.clone()]).await?;

        let candidate_works4: Vec<CandidateWork> = bangumi_results4
            .iter()
            .filter_map(|result| {
                if let Some(bangumi_id) = result.bangumi_id {
                    Some(CandidateWork {
                        bangumi_id,
                        japanese_title: result.original_title.clone(),
                        chinese_title: result.chinese_name.clone().unwrap_or_default(),
                        air_date: result.air_date.map(|d| d.to_string()),
                        aliases: result.aliases.clone(),
                        score: None,
                    })
                } else {
                    None
                }
            })
            .collect();

        println!("找到 {} 个候选作品", candidate_works4.len());
        let result4 = match_works_with_ai(&anime_work4, &candidate_works4, &ai_config).await?;
        println!("匹配结果: {:?}", result4);
        println!("预期结果: Some(564421)");
        assert_eq!(result4, Some(564421), "异世界四重奏3应该匹配到异世界四重奏 第三季 (ID: 564421)");
        println!("✅ 匹配结果符合预期");

        // 测试案例5: 怪物弹珠 Dead Death Reloaded
        println!("\n📝 测试案例5: 怪物弹珠 Dead Death Reloaded");
        let anime_work5 = AnimeWork {
            original_title: "モンスターストライク デッドバースリローデッド".to_string(),
            cleaned_title: "モンスターストライク デッドバースリローデッド".to_string(),
            air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 21),
            keywords: vec![
                "モンスターストライク デッドバースリローデッド".to_string(),
                "モンスターストライク デッドバース リローデッド".to_string(),
                "Monster Strike Dead Death Reloaded".to_string(),
                "怪物弹珠 Dead Death Reloaded".to_string(),
                "Monster Strike Dead Death Reloaded".to_string(),
                "怪物弹珠 死亡重载".to_string(),
                "MonSt Dead Death Reloaded".to_string(),
            ],
        }; // 该数据应该匹配到怪物弹珠 DEADVERSE RELOADED，bangumi_id: 570330

        let bangumi_results5 = search_bangumi_for_works(&[anime_work5.clone()]).await?;

        let candidate_works5: Vec<CandidateWork> = bangumi_results5
            .iter()
            .filter_map(|result| {
                if let Some(bangumi_id) = result.bangumi_id {
                    Some(CandidateWork {
                        bangumi_id,
                        japanese_title: result.original_title.clone(),
                        chinese_title: result.chinese_name.clone().unwrap_or_default(),
                        air_date: result.air_date.map(|d| d.to_string()),
                        aliases: result.aliases.clone(),
                        score: None,
                    })
                } else {
                    None
                }
            })
            .collect();

        println!("找到 {} 个候选作品", candidate_works5.len());
        let result5 = match_works_with_ai(&anime_work5, &candidate_works5, &ai_config).await?;
        println!("匹配结果: {:?}", result5);
        println!("预期结果: Some(570330)");
        assert_eq!(result5, Some(570330), "怪物弹珠 Dead Death Reloaded应该匹配到怪物弹珠 DEADVERSE RELOADED (ID: 570330)");
        println!("✅ 匹配结果符合预期");

        Ok(())
    }

    #[tokio::test]
    async fn test_ai_batch_matching() -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🧪 测试AI批量匹配...");

        use crate::ai::object_matcher::{CandidateWork, batch_match_works_with_ai};
        use crate::meta_providers::bangumi::search_bangumi_for_works;
        use crate::models::{AiConfig, AnimeWork};

        let ai_config = AiConfig::deepseek();

        // 使用关键测试案例，包含边界情况和复杂匹配场景
        let source_works = vec![
            // 基础测试案例 - 确保基本功能正常
            AnimeWork {
                original_title: "SHIBUYA♡HACHI 第4クール".to_string(),
                cleaned_title: "SHIBUYA♡HACHI 第4クール".to_string(),
                air_date: None, // 不设置日期，避免日期过滤问题
                keywords: vec![
                    "SHIBUYA♡HACHI 第4クール".to_string(),
                    "SHIBUYA HACHI 第4クール".to_string(),
                    "SHIBUYA HACHI Season 4".to_string(),
                    "SHIBUYA HACHI 第四季".to_string(),
                    "涩谷八 第四部分".to_string(),
                ],
            }, // 该数据应该匹配到SHIBUYA♡HACHI，bangumi_id: 582915
            AnimeWork {
                original_title: "異世界食堂".to_string(),
                cleaned_title: "異世界食堂".to_string(),
                air_date: None, // 不设置日期，避免日期过滤问题
                keywords: vec![
                    "異世界食堂".to_string(),
                    "异世界食堂".to_string(),
                    "Isekai Shokudou".to_string(),
                    "异世界餐厅".to_string(),
                ],
            }, // 该数据应该匹配到异世界食堂，bangumi_id: 192252
            // 关键边界测试案例 - 测试算法不过度匹配
            AnimeWork {
                original_title: "破産富豪".to_string(),
                cleaned_title: "破产富豪".to_string(),
                air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 1),
                keywords: vec!["破产富豪".to_string(), "Bankrupt Billionaire".to_string()],
            }, // 该数据应该匹配不到任何结果
            AnimeWork {
                original_title: "ある日、お姫様になってしまった件について".to_string(),
                cleaned_title: "某天成为公主".to_string(),
                air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 1),
                keywords: vec![
                    "某天成为公主".to_string(),
                    "有一天我变成了公主".to_string(),
                    "One Day I Became a Princess".to_string(),
                ],
            }, // 该数据应该匹配魔法公主的小烦恼，bangumi_id: 434807
            AnimeWork {
                original_title: "架空のアニメ作品".to_string(),
                cleaned_title: "虚构的动画作品".to_string(),
                air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 1),
                keywords: vec![
                    "虚构的动画作品".to_string(),
                    "Fictional Anime Work".to_string(),
                    "不存在于Bangumi的作品".to_string(),
                ],
            }, // 该数据应该匹配不到任何结果
            AnimeWork {
                original_title: "羅小黒戦記".to_string(),
                cleaned_title: "罗小黑战记".to_string(),
                air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 1),
                keywords: vec![
                    "罗小黑战记".to_string(),
                    "The Legend of Luo Xiao Hei".to_string(),
                ],
            }, // 该数据应该匹配不到任何结果
            // 复杂匹配测试案例 - 测试季度和复杂名称匹配
            AnimeWork {
                original_title: "異世界かるてっと3".to_string(),
                cleaned_title: "異世界かるてっと3".to_string(),
                air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 13),
                keywords: vec![
                    "異世界かるてっと3".to_string(),
                    "異世界かるてっと 3".to_string(),
                    "Isekai Quartet 3".to_string(),
                    "Isekai Quartet Season 3".to_string(),
                    "异世界四重奏3".to_string(),
                    "异世界四重奏 第三季".to_string(),
                    "Isekai Quartet 第三季".to_string(),
                ],
            }, // 该数据应该匹配到 异世界四重奏 第三季，bangumi_id: 564421
            AnimeWork {
                original_title: "モンスターストライク デッドバースリローデッド".to_string(),
                cleaned_title: "モンスターストライク デッドバースリローデッド".to_string(),
                air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 21),
                keywords: vec![
                    "モンスターストライク デッドバースリローデッド".to_string(),
                    "モンスターストライク デッドバース リローデッド".to_string(),
                    "Monster Strike Dead Death Reloaded".to_string(),
                    "怪物弹珠 Dead Death Reloaded".to_string(),
                    "Monster Strike Dead Death Reloaded".to_string(),
                    "怪物弹珠 死亡重载".to_string(),
                    "MonSt Dead Death Reloaded".to_string(),
                ],
            }, // 该数据应该匹配到怪物弹珠 DEADVERSE RELOADED，bangumi_id: 570330
            AnimeWork {
                original_title: "ポケモンコンシェルジュ【2nd Season】".to_string(),
                cleaned_title: "ポケモンコンシェルジュ 2nd Season".to_string(),
                air_date: chrono::NaiveDate::from_ymd_opt(2025, 9, 4),
                keywords: vec![
                    "ポケモンコンシェルジュ 2nd Season".to_string(),
                    "ポケモンコンシェルジュ 第二期".to_string(),
                    "Pokemon Concierge Season 2".to_string(),
                    "宝可梦礼宾部 第二季".to_string(),
                    "宝可梦礼宾部2".to_string(),
                    "Pokemon Concierge S2".to_string(),
                ],
            }, // 该数据应该匹配到宝可梦 礼宾部 新剧集，bangumi_id: 481530
        ];

        println!("准备批量测试数据，共 {} 个作品", source_works.len());
        for (i, work) in source_works.iter().enumerate() {
            println!(
                "  作品{}: {} (关键词: {:?})",
                i, work.cleaned_title, work.keywords
            );
        }

        // 第一步：使用search_bangumi_for_works获取所有候选作品
        println!("\n🚀 第一步：搜索Bangumi获取候选作品...");
        let bangumi_results = search_bangumi_for_works(&source_works).await?;

        // 构建候选作品映射 - 每个源作品对应自己的候选作品列表
        let mut candidate_works_map = vec![Vec::new(); source_works.len()];

        // 将Bangumi结果映射回对应的源作品
        for (index, result) in bangumi_results.iter().enumerate() {
            if index < source_works.len() {
                // 为每个Bangumi结果创建候选作品
                if let Some(bangumi_id) = result.bangumi_id {
                    candidate_works_map[index].push(CandidateWork {
                        bangumi_id,
                        japanese_title: result.original_title.clone(),
                        chinese_title: result.chinese_name.clone().unwrap_or_default(),
                        air_date: result.air_date.map(|d| d.to_string()),
                        aliases: result.aliases.clone(),
                        score: None,
                    });
                }
            }
        }

        println!("候选作品映射构建完成:");
        for (i, candidates) in candidate_works_map.iter().enumerate() {
            println!("  作品{}: {} 个候选作品", i, candidates.len());
        }

        // 第二步：使用batch_match_works_with_ai进行批量匹配
        println!("\n🚀 第二步：执行AI批量匹配...");

        // 调试：打印候选作品信息
        for (i, candidates) in candidate_works_map.iter().enumerate() {
            if candidates.is_empty() {
                println!("⚠️ 作品{}: 没有候选作品", i);
            } else {
                println!("📋 作品{}: {} 个候选作品", i, candidates.len());
                for candidate in candidates {
                    println!(
                        "   候选: [ID: {}] {} (中文: {}) (日期: {:?})",
                        candidate.bangumi_id,
                        candidate.japanese_title,
                        candidate.chinese_title,
                        candidate.air_date
                    );
                }
            }
        }

        let source_works_refs: Vec<&AnimeWork> = source_works.iter().collect();
        let candidate_works_map_refs: Vec<&Vec<CandidateWork>> = candidate_works_map.iter().collect();

        let batch_results =
            batch_match_works_with_ai(&source_works_refs, &candidate_works_map_refs, &ai_config).await?;

        // 第三步：验证匹配结果是否符合预期
        println!("\n📊 AI批量匹配结果验证:");
        let mut test_passed = true;

        // 预期结果映射
        let expected_results = vec![
            Some(582915), // 作品0: SHIBUYA♡HACHI - 应该匹配到582915
            Some(192252), // 作品1: 异世界食堂 - 应该匹配到192252
            None,         // 作品2: 破产富豪 - 应该匹配不到任何结果
            Some(434807), // 作品3: 某天成为公主 - 应该匹配魔法公主的小烦恼，bangumi_id: 434807
            None,         // 作品4: 虚构的动画作品 - 应该匹配不到任何结果
            None,         // 作品5: 罗小黑战记 - 应该匹配不到任何结果
            Some(564421), // 作品6: 异世界四重奏3 - 应该匹配到564421
            Some(570330), // 作品7: 怪物弹珠 Dead Death Reloaded - 应该匹配到570330
            Some(481530), // 作品8: ポケモンコンシェルジュ【2nd Season】 - 应该匹配到宝可梦 礼宾部 新剧集，bangumi_id: 481530
        ];

        for (i, (result, expected)) in batch_results
            .iter()
            .zip(expected_results.iter())
            .enumerate()
        {
            let work = &source_works[i];
            println!("\n  作品{}: '{}'", i, work.cleaned_title);
            println!("    预期结果: {:?}", expected);
            println!("    实际结果: {:?}", result);

            if result == expected {
                println!("    ✅ 匹配结果符合预期");
                if let Some(bangumi_id) = result {
                    println!("      匹配到Bangumi ID: {}", bangumi_id);
                }
            } else {
                println!("    ❌ 匹配结果不符合预期");
                test_passed = false;
            }
        }

        // 输出整体测试结果
        println!("\n📈 批量匹配测试总结:");
        println!("   总作品数: {}", source_works.len());
        println!(
            "   预期匹配: {} 个作品",
            expected_results.iter().filter(|r| r.is_some()).count()
        );
        println!(
            "   实际匹配: {} 个作品",
            batch_results.iter().filter(|r| r.is_some()).count()
        );

        if test_passed {
            println!("   ✅ 所有匹配结果都符合预期，测试通过！");
        } else {
            println!("   ❌ 部分匹配结果不符合预期，测试失败！");
            return Err("AI批量匹配测试失败".into());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_problematic_searches() -> Result<(), Box<dyn std::error::Error>> {
        use crate::meta_providers::bangumi::search_bangumi_with_keyword;
        let client = reqwest::Client::new();
        let isekai_date_range = chrono::NaiveDate::from_ymd_opt(2025, 10, 13);
        let monster_date_range = chrono::NaiveDate::from_ymd_opt(2025, 10, 21);

        println!("\n🔍 测试搜索: 異世界かるてっと3");
        let subjects =
            search_bangumi_with_keyword(&client, "異世界かるてっと3", &isekai_date_range).await?;
        println!("搜索结果数量: {}", subjects.len());
        for subject in &subjects {
            println!("  作品: {} (ID: {})", subject.name, subject.id);
        }

        println!("\n🔍 测试搜索: 異世界かるてっと");
        let subjects2 =
            search_bangumi_with_keyword(&client, "異世界かるてっと", &isekai_date_range).await?;
        println!("搜索结果数量: {}", subjects2.len());
        for subject in &subjects2 {
            println!("  作品: {} (ID: {})", subject.name, subject.id);
        }

        println!("\n🔍 测试搜索: 异世界四重奏");
        let subjects3 =
            search_bangumi_with_keyword(&client, "异世界四重奏", &isekai_date_range).await?;
        println!("搜索结果数量: {}", subjects3.len());
        for subject in &subjects3 {
            println!("  作品: {} (ID: {})", subject.name, subject.id);
        }

        println!("\n🔍 测试搜索: モンスターストライク デッドバースリローデッド");
        let subjects4 = search_bangumi_with_keyword(
            &client,
            "モンスターストライク デッドバースリローデッド",
            &monster_date_range,
        )
        .await?;
        println!("搜索结果数量: {}", subjects4.len());
        for subject in &subjects4 {
            println!("  作品: {} (ID: {})", subject.name, subject.id);
        }

        Ok(())
    }
}
