use crate::data::ServiceState;

use super::*;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, ChildStderr, ChildStdout};
use tokio::sync::RwLock;

pub trait ProcessService: Clone + Sync + Send + Sized {
    const SERVICE_NAME: &'static str;
    const COMMAND: &'static str;
    const RESTART_TIME: u64;
    fn get_args(&self) -> &[&str];
}

#[derive(Debug, Clone)]
pub struct ProcessManager<P: ProcessService> {
    status: Arc<RwLock<ServiceState>>,
    state_manager: StateManager,
    meta: P,
}

#[async_trait]
impl<P: ProcessService> Service for ProcessManager<P> {
    fn name(&self) -> &'static str {
        P::SERVICE_NAME
    }

    fn restart_time(&self) -> u64 {
        P::RESTART_TIME
    }

    async fn state_manager(&self) -> StateManager {
        self.state_manager.clone()
    }

    async fn start(&self) -> Result<()> {
        {
            let mut status = self.status.write().await;
            if *status == ServiceState::Running {
                return Err(anyhow::anyhow!(format!(
                    "{} service is already running!",
                    P::COMMAND
                )));
            }
            *status = ServiceState::Running;
        }

        let mut cmd_process = tokio::process::Command::new(P::COMMAND)
            .args(self.meta.get_args())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()?;

        let stdout = cmd_process.stdout.take().unwrap();
        let stderr = cmd_process.stderr.take().unwrap();

        tokio::try_join!(
            self.clone().watch_process(cmd_process),
            self.clone().watch_bus(),
            self.clone().log_stdout(stdout),
            self.clone().log_stderr(stderr),
        )?;
        Ok(())
    }
}

impl<P: ProcessService> ProcessManager<P> {
    pub async fn new(meta: P, state_manager: StateManager) -> Result<Self> {
        Ok(Self {
            meta,
            state_manager,
            status: Arc::new(RwLock::new(ServiceState::Stopped)),
        })
    }

    pub async fn current_state(&self) -> Result<ServiceState> {
        Ok(*self.status.read().await)
    }

    async fn watch_process(self, mut cmd_process: Child) -> Result<()> {
        let exit_status = cmd_process.wait().await?;
        *self.status.write().await = ServiceState::Failed;
        // This sleep gives a little time for buffers to clear
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        Err::<(), anyhow::Error>(anyhow::anyhow!(format!(
            "{} process exited with {}",
            P::COMMAND,
            exit_status
        )))
    }

    async fn watch_bus(self) -> Result<()> {
        let mut bus = crate::bus::subscribe();
        loop {
            match bus.recv().await? {
                crate::bus::Event::Shutdown => {
                    *self.status.write().await = ServiceState::Stopped;
                    return Err::<(), anyhow::Error>(anyhow::anyhow!(format!(
                        "Aborting {} due to shutdown signal",
                        P::SERVICE_NAME
                    )));
                }
            }
        }
    }

    async fn log_stdout(self, stdout: ChildStdout) -> Result<()> {
        let mut stdout = tokio::io::BufReader::new(stdout);
        let mut buf = String::new();
        while stdout.read_line(&mut buf).await? != 0 {
            info!("[{}] {}", P::SERVICE_NAME, buf.trim_end());
            buf.clear();
        }
        Ok(())
    }

    async fn log_stderr(self, stderr: ChildStderr) -> Result<()> {
        let mut stderr = tokio::io::BufReader::new(stderr);
        let mut buf = String::new();
        while stderr.read_line(&mut buf).await? != 0 {
            warn!("[{}] {}", P::SERVICE_NAME, buf.trim_end());
            buf.clear();
        }
        Ok(())
    }
}
