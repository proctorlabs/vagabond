use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant, Display)]
pub enum ServiceStatus {
    Stopped,
    Running,
    Failed,
}

#[async_trait]
pub trait Service: Sized + Clone + Sync + Send {
    fn name(&self) -> &'static str;
    fn restart_time(&self) -> u64;
    async fn start(&self) -> Result<()>;

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
            if crate::state::status().await == crate::Status::ShuttingDown {
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
