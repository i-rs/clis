// User-facing strings for the TUI. All user-visible text lives here.

pub const INPUT_PLACEHOLDER: &str = "输入消息...";

pub const STATUS_BAR: &str = "  [?] 键盘快捷键  [Enter] 发送  [Esc] 退出  [Ctrl+C] 取消  [Ctrl+Z] 撤销  [Ctrl+T] 转录  [[] []] 选择";

pub const SHORTCUT_TITLE: &str = "键盘快捷键";

pub const SHORTCUTS: &[(&str, &str)] = &[
    ("? / Esc", "关闭此面板"),
    ("Enter", "发送消息"),
    ("Alt+Enter", "换行"),
    ("Esc / q", "退出"),
    ("Ctrl+C", "取消当前生成"),
    ("Ctrl+Z", "撤销文件修改"),
    ("Ctrl+T", "转录模式（完整输出）"),
    ("Ctrl+D", "HTTP 调试面板"),
    ("[ / ]", "选择上/下一条消息"),
    ("r", "展开/折叠选中消息的思考过程"),
    ("↑ / ↓ / PgUp", "滚动聊天"),
    ("Tab", "工具名补全"),
];

pub const REASONING_VISIBLE: &str = " ▼ 思考过程（按 r 折叠）";
pub const REASONING_HIDDEN: &str = " ▶ 思考过程（按 r 展开）";

pub fn scrolled_up_hint(count: usize) -> String {
    format!(" ↑ {} 条历史消息 ", count)
}
