use super::*;
use crate::Status;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Service: Sized + Clone + Sync + Send {
    fn name(&self) -> &'static str;
    fn restart_time(&self) -> u64;
    async fn start(&self) -> Result<()>;
    async fn state_manager(&self) -> StateManager;

    async fn spawn(&self) -> Result<()>
    where
        Self: 'static,
    {
        tokio::spawn(self.clone().run_persistent());
        Ok(())
    }

    async fn run_persistent(self) -> Result<()> {
        loop {
            match self.start().await {
                Ok(_) => {}
                Err(e) => {
                    error!("{} service error: {}", self.name(), e);
                }
            };
            if self.state_manager().await.current_status().await == Status::ShuttingDown {
                break;
            }
            warn!(
                "Restarting {} service in {} seconds...",
                self.name(),
                self.restart_time()
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(self.restart_time())).await;
        }
        Ok(())
    }
}
