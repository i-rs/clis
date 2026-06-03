// User-facing strings for the TUI. All user-visible text lives here.

pub const INPUT_PLACEHOLDER: &str = "输入消息... 输入 /help 查看命令";

pub const STATUS_BAR: &str = "  [?] 快捷键  [/help] 命令  [Enter] 发送  [Esc] 退出  [Ctrl+C] 取消  [Ctrl+Z] 撤销  [Ctrl+T] 转录";

pub const SHORTCUT_TITLE: &str = "键盘快捷键";

pub const SHORTCUTS: &[(&str, &str)] = &[
    ("? / Esc", "关闭此面板"),
    ("/help", "查看所有 Slash 命令"),
    ("Enter", "发送消息"),
    ("Alt+Enter", "换行"),
    ("Esc / q", "退出"),
    ("Ctrl+C", "取消当前生成"),
    ("Ctrl+Z", "撤销文件修改"),
    ("Ctrl+T", "转录模式（完整输出）"),
    ("Ctrl+B", "HTTP 调试面板"),
    ("[ / ]", "选择上/下一条消息"),
    ("↑ / ↓ / PgUp", "滚动聊天"),
    ("Tab", "补全工具名或命令（/ 开头时）"),
    ("鼠标点击", "展开/折叠工具和思考卡片"),
];

pub fn scrolled_up_hint(count: usize) -> String {
    format!(" ↑ {} 条历史消息 ", count)
}
