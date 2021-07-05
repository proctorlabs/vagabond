use lazy_static::lazy_static;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct State {
    status: Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant, Display)]
pub enum Status {
    Starting,
    Running,
    ShuttingDown,
}

lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(State {
        status: Status::Starting,
    });
}

pub async fn shutdown() {
    let mut state = STATE.lock().await;
    state.status = Status::ShuttingDown;
}

pub async fn running() {
    let mut state = STATE.lock().await;
    state.status = Status::Running;
}

pub async fn status() -> Status {
    let state = STATE.lock().await;
    state.status
}
