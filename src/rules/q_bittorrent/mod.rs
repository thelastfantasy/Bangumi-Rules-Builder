use crate::models::{BangumiResult, QBRule, TorrentParams, RuleGenerationResult};

fn sanitize_work_name(work_name: &str) -> String {
    let mut sanitized = work_name.to_string();

    // 替换非法文件名字符为对应的全角字符
    let illegal_chars = [
        ('/', '／'),  // 全角斜杠
        ('\\', '＼'), // 全角反斜杠
        (':', '：'),  // 全角冒号
        ('*', '＊'),  // 全角星号
        ('?', '？'),  // 全角问号
        ('"', '＂'), // 全角双引号
        ('<', '＜'),  // 全角小于号
        ('>', '＞'),  // 全角大于号
        ('|', '｜'),  // 全角竖线
    ];

    for (illegal_char, full_width_char) in illegal_chars {
        sanitized = sanitized.replace(illegal_char, &full_width_char.to_string());
    }

    // 去除开头和结尾的空格
    sanitized = sanitized.trim().to_string();

    // 如果为空字符串，使用默认名称
    if sanitized.is_empty() {
        "Unknown_Work".to_string()
    } else {
        sanitized
    }
}

pub fn generate_qb_rules(
    bangumi_results: &[BangumiResult],
    root_path: &str,
    season_name: &str,
) -> Result<RuleGenerationResult, Box<dyn std::error::Error>> {
    let mut rules = serde_json::Map::new();
    let mut failed_works = Vec::new();

    for result in bangumi_results {
        // 确定作品名称：优先使用中文名称，如果没有则使用清理后的日文标题
        let work_name = if let Some(chinese_name) = &result.chinese_name {
            chinese_name.clone()
        } else {
            result.cleaned_title.clone()
        };

        // 构建名称模式：对于有Bangumi信息的作品，使用中文名称和别名
        // 对于没有Bangumi信息的作品，使用AI生成的关键字
        let mut all_names = vec![work_name.clone()];

        if result.bangumi_id.is_some() {
            // 有Bangumi信息：使用中文名称和别名
            all_names.extend(result.aliases.clone());
            // 添加清理后的日文标题，提高匹配准确性
            if work_name != result.cleaned_title {
                all_names.push(result.cleaned_title.clone());
            }
        } else {
            // 没有Bangumi信息：使用AI生成的关键字
            all_names.extend(result.keywords.clone());
        }

        // 去重并转义特殊字符
        let mut unique_names: Vec<String> = all_names.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect();
        unique_names.sort();
        let escaped_names: Vec<String> = unique_names.iter().map(|name| regex::escape(name)).collect();

        // 构建mustContain模式
        let name_pattern = escaped_names.join("|");
        let must_contain = format!(
            ".+({}).+((1080|2160|WebRip).+(CHS|CHT|GB|BIG5|简|繁|B-Global|Baha|bilibili|CR|Sentai|x264\\sAAC|无字幕)|(CHS|CHT|GB|BIG5|简|繁|B-Global|Baha|bilibili|CR|Sentai|x264\\sAAC|无字幕).+(1080|2160|WebRip)).+",
            name_pattern
        );

        // 构建mustNotContain模式
        let must_not_contain = ".+01\\-.+|.+合集.+|.+先行.+|.+\\[V0.+|.+全集.+".to_string();

        // 清理作品名称中的非法字符
        let sanitized_work_name = sanitize_work_name(&work_name);

        // 构建保存路径
        let save_path = format!(r"{}\{}\{}", root_path, season_name, sanitized_work_name);
        let save_path_forward = save_path.replace("\\", "/");

        // 创建规则
        let rule = QBRule {
            add_paused: None,
            affected_feeds: vec![
                "https://acg.rip/1.xml".to_string(),
                "https://nyaa.si/?page=rss&c=1_4".to_string(),
                "https://acg.rip/page/6.xml?term=jibaketa+kiratto]".to_string(),
                "https://share.dmhy.org/topics/rss/sort_id/2/rss.xml".to_string(),
            ],
            assigned_category: format!("Anime/{}", season_name),
            enabled: true,
            episode_filter: "".to_string(),
            ignore_days: 0,
            last_match: "05 Aug 2025 16:59:34 +0000".to_string(),
            must_contain: must_contain,
            must_not_contain: must_not_contain,
            previously_matched_episodes: vec![],
            priority: 0,
            save_path: save_path.clone(),
            smart_filter: false,
            torrent_content_layout: None,
            torrent_params: TorrentParams {
                category: format!("Anime/{}", season_name),
                download_limit: -1,
                download_path: "".to_string(),
                inactive_seeding_time_limit: -2,
                operating_mode: "AutoManaged".to_string(),
                ratio_limit: -2,
                save_path: save_path_forward,
                seeding_time_limit: -2,
                share_limit_action: "Default".to_string(),
                skip_checking: false,
                ssl_certificate: "".to_string(),
                ssl_dh_params: "".to_string(),
                ssl_private_key: "".to_string(),
                tags: vec![],
                upload_limit: -1,
                use_auto_tmm: false,
            },
            use_regex: true,
        };

        // 使用规则名称作为键
        let rule_name = format!("{} {}", season_name, work_name);

        // 尝试生成规则，捕获可能的错误
        match serde_json::to_value(rule) {
            Ok(rule_value) => {
                rules.insert(rule_name, rule_value);
            }
            Err(e) => {
                failed_works.push((work_name, format!("序列化失败: {}", e)));
            }
        }
    }

    Ok(RuleGenerationResult {
        rules: serde_json::Value::Object(rules),
        failed_works,
    })
}