mod commands;
mod models;
mod presentation;
mod service;
mod storage;

i_rs_core::define_cli_tool! {
    binary: "i-rs-mood",
    about: "Mood tracking CLI",
    add_args: {
        mood: String => "心情描述(开心/难过/焦虑/平静/兴奋/疲惫...",
        date: Option<String> => "日期(YYYY-MM-DD)",
    },
    update_args: {
        mood: Option<String> => "新心情",
        date: Option<String> => "新日期(YYYY-MM-DD)",
    },
    list_args: {
        days: Option<usize> => "最近N天",
        calendar: bool => "日历视图",
    },
}

#[cfg(test)]
mod tests;
