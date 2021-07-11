use crate::data::ServiceState;

use super::*;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, ChildStderr, ChildStdout};
use tokio::sync::RwLock;
use tokio::time::sleep;

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

pub trait ProcessService: Clone + Sync + Send + Sized {
    const SERVICE_NAME: &'static str;
    const COMMAND: &'static str;
    const RESTART_TIME: u64;
    fn get_args(&self) -> Vec<String>;
}

#[derive(Debug, Clone)]
pub struct ProcessManager<P: ProcessService> {
    status: Arc<RwLock<ServiceState>>,
    state_manager: StateManager,
    pid: Arc<AtomicU32>,
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
                    P::SERVICE_NAME
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
        self.set_pid(cmd_process.id().unwrap_or_default());

        tokio::try_join!(
            self.clone().watch_process(&mut cmd_process),
            self.clone().watch_bus(),
            self.clone().log_stdout(stdout),
            self.clone().log_stderr(stderr),
        )?;
        cmd_process.wait().await?;
        Ok(())
    }
}

impl<P: ProcessService> ProcessManager<P> {
    pub async fn new(meta: P, state_manager: StateManager) -> Result<Self> {
        Ok(Self {
            meta,
            state_manager,
            pid: Arc::new(AtomicU32::new(0)),
            status: Arc::new(RwLock::new(ServiceState::Stopped)),
        })
    }

    pub fn get_pid(&self) -> u32 {
        self.pid.load(Ordering::Relaxed)
    }

    fn set_pid(&self, pid: u32) {
        let cid = self.get_pid();
        self.pid
            .compare_exchange(cid, pid, Ordering::Relaxed, Ordering::Relaxed)
            .unwrap_or_default();
    }

    pub fn signal(&self, signal: Signal) -> Result<()> {
        let upid = self.get_pid();
        if upid != 0 {
            let pid = Pid::from_raw(upid as i32);
            signal::kill(pid, signal)?;
        }
        Ok(())
    }

    pub async fn current_state(&self) -> Result<ServiceState> {
        Ok(*self.status.read().await)
    }

    async fn watch_process(self, cmd_process: &mut Child) -> Result<()> {
        let exit_status = cmd_process.wait().await?;
        self.set_pid(0);
        *self.status.write().await = ServiceState::Failed;
        warn!(
            "[{}] Process exited with status {}",
            P::SERVICE_NAME,
            exit_status
        );
        sleep(Duration::from_millis(500)).await;
        Err(anyhow::anyhow!("[{}] Exited", P::SERVICE_NAME))
    }

    async fn watch_bus(self) -> Result<()> {
        let mut bus = crate::bus::subscribe();
        loop {
            match bus.recv().await? {
                crate::bus::Event::Shutdown => {
                    *self.status.write().await = ServiceState::Stopped;
                    warn!("[{}] Aborting service due to shutdown", P::SERVICE_NAME);
                    self.signal(Signal::SIGTERM)?;
                    return Ok(());
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
