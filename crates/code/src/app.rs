use crate::config::Config;

pub struct App {
    pub config: Config,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}
