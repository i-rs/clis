mod commands;
mod models;
mod presentation;
mod service;
mod storage;

i_rs_core::define_cli_tool! {
    binary: "i-rs-weight",
    about: "Weight tracking CLI",
    add_args: {
        weight: f64 => "体重值(kg)",
        date: Option<String> => "日期(YYYY-MM-DD)",
    },
    update_args: {
        weight: Option<f64> => "体重值(kg)",
    },
    list_args: {
        days: Option<usize> => "最近N天",
        chart: bool => "显示ASCII图表",
        stats: bool => "体重统计",
    },
}

#[cfg(test)]
mod tests;
