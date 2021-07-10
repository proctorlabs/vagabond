use super::*;
use crate::{app::Vagabond, data::*};
use anyhow::Result;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

mod dispatcher;

#[derive(Debug, Clone)]
pub struct HttpServer {
    running: Arc<Mutex<bool>>,
    state_manager: StateManager,
}

impl HttpServer {
    pub async fn new(state_manager: StateManager) -> Result<Self> {
        Ok(HttpServer {
            running: Arc::new(Mutex::new(false)),
            state_manager,
        })
    }

    fn handle_websocket(ws: warp::ws::Ws, app: Vagabond) -> impl warp::Reply {
        ws.on_upgrade(|websocket| {
            let (mut tx, mut rx) = websocket.split();
            async move {
                info!("Websocket connected");
                loop {
                    match rx.next().await {
                        Some(Ok(m)) => {
                            if let Ok(txt) = m.to_str() {
                                let r = serde_json::from_str::<WebsocketRxMessage>(txt);
                                if let Ok(parsed) = r {
                                    let r = parsed.dispatch(&app, &mut tx).await;
                                    if let Err(e) = r {
                                        let txmsg: String = serde_json::to_string(
                                            &WebsocketTxMessage::Error(format!("{}", e)),
                                        )
                                        .unwrap_or_else(|e| {
                                            format!("{}{:?}\"}}", r#"{"type":"error","data":""#, e)
                                        });
                                        let msg = warp::ws::Message::text(txmsg);
                                        if let Err(e) = tx.send(msg).await {
                                            warn!("Could not send websocket reply! {:?}", e);
                                        }
                                    }
                                } else {
                                    warn!("Failed to parse websocket message! {:?}", r);
                                }
                            }
                        }
                        Some(Err(e)) => {
                            warn!("Error received on websocket! {}", e);
                            break;
                        }
                        _ => break,
                    }
                }
                info!("Websocket disconnected");
            }
        })
    }
}

#[async_trait]
impl Service for HttpServer {
    fn name(&self) -> &'static str {
        "HTTP"
    }

    fn restart_time(&self) -> u64 {
        15
    }

    async fn state_manager(&self) -> StateManager {
        self.state_manager.clone()
    }

    async fn start(&self) -> Result<()> {
        let app = self.state_manager.vagabond().await?;
        let app = warp::any().map(move || app.clone());
        let content = warp::get().and(warp::fs::dir("./static/"));

        let alternate_index = warp::get().and(
            warp::fs::file("./static/index.html")
                .and(warp::path::param::<String>().and(warp::path::end()))
                .map(|file, _param| file),
        );

        let ws = warp::path!("api" / "sock")
            .and(warp::ws())
            .and(app.clone())
            .map(Self::handle_websocket);

        let routes = ws.or(content).or(alternate_index);

        warp::serve(routes).run(([127, 0, 0, 1], 8081)).await;
        Ok(())
    }
}
