use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "vagabond", rename_all = "kebab_case")]
pub struct CliArgs {
    #[structopt(short, long, default_value = "/etc/vagabond.toml")]
    /// Vagabond configuration file
    pub config: PathBuf,

    #[structopt(short, long, default_value = "info")]
    /// Log level to use [trace, debug, info, warn, error]
    pub log_level: flexi_logger::LevelFilter,
}

impl CliArgs {
    pub fn new() -> Self {
        CliArgs::from_args()
    }
}
