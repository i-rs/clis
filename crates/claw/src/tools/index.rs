use std::collections::HashSet;

/// Tool entry with (name, description, category_label).
/// Category labels are used as section headers in the system prompt.
pub const TOOL_INDEX: &[(&str, &str, &str)] = &[
    // 健康管理
    ("weight", "体重管理：记录、查看、统计体重数据", "健康管理"),
    ("height", "身高管理：记录、查看身高数据", "健康管理"),
    ("run", "跑步记录：记录跑步、查看计划、统计数据", "健康管理"),
    ("sleep", "睡眠记录：记录睡眠时长、查看统计", "健康管理"),
    ("mood", "心情记录：记录心情、查看日历", "健康管理"),
    ("water", "饮水记录：记录饮水量", "健康管理"),
    ("step", "步数记录：记录每日步数", "健康管理"),
    ("dose", "用药记录：记录药物剂量", "健康管理"),
    ("meal", "饮食记录：记录每日饮食", "健康管理"),
    ("exercise", "运动记录：记录运动、查看统计", "健康管理"),
    ("fast", "禁食记录：记录禁食时间", "健康管理"),
    ("cycle", "生理周期：记录月经周期", "健康管理"),
    ("sit", "久坐记录：记录久坐提醒", "健康管理"),
    ("allergy", "过敏记录：记录过敏情况", "健康管理"),
    ("cal", "卡路里：估算卡路里摄入", "健康管理"),
    // 财务管理
    ("ledger", "记账：记录收支、查看流水", "财务管理"),
    ("budget", "预算管理：设置预算、查看支出统计", "财务管理"),
    ("invest", "投资跟踪：记录投资、查看收益统计", "财务管理"),
    ("debt", "债务管理：记录债务、还款、统计", "财务管理"),
    ("goal", "储蓄目标：设定目标、存款里程碑", "财务管理"),
    ("invoice", "发票管理：记录发票、统计", "财务管理"),
    ("tax", "税务记录：记录税务信息、统计", "财务管理"),
    ("recur", "定期支出：管理周期性支出", "财务管理"),
    ("sub", "订阅管理：跟踪订阅服务", "财务管理"),
    // 任务与习惯
    ("todo", "待办事项：添加、完成、列出待办", "任务与习惯"),
    ("habit", "习惯跟踪：记录习惯、打卡、连续天数", "任务与习惯"),
    ("project", "项目管理：管理项目、里程碑、统计", "任务与习惯"),
    ("time", "时间跟踪：计时、记录、报告、统计", "任务与习惯"),
    ("remind", "提醒管理：设置提醒、标记完成", "任务与习惯"),
    // 媒体与知识
    ("movie", "电影跟踪：记录看过的电影、评分统计", "媒体与知识"),
    ("podcast", "播客跟踪：记录收听的播客", "媒体与知识"),
    ("read", "阅读跟踪：记录阅读进度", "媒体与知识"),
    ("article", "文章跟踪：收藏和跟踪文章", "媒体与知识"),
    ("quote", "语录收集：收集和查看语录", "媒体与知识"),
    ("snippet", "代码片段：管理代码片段", "媒体与知识"),
    ("vocab", "词汇学习：学习单词、测验、统计", "媒体与知识"),
    ("bookmark", "书签管理：管理浏览器书签", "媒体与知识"),
    ("note", "笔记管理：记录和查看笔记", "媒体与知识"),
    // 生活记录
    ("grocery", "购物清单：管理购物清单、购买标记", "生活记录"),
    ("pig", "欲望记录：记录想吃的食物/东西", "生活记录"),
    ("want", "愿望清单：记录想要的东西", "生活记录"),
    ("gift", "礼物规划：记录礼物想法、统计", "生活记录"),
    ("birthday", "生日跟踪：记录亲友生日、统计", "生活记录"),
    ("event", "事件管理：记录重要事件、统计", "生活记录"),
    ("contact", "联系人：管理联系人、提醒联系", "生活记录"),
    // 家居清洁
    ("sheet", "床单更换：记录床单更换时间", "家居清洁"),
    ("toothbrush", "牙刷更换：记录牙刷更换", "家居清洁"),
    ("towel", "毛巾更换：记录毛巾更换", "家居清洁"),
    ("bed", "床品更换：记录床垫/枕头更换", "家居清洁"),
    ("ac", "空调清洁：记录空调清洗", "家居清洁"),
    ("filter", "滤网清洁：记录滤网清洗", "家居清洁"),
    ("purify", "净水器：记录净水器滤芯更换", "家居清洁"),
    ("appliance", "家电管理：记录家电维护、统计", "家居清洁"),
    // 宠物养护
    ("feedpet", "宠物喂食：记录宠物喂食", "宠物养护"),
    ("petbath", "宠物洗澡：记录宠物洗澡", "宠物养护"),
    ("walkdog", "遛狗记录：记录遛狗时间", "宠物养护"),
    ("aqua", "鱼缸维护：记录鱼缸维护", "宠物养护"),
    // 出行与车辆
    ("car", "车辆管理：加油、保养、统计", "出行与车辆"),
    ("cycling", "骑行记录：记录骑行、统计", "出行与车辆"),
    // 生活配置
    ("kv", "键值存储：存储和获取键值对", "生活配置"),
    ("keys", "API密钥：管理 API 密钥", "生活配置"),
    ("password", "密码管理：管理网站密码", "生活配置"),
    ("domain", "域名管理：跟踪域名到期", "生活配置"),
    ("deploy", "部署跟踪：记录部署、回滚", "生活配置"),
    ("vision", "愿景跟踪：记录愿景进展", "生活配置"),
    ("server", "服务器管理：管理服务器信息", "生活配置"),
    ("spark", "灵感收集：随时记录灵感", "生活配置"),
    ("bestby", "保质期：跟踪物品保质期", "生活配置"),
];

/// Format a compact tool index for system prompt Layer 2.
/// If `enabled` is Some, only include tools in that set (empty set = all).
/// Outputs tools grouped by category with section headers.
pub fn format_index(enabled: Option<&HashSet<String>>) -> String {
    let active: Vec<&(&str, &str, &str)> = TOOL_INDEX
        .iter()
        .filter(|(name, _, _)| is_tool_enabled(name, enabled))
        .collect();
    let mut result = String::from("## 工具索引（");
    result.push_str(&format!("{}个工具，按领域分组)\n\n", active.len()));
    let mut last_category = String::new();
    for (name, desc, category) in &active {
        if *category != last_category {
            if !last_category.is_empty() {
                result.push('\n');
            }
            result.push_str(&format!("**{}**\n", category));
            last_category = category.to_string();
        }
        result.push_str(&format!("- {}: {}\n", name, desc));
    }
    result
}

pub fn is_tool_enabled(tool: &str, enabled: Option<&HashSet<String>>) -> bool {
    match enabled {
        Some(set) if !set.is_empty() => set.contains(tool),
        _ => true,
    }
}

pub fn search(query: &str) -> String {
    let query_lower = query.to_lowercase();
    let results: Vec<String> = TOOL_INDEX
        .iter()
        .filter(|(name, desc, _)| {
            name.contains(&query_lower) || desc.to_lowercase().contains(&query_lower)
        })
        .map(|(name, desc, _)| format!("i-rs-{}: {}", name, desc))
        .collect();

    if results.is_empty() {
        return "没有找到匹配的工具，请尝试用其他关键词搜索".to_string();
    }

    results.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tool_enabled_no_filter() {
        assert!(is_tool_enabled("weight", None), "无过滤时所有工具应启用");
        assert!(is_tool_enabled("anything", None));
    }

    #[test]
    fn test_is_tool_enabled_empty_set() {
        let set = HashSet::new();
        assert!(
            is_tool_enabled("weight", Some(&set)),
            "空集合应表示启用所有工具"
        );
    }

    #[test]
    fn test_is_tool_enabled_in_set() {
        let mut set = HashSet::new();
        set.insert("weight".to_string());
        assert!(is_tool_enabled("weight", Some(&set)));
        assert!(!is_tool_enabled("mood", Some(&set)), "mood 不在启用集合中");
    }

    #[test]
    fn test_search_found() {
        let result = search("体重");
        assert!(result.contains("weight"), "搜索'体重'应找到 weight");
    }

    #[test]
    fn test_search_not_found() {
        let result = search("zzz_nonexistent_zzz");
        assert!(result.contains("没有找到"));
    }

    #[test]
    fn test_format_index_all() {
        let result = format_index(None);
        assert!(result.contains("weight"));
        assert!(result.contains("健康管理"));
        assert!(result.contains("工具索引"));
    }

    #[test]
    fn test_format_index_filtered() {
        let mut set = HashSet::new();
        set.insert("weight".to_string());
        let result = format_index(Some(&set));
        assert!(result.contains("weight"));
        assert!(!result.contains("mood"), "mood 被过滤不应出现");
    }

    #[test]
    fn test_too_index_const_entries() {
        assert!(TOOL_INDEX.len() > 50, "应有至少 50 个工具索引条目");
    }
}
