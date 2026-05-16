use std::collections::HashSet;

pub const TOOL_INDEX: &[(&str, &str)] = &[
    // 健康管理
    ("weight", "体重管理：记录、查看、统计体重数据"),
    ("height", "身高管理：记录、查看身高数据"),
    ("run", "跑步记录：记录跑步、查看计划、统计数据"),
    ("sleep", "睡眠记录：记录睡眠时长、查看统计"),
    ("mood", "心情记录：记录心情、查看日历"),
    ("water", "饮水记录：记录饮水量"),
    ("step", "步数记录：记录每日步数"),
    ("dose", "用药记录：记录药物剂量"),
    ("meal", "饮食记录：记录每日饮食"),
    ("exercise", "运动记录：记录运动、查看统计"),
    ("fast", "禁食记录：记录禁食时间"),
    ("cycle", "生理周期：记录月经周期"),
    ("sit", "久坐记录：记录久坐提醒"),
    ("allergy", "过敏记录：记录过敏情况"),
    ("cal", "卡路里：估算卡路里摄入"),
    // 财务管理
    ("ledger", "记账：记录收支、查看流水"),
    ("budget", "预算管理：设置预算、查看支出统计"),
    ("invest", "投资跟踪：记录投资、查看收益统计"),
    ("debt", "债务管理：记录债务、还款、统计"),
    ("goal", "储蓄目标：设定目标、存款里程碑"),
    ("invoice", "发票管理：记录发票、统计"),
    ("tax", "税务记录：记录税务信息、统计"),
    ("recur", "定期支出：管理周期性支出"),
    ("sub", "订阅管理：跟踪订阅服务"),
    // 任务与习惯
    ("todo", "待办事项：添加、完成、列出待办"),
    ("habit", "习惯跟踪：记录习惯、打卡、连续天数"),
    ("project", "项目管理：管理项目、里程碑、统计"),
    ("time", "时间跟踪：计时、记录、报告、统计"),
    ("remind", "提醒管理：设置提醒、标记完成"),
    // 媒体与知识
    ("movie", "电影跟踪：记录看过的电影、评分统计"),
    ("podcast", "播客跟踪：记录收听的播客"),
    ("read", "阅读跟踪：记录阅读进度"),
    ("article", "文章跟踪：收藏和跟踪文章"),
    ("quote", "语录收集：收集和查看语录"),
    ("snippet", "代码片段：管理代码片段"),
    ("vocab", "词汇学习：学习单词、测验、统计"),
    ("bookmark", "书签管理：管理浏览器书签"),
    ("note", "笔记管理：记录和查看笔记"),
    // 生活记录
    ("grocery", "购物清单：管理购物清单、购买标记"),
    ("pig", "欲望记录：记录想吃的食物/东西"),
    ("want", "愿望清单：记录想要的东西"),
    ("gift", "礼物规划：记录礼物想法、统计"),
    ("birthday", "生日跟踪：记录亲友生日、统计"),
    ("event", "事件管理：记录重要事件、统计"),
    ("contact", "联系人：管理联系人、提醒联系"),
    // 家居清洁
    ("sheet", "床单更换：记录床单更换时间"),
    ("toothbrush", "牙刷更换：记录牙刷更换"),
    ("towel", "毛巾更换：记录毛巾更换"),
    ("bed", "床品更换：记录床垫/枕头更换"),
    ("ac", "空调清洁：记录空调清洗"),
    ("filter", "滤网清洁：记录滤网清洗"),
    ("purify", "净水器：记录净水器滤芯更换"),
    ("appliance", "家电管理：记录家电维护、统计"),
    // 宠物养护
    ("feedpet", "宠物喂食：记录宠物喂食"),
    ("petbath", "宠物洗澡：记录宠物洗澡"),
    ("walkdog", "遛狗记录：记录遛狗时间"),
    ("aqua", "鱼缸维护：记录鱼缸维护"),
    // 出行与车辆
    ("car", "车辆管理：加油、保养、统计"),
    ("cycling", "骑行记录：记录骑行、统计"),
    // 生活配置
    ("kv", "键值存储：存储和获取键值对"),
    ("keys", "API密钥：管理 API 密钥"),
    ("password", "密码管理：管理网站密码"),
    ("domain", "域名管理：跟踪域名到期"),
    ("deploy", "部署跟踪：记录部署、回滚"),
    ("vision", "愿景跟踪：记录愿景进展"),
    ("server", "服务器管理：管理服务器信息"),
    ("spark", "灵感收集：随时记录灵感"),
    ("bestby", "保质期：跟踪物品保质期"),
];

/// Format a compact tool index for system prompt Layer 2.
/// If `enabled` is Some, only include tools in that set (empty set = all).
pub fn format_index(enabled: Option<&HashSet<String>>) -> String {
    let active: Vec<&(&str, &str)> = TOOL_INDEX.iter()
        .filter(|(name, _)| is_tool_enabled(name, enabled))
        .collect();
    let mut result = String::from("## 工具索引（");
    result.push_str(&format!("{}个工具)\n\n", active.len()));
    for (name, desc) in active {
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
        .filter(|(name, desc)| {
            name.contains(&query_lower) || desc.to_lowercase().contains(&query_lower)
        })
        .map(|(name, desc)| format!("i-rs-{}: {}", name, desc))
        .collect();

    if results.is_empty() {
        return "没有找到匹配的工具，请尝试用其他关键词搜索".to_string();
    }

    results.join("\n")
}
