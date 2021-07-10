use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{app::Vagabond, config::VagabondConfig};

#[derive(Debug, Clone, Deref)]
pub struct StateManager(Arc<StateManagerInner>);

#[derive(Debug, Clone)]
pub struct StateManagerInner {
    pub config: VagabondConfig,
    app: Arc<RwLock<Option<Vagabond>>>,
    status: Arc<RwLock<Status>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant, Display)]
pub enum Status {
    Starting,
    Running,
    ShuttingDown,
}

impl StateManager {
    pub async fn new(config: VagabondConfig) -> Result<Self> {
        Ok(Self(Arc::new(StateManagerInner {
            status: Arc::new(RwLock::new(Status::Starting)),
            app: Arc::new(RwLock::new(None)),
            config,
        })))
    }

    async fn app_available(&self) -> bool {
        self.app.read().await.is_some()
    }

    pub async fn vagabond(&self) -> Result<Vagabond> {
        if self.app_available().await {
            Ok(self.app.read().await.as_ref().unwrap().clone())
        } else {
            anyhow::bail!("Vagabond instance not yet available!")
        }
    }

    pub async fn set_app_instance(&self, app: Vagabond) -> Result<()> {
        if self.app_available().await {
            anyhow::bail!("App instance already initialized!")
        } else {
            *self.app.write().await = Some(app);
        }
        Ok(())
    }

    pub async fn current_status(&self) -> Status {
        *self.status.read().await
    }

    pub async fn transition(&self, status: Status) {
        *self.status.write().await = status;
    }
}
