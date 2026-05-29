use crate::config::Config;

pub fn show_version(config: &Config) {
    println!("i-rs-code {}", env!("CARGO_PKG_VERSION"));
    println!("Provider:  {}", config.provider);
    println!("Model:     {}", config.effective_model());
    println!("Config:    {}", crate::config::config_path().display());
    println!("Data dir:  {}", crate::config::i_rs_code_dir().display());
    if let Some(ref ws) = config.workspace {
        println!("Workspace: {}", ws);
    }
}
