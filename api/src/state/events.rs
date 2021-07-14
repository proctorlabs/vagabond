use std::fmt::Debug;

use anyhow::Result;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct EventChannel<T: Clone + Debug + Send + Sync + 'static, const CAP: usize> {
    channel: Sender<T>,
}

impl<T: Clone + Debug + Send + Sync + 'static, const CAP: usize> EventChannel<T, CAP> {
    pub fn new() -> Self {
        let (channel, _) = channel(CAP);
        Self { channel }
    }

    pub fn has_receivers(&self) -> bool {
        self.channel.receiver_count() > 0
    }

    pub fn send(&self, msg: T) -> Result<()> {
        self.channel.send(msg)?;
        Ok(())
    }

    pub fn subscribe(&self) -> Receiver<T> {
        self.channel.subscribe()
    }

    pub async fn wait_for(&self, f: fn(msg: &T) -> bool) -> Result<T> {
        let mut rcvr = self.subscribe();
        loop {
            let msg = rcvr.recv().await?;
            if f(&msg) {
                return Ok(msg);
            }
        }
    }
}
