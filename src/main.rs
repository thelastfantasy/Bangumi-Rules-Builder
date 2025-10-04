use std::fs;

mod ai;
mod logger;
mod meta_providers;
mod models;
mod rules;
mod sites;
mod utils;

use crate::models::Task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    logger::init_default()?;
    log::info!("Bangumi规则生成器启动");

    // 读取tasks.json文件
    let json_content = match fs::read_to_string("tasks.json") {
        Ok(content) => {
            log::info!("成功读取配置文件 tasks.json");
            content
        }
        Err(e) => {
            log::error!("无法读取配置文件 tasks.json: {}", e);
            return Err(e.into());
        }
    };

    let task: Task = match serde_json::from_str(&json_content) {
        Ok(task) => {
            log::info!("成功解析配置文件");
            task
        }
        Err(e) => {
            log::error!("配置文件格式错误: {}", e);
            return Err(e.into());
        }
    };

    log::info!("任务描述: {}", task.description);
    log::info!("站点: {}", task.site);

    match task.site {
        models::SiteType::Kansou => {
            log::info!("开始处理kansou站点...");
            sites::kansou::process_kansou_site(&task).await?;
            log::info!("kansou站点处理完成");
        } // 未来添加其他站点支持
          // models::SiteType::ModelScope => {
          //     log::info!("开始处理modelscope站点...");
          //     sites::modelscope::process_modelscope_site(&task).await?;
          //     log::info!("modelscope站点处理完成");
          // }
          // models::SiteType::AnimeList => {
          //     log::info!("开始处理animelist站点...");
          //     sites::animelist::process_animelist_site(&task).await?;
          //     log::info!("animelist站点处理完成");
          // }
    }

    log::info!("Bangumi规则生成器运行完成");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::meta_providers::bangumi::{search_bangumi_for_works, search_bangumi_with_keyword};
    use crate::models::AnimeWork;
    use crate::logger;

    #[tokio::test]
    async fn test_specific_work() -> Result<(), Box<dyn std::error::Error>> {
        log::info!("🧪 测试特定作品的Bangumi搜索...");

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

        log::info!("测试作品: {}", test_work.cleaned_title);
        log::info!("关键词数量: {}", test_work.keywords.len());
        log::debug!("关键词列表: {:?}", test_work.keywords);

        // 测试搜索
        let results = search_bangumi_for_works(&[test_work]).await?;

        if let Some(result) = results.first() {
            log::debug!("搜索结果: {:?}", result);
            if result.bangumi_id.is_some() {
                log::info!("✅ 成功找到Bangumi信息!");
                log::info!("   Bangumi ID: {}", result.bangumi_id.unwrap());
                log::info!("   中文名称: {:?}", result.chinese_name);
                log::info!("   别名: {:?}", result.aliases);
            } else {
                log::warn!("❌ 未找到Bangumi信息");
                log::debug!("⚠️ 问题分析:");
                log::debug!("   - 关键词测试显示 '青のミブロ' 能找到作品 (ID: 454630)");
                log::debug!("   - 但完整作品搜索时没有匹配成功");
                log::debug!("   - 可能原因: 匹配阈值过高或日期过滤问题");
            }
        } else {
            log::warn!("❌ 没有搜索结果");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_specific_keywords() -> Result<(), Box<dyn std::error::Error>> {
        log::info!("🧪 测试特定关键词的Bangumi搜索...");
        let client = reqwest::Client::new();

        // 测试 "青のミブロ" 关键词
        log::info!("📝 测试关键词: 青のミブロ 第二期 芹沢暗殺編");
        let subjects =
            search_bangumi_with_keyword(&client, "青のミブロ 第二期 芹沢暗殺編", &None).await?;
        if !subjects.is_empty() {
            let subject = &subjects[0];
            log::info!("✅ 成功找到作品: {}", subject.name);
            log::info!("   Bangumi ID: {}", subject.id);
            log::info!("   中文名称: {}", subject.name_cn);
        } else {
            log::warn!("❌ 未找到作品");
        }

        // 测试 "SHIBUYA HACHI" 关键词
        log::info!("📝 测试关键词: SHIBUYA HACHI 第4クール");
        let subjects =
            search_bangumi_with_keyword(&client, "SHIBUYA HACHI 第4クール", &None).await?;
        if !subjects.is_empty() {
            let subject = &subjects[0];
            log::info!("✅ 成功找到作品: {}", subject.name);
            log::info!("   Bangumi ID: {}", subject.id);
            log::info!("   中文名称: {}", subject.name_cn);
        } else {
            log::warn!("❌ 未找到作品");
        }

        // 测试 "Ao no Miburo" 关键词
        log::info!("📝 测试关键词: Ao no Miburo");
        let subjects = search_bangumi_with_keyword(&client, "Ao no Miburo", &None).await?;
        if !subjects.is_empty() {
            let subject = &subjects[0];
            log::info!("✅ 成功找到作品: {}", subject.name);
            log::info!("   Bangumi ID: {}", subject.id);
            log::info!("   中文名称: {}", subject.name_cn);
        } else {
            log::warn!("❌ 未找到作品");
        }

        // 测试 "SANDA" 关键词
        log::info!("📝 测试关键词: SANDA");
        let subjects = search_bangumi_with_keyword(&client, "SANDA", &None).await?;
        if !subjects.is_empty() {
            let subject = &subjects[0];
            log::info!("✅ 成功找到作品: {}", subject.name);
            log::info!("   Bangumi ID: {}", subject.id);
            log::info!("   中文名称: {}", subject.name_cn);
        } else {
            log::warn!("❌ 未找到作品");
        }

        // 测试 "SANDA サンダ" 关键词
        log::info!("📝 测试关键词: SANDA サンダ");
        let subjects = search_bangumi_with_keyword(&client, "SANDA サンダ", &None).await?;
        if !subjects.is_empty() {
            let subject = &subjects[0];
            log::info!("✅ 成功找到作品: {}", subject.name);
            log::info!("   Bangumi ID: {}", subject.id);
            log::info!("   中文名称: {}", subject.name_cn);
        } else {
            log::warn!("❌ 未找到作品");
        }

        // 测试 "SANDA" 关键词带日期过滤
        log::info!("📝 测试关键词: SANDA (带日期过滤 2025-10-03)");
        let sanda_date = chrono::NaiveDate::from_ymd_opt(2025, 10, 3);
        let subjects = search_bangumi_with_keyword(&client, "SANDA", &sanda_date).await?;
        if !subjects.is_empty() {
            let subject = &subjects[0];
            log::info!("✅ 成功找到作品: {}", subject.name);
            log::info!("   Bangumi ID: {}", subject.id);
            log::info!("   中文名称: {}", subject.name_cn);
        } else {
            log::warn!("❌ 未找到作品 - 日期过滤可能太严格");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_without_date_filter() -> Result<(), Box<dyn std::error::Error>> {
        log::info!("🧪 测试无日期过滤的Bangumi搜索...");

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

        log::info!("测试作品: {}", test_work.cleaned_title);
        log::info!("关键词数量: {}", test_work.keywords.len());
        log::info!("无日期过滤");

        // 测试搜索
        let results = search_bangumi_for_works(&[test_work]).await?;

        if let Some(result) = results.first() {
            log::debug!("搜索结果: {:?}", result);
            if result.bangumi_id.is_some() {
                log::info!("✅ 成功找到Bangumi信息!");
                log::info!("   Bangumi ID: {}", result.bangumi_id.unwrap());
                log::info!("   中文名称: {:?}", result.chinese_name);
                log::info!("   别名: {:?}", result.aliases);
            } else {
                log::warn!("❌ 未找到Bangumi信息");
            }
        } else {
            log::warn!("❌ 没有搜索结果");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_ai_individual_matching() -> Result<(), Box<dyn std::error::Error>> {
        log::info!("🧪 测试AI单个匹配...");

        use crate::ai::object_matcher::{CandidateWork, batch_match_works_with_ai};
        use crate::meta_providers::bangumi::search_bangumi_for_works;
        use crate::models::{AiConfig, AnimeWork};

        let ai_config = AiConfig::deepseek();

        // 测试案例1: 破产富豪
        log::info!("📝 测试案例1: 破产富豪");
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

        log::info!("找到 {} 个候选作品", candidate_works1.len());
        let result1 =
            batch_match_works_with_ai(&[&anime_work1], &[&candidate_works1], &ai_config).await?;
        let result1 = result1.first().copied().flatten();
        log::debug!("匹配结果: {:?}", result1);
        log::debug!("预期结果: None");
        assert_eq!(result1, None, "破产富豪应该匹配不到任何结果");
        log::info!("✅ 匹配结果符合预期");

        // 测试案例
        log::info!("📝 测试案例");
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

        log::info!("找到 {} 个候选作品", candidate_works2.len());
        let result2 =
            batch_match_works_with_ai(&[&anime_work2], &[&candidate_works2], &ai_config).await?;
        let result2 = result2.first().copied().flatten();
        log::debug!("匹配结果: {:?}", result2);
        log::debug!("预期结果: Some(434807)");
        assert_eq!(
            result2,
            Some(434807),
            "某天成为公主应该匹配到魔法公主的小烦恼 (ID: 434807)"
        );
        log::info!("✅ 匹配结果符合预期");

        // 测试案例3: 罗小黑战记
        log::info!("📝 测试案例3: 罗小黑战记");
        let anime_work3 = AnimeWork {
            original_title: "羅小黒戦記".to_string(),
            cleaned_title: "罗小黑战记".to_string(),
            air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 1),
            keywords: vec![
                "罗小黑战记".to_string(),
                "The Legend of Luo Xiao Hei".to_string(),
            ],
        }; // 该数据应该匹配不到任何结果，但如果放大日期范围到100天会匹配到442114 （放送日本：2025-07-18）

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

        log::info!("找到 {} 个候选作品", candidate_works3.len());
        let result3 =
            batch_match_works_with_ai(&[&anime_work3], &[&candidate_works3], &ai_config).await?;
        let result3 = result3.first().copied().flatten();
        log::debug!("匹配结果: {:?}", result3);
        log::debug!("预期结果: Some(442114)");
        assert_eq!(result3, Some(442114), "罗小黑战记应该匹配到 (ID: 442114)");
        log::info!("✅ 匹配结果符合预期");

        // 测试案例4: 异世界四重奏3
        log::info!("📝 测试案例4: 异世界四重奏3");
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

        log::info!("找到 {} 个候选作品", candidate_works4.len());
        let result4 =
            batch_match_works_with_ai(&[&anime_work4], &[&candidate_works4], &ai_config).await?;
        let result4 = result4.first().copied().flatten();
        log::debug!("匹配结果: {:?}", result4);
        log::debug!("预期结果: Some(564421)");
        assert_eq!(
            result4,
            Some(564421),
            "异世界四重奏3应该匹配到异世界四重奏 第三季 (ID: 564421)"
        );
        log::info!("✅ 匹配结果符合预期");

        // 测试案例5: 怪物弹珠 Dead Death Reloaded
        log::info!("📝 测试案例5: 怪物弹珠 Dead Death Reloaded");
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

        log::info!("找到 {} 个候选作品", candidate_works5.len());
        let result5 =
            batch_match_works_with_ai(&[&anime_work5], &[&candidate_works5], &ai_config).await?;
        let result5 = result5.first().copied().flatten();
        log::debug!("匹配结果: {:?}", result5);
        log::debug!("预期结果: Some(570330)");
        assert_eq!(
            result5,
            Some(570330),
            "怪物弹珠 Dead Death Reloaded应该匹配到怪物弹珠 DEADVERSE RELOADED (ID: 570330)"
        );
        log::info!("✅ 匹配结果符合预期");

        Ok(())
    }

    #[tokio::test]
    async fn test_ai_batch_matching() -> Result<(), Box<dyn std::error::Error>> {
        log::info!("\n🧪 测试AI批量匹配...");

        use crate::meta_providers::bangumi::search_bangumi_for_works;
        use crate::models::AnimeWork;

        // 使用关键测试案例，包含边界情况和复杂匹配场景
        let source_works = vec![
            // 基础测试案例 - 确保基本功能正常
            AnimeWork {
                original_title: "SHIBUYA♡HACHI 第4クール".to_string(),
                cleaned_title: "SHIBUYA♡HACHI 第4クール".to_string(),
                air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 4),
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
                air_date: chrono::NaiveDate::from_ymd_opt(2017, 7, 3),
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
            }, // 该数据应该匹配不到任何结果，但如果放大日期范围到100天的话会匹配到442114 （放送日本：2025-07-18）
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
            // 新增测试案例 - 2025年10月新番
            AnimeWork {
                original_title: "ガングリオン".to_string(),
                cleaned_title: "ガングリオン".to_string(),
                air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 3),
                keywords: vec![
                    "ガングリオン".to_string(),
                    "Ganglion".to_string(),
                    "Ganglion anime".to_string(),
                    "神经节".to_string(),
                    "Ganglion 2025".to_string(),
                    "Ganglion new anime".to_string(),
                ],
            }, // 该数据应该匹配到ガングリオン，bangumi_id: 581598
            AnimeWork {
                original_title: "SANDA【サンダ】".to_string(),
                cleaned_title: "SANDA".to_string(),
                air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 3),
                keywords: vec![
                    "SANDA".to_string(),
                    "SANDA サンダ".to_string(),
                    "SANDA anime".to_string(),
                    "三太".to_string(),
                    "SANDA 2025".to_string(),
                    "Sanda new series".to_string(),
                ],
            }, // 该数据应该匹配到SANDA，bangumi_id: 503303
            AnimeWork {
                original_title: "信じていた仲間達にダンジョン奥地で殺されかけたがギフト『無限ガチャ』でレベル9999の仲間達を手に入れて元パーティーメンバーと世界に復讐＆『ざまぁ！』します！".to_string(),
                cleaned_title: "信じていた仲間達にダンジョン奥地で殺されかけたがギフト『無限ガチャ』でレベル9999の仲間達を手に入れて元パーティーメンバーと世界に復讐＆『ざまぁ！』します！".to_string(),
                air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 3),
                keywords: vec![
                    "信じていた仲間達にダンジョン奥地で殺されかけたがギフト 無限ガチャ でレベル9999の仲間達を手に入れて元パーティーメンバーと世界に復讐 ざまぁ します".to_string(),
                    "被信任的同伴在迷宫深处杀害但获得无限扭蛋".to_string(),
                    "Shinjiteita Nakamatachi ni Dungeon Okuchi de Korosarekaketa ga Gift Mugen Gacha de Level 9999 no Nakamatachi wo Te ni Irete Moto Party Member to Sekai ni Fukushuu Zamaa Shimasu".to_string(),
                    "无限扭蛋复仇记".to_string(),
                    "Mugen Gacha revenge anime".to_string(),
                    "Level 9999 companions revenge".to_string(),
                ],
            }, // 该数据应该匹配到信じていた仲間達にダンジョン奥地で殺されかけたがギフト『無限ガチャ』でレベル9999の仲間達を手に入れて元パーティーメンバーと世界に復讐＆『ざまぁ！』します！，bangumi_id: 524195
            AnimeWork {
                original_title: "最後にひとつだけお願いしてもよろしいでしょうか".to_string(),
                cleaned_title: "最後にひとつだけお願いしてもよろしいでしょうか".to_string(),
                air_date: chrono::NaiveDate::from_ymd_opt(2025, 10, 3),
                keywords: vec![
                    "最後にひとつだけお願いしてもよろしいでしょうか".to_string(),
                    "最后能再拜托您一件事吗".to_string(),
                    "Saigo ni Hitotsu dake Onegai shitemo Yoroshii deshou ka".to_string(),
                    "Saigo ni Hitotsu dake Onegai shitemo Yoroshii deshou ka anime".to_string(),
                    "最后一个请求".to_string(),
                    "One Last Request anime".to_string(),
                ],
            }, // 该数据应该匹配到最后にひとつだけお願いしてもよろしいでしょうか，bangumi_id: 513348
        ];

        log::info!("准备批量测试数据，共 {} 个作品", source_works.len());
        for (i, work) in source_works.iter().enumerate() {
            log::info!(
                "  作品{}: {} (关键词: {:?})",
                i, work.cleaned_title, work.keywords
            );
        }

        // 使用search_bangumi_for_works获取所有Bangumi匹配结果
        log::info!("\n🚀 执行Bangumi搜索和AI匹配...");
        let bangumi_results = search_bangumi_for_works(&source_works).await?;

        // 直接从Bangumi结果中提取匹配的Bangumi ID
        let batch_results: Vec<Option<u32>> = bangumi_results
            .iter()
            .map(|result| result.bangumi_id)
            .collect();

        // 第三步：验证匹配结果是否符合预期
        log::info!("\n📊 AI批量匹配结果验证:");
        let mut test_passed = true;

        // 预期结果映射
        let expected_results = vec![
            Some(582915), // 作品0: SHIBUYA♡HACHI - 应该匹配到582915
            Some(192252), // 作品1: 异世界食堂 - 应该匹配到192252
            None,         // 作品2: 破产富豪 - 应该匹配不到任何结果
            Some(434807), // 作品3: 某天成为公主 - 应该匹配魔法公主的小烦恼，bangumi_id: 434807
            None,         // 作品4: 虚构的动画作品 - 应该匹配不到任何结果
            Some(442114), // 作品5: 罗小黑战记 - 由于日期范围放宽到100天，现在匹配到442114 （放送日本：2025-07-18）
            Some(564421), // 作品6: 异世界四重奏3 - 应该匹配到564421
            Some(570330), // 作品7: 怪物弹珠 Dead Death Reloaded - 应该匹配到570330
            Some(481530), // 作品8: ポケモンコンシェルジュ【2nd Season】 - 应该匹配到宝可梦 礼宾部 新剧集，bangumi_id: 481530
            Some(581598), // 作品9: ガングリオン - 应该匹配到581598
            Some(503303), // 作品10: SANDA - 应该匹配到503303
            Some(524195), // 作品11: 信じていた仲間達にダンジョン奥地で殺されかけたがギフト『無限ガチャ』でレベル9999の仲間達を手に入れて元パーティーメンバーと世界に復讐＆『ざまぁ！』します！ - 应该匹配到524195
            Some(513348), // 作品12: 最後にひとつだけお願いしてもよろしいでしょうか - 应该匹配到513348
        ];

        for (i, (result, expected)) in batch_results
            .iter()
            .zip(expected_results.iter())
            .enumerate()
        {
            let work = &source_works[i];
            log::info!("\n  作品{}: '{}'", i, work.cleaned_title);
            log::info!("    预期结果: {:?}", expected);
            log::info!("    实际结果: {:?}", result);

            if result == expected {
                log::info!("    ✅ 匹配结果符合预期");
                if let Some(bangumi_id) = result {
                    log::info!("      匹配到Bangumi ID: {}", bangumi_id);
                }
            } else {
                log::info!("    ❌ 匹配结果不符合预期");
                test_passed = false;
            }
        }

        // 输出整体测试结果
        log::info!("\n📈 批量匹配测试总结:");
        log::info!("   总作品数: {}", source_works.len());
        log::info!(
            "   预期匹配: {} 个作品",
            expected_results.iter().filter(|r| r.is_some()).count()
        );
        log::info!(
            "   实际匹配: {} 个作品",
            batch_results.iter().filter(|r| r.is_some()).count()
        );

        if test_passed {
            log::info!("   ✅ 所有匹配结果都符合预期，测试通过！");
        } else {
            log::info!("   ❌ 部分匹配结果不符合预期，测试失败！");
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

        log::info!("\n🔍 测试搜索: 異世界かるてっと3");
        let subjects =
            search_bangumi_with_keyword(&client, "異世界かるてっと3", &isekai_date_range).await?;
        log::info!("搜索结果数量: {}", subjects.len());
        for subject in &subjects {
            log::info!("  作品: {} (ID: {})", subject.name, subject.id);
        }

        log::info!("\n🔍 测试搜索: 異世界かるてっと");
        let subjects2 =
            search_bangumi_with_keyword(&client, "異世界かるてっと", &isekai_date_range).await?;
        log::info!("搜索结果数量: {}", subjects2.len());
        for subject in &subjects2 {
            log::info!("  作品: {} (ID: {})", subject.name, subject.id);
        }

        log::info!("\n🔍 测试搜索: 异世界四重奏");
        let subjects3 =
            search_bangumi_with_keyword(&client, "异世界四重奏", &isekai_date_range).await?;
        log::info!("搜索结果数量: {}", subjects3.len());
        for subject in &subjects3 {
            log::info!("  作品: {} (ID: {})", subject.name, subject.id);
        }

        log::info!("\n🔍 测试搜索: モンスターストライク デッドバースリローデッド");
        let subjects4 = search_bangumi_with_keyword(
            &client,
            "モンスターストライク デッドバースリローデッド",
            &monster_date_range,
        )
        .await?;
        log::info!("搜索结果数量: {}", subjects4.len());
        for subject in &subjects4 {
            log::info!("  作品: {} (ID: {})", subject.name, subject.id);
        }

        Ok(())
    }

    #[test]
    fn test_logger_functionality() -> Result<(), Box<dyn std::error::Error>> {
        // 测试日志系统功能
        log::info!("\n🧪 测试日志系统功能...");

        // 测试初始化
        logger::init_default()?;

        // 测试各种日志输出
        log::debug!("调试信息测试");
        log::info!("普通信息测试");
        log::warn!("警告信息测试");
        log::error!("错误信息测试");

        log::info!("✅ 日志系统功能测试完成");
        Ok(())
    }
}
