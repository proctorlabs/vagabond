#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate derive_more;

#[macro_use]
extern crate yarte;

#[macro_use]
mod macros;

mod app;
mod args;
mod bus;
mod config;
mod services;
mod state;
mod system;

use anyhow::Result;
use config::VagabondConfig;
use flexi_logger::{AdaptiveFormat, Logger};
use state::Status;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = args::CliArgs::new();
    Logger::try_with_str(&args.log_level.as_str())?
        .adaptive_format_for_stderr(AdaptiveFormat::Default)
        .set_palette("196;208;31;8;59".into())
        .start()?;
    let config = VagabondConfig::from_file(args.config).await?;
    let mut vagabond = app::Vagabond::new(config).await?;
    match vagabond.start().await {
        Ok(_) => {}
        Err(e) => {
            error!("Failed: {}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}
