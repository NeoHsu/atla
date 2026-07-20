use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Set a config key (run `atla config list` to see keys)
    Set { key: String, value: String },
    /// Print one config value
    Get { key: String },
    /// Print all config entries
    List,
}
