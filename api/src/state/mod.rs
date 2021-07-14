use crate::{app::Vagabond, config::VagabondConfig};
use anyhow::Result;
use std::{sync::Arc, time::Duration};
use tokio::{sync::RwLock, time::sleep};

use self::events::EventChannel;

mod events;

#[derive(Debug, Clone, Deref)]
pub struct StateManager(Arc<StateManagerInner>);

#[derive(Debug, Clone)]
pub struct StateManagerInner {
    pub config: VagabondConfig,
    app: Arc<RwLock<Option<Vagabond>>>,
    status: Arc<RwLock<Status>>,
    shutdown: EventChannel<(), 32>,
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
            shutdown: EventChannel::new(),
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

    pub async fn finish_startup(&self) {
        *self.status.write().await = Status::Running;
    }

    pub async fn wait_for_shutdown(&self) -> Result<()> {
        self.shutdown.wait_for(|_| true).await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        *self.status.write().await = Status::ShuttingDown;
        self.shutdown.send(())?;
        while self.shutdown.has_receivers() {
            sleep(Duration::from_millis(100)).await;
        }
        Ok(())
    }
}
