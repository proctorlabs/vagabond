use anyhow::Result;
use lazy_static::lazy_static;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[derive(Debug, Clone, IsVariant, Display)]
pub enum Event {
    Shutdown,
}

lazy_static! {
    static ref BUS: Sender<Event> = {
        let (snd, _) = channel(24);
        snd
    };
}

pub fn broadcast(event: Event) -> Result<()> {
    BUS.send(event)?;
    Ok(())
}

pub fn subscribe() -> Receiver<Event> {
    BUS.subscribe()
}

pub fn receiver_count() -> usize {
    BUS.receiver_count()
}
