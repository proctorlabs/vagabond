use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::VagabondConfig;

#[derive(Debug, Clone, Deref)]
pub struct StateManager(Arc<StateManagerInner>);

#[derive(Debug, Clone)]
pub struct StateManagerInner {
    pub config: VagabondConfig,
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
            config,
        })))
    }

    pub async fn current_status(&self) -> Status {
        *self.status.read().await
    }

    pub async fn transition(&self, status: Status) {
        *self.status.write().await = status;
    }
}
