use crate::{state::StateManager, util::run_command};
use anyhow::Result;

config_file! { WireguardConfigTemplate("wireguard.conf.hbs") => "/etc/wireguard/wireguard.conf" }

#[derive(Debug)]
pub struct Wireguard(pub StateManager);

impl Wireguard {
    pub async fn setup(&self) -> Result<()> {
        self.write_config().await?;
        match self.up().await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to initialize wireguard due to failure: {:?}", e);
            }
        };
        Ok(())
    }

    pub async fn write_config(&self) -> Result<()> {
        let cfg = &self.0.config.wireguard;
        if cfg.enabled {
            WireguardConfigTemplate::write_to(
                &format!("/etc/wireguard/{}.conf", &cfg.interface),
                self.0.config.clone(),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn up(&self) -> Result<()> {
        self.down().await.unwrap_or_default();
        run_command(
            "wg-quick",
            ["up", self.0.config.wireguard.interface.as_str()],
        )
        .await
    }

    pub async fn down(&self) -> Result<()> {
        run_command(
            "wg-quick",
            ["down", self.0.config.wireguard.interface.as_str()],
        )
        .await
    }

    // Commented placeholders for planned functionality...
    // pub async fn genkey(&self) -> Result<String> {
    //     Ok("".into())
    // }

    // pub async fn genpsk(&self) -> Result<String> {
    //     Ok("".into())
    // }

    // pub async fn pubkey(&self) -> Result<String> {
    //     Ok("".into())
    // }
}
