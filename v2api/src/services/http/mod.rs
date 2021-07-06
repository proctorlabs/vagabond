use super::*;
use anyhow::Result;
use async_trait::async_trait;
use futures::{FutureExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

#[derive(Debug, Clone)]
pub struct HttpServer {
    running: Arc<Mutex<bool>>,
    config: VagabondConfig,
}

impl HttpServer {
    pub async fn new(config: VagabondConfig) -> Result<Self> {
        Ok(HttpServer {
            running: Arc::new(Mutex::new(false)),
            config,
        })
    }

    fn handle_websocket(ws: warp::ws::Ws) -> impl warp::Reply {
        ws.on_upgrade(|websocket| {
            let (tx, rx) = websocket.split();
            rx.forward(tx).map(|result| {
                if let Err(e) = result {
                    eprintln!("websocket error: {:?}", e);
                }
            })
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

    async fn start(&self) -> Result<()> {
        let content = warp::get().and(warp::fs::dir("./static/"));

        let alternate_index = warp::get().and(
            warp::fs::file("./static/index.html")
                .and(warp::path::param::<String>().and(warp::path::end()))
                .map(|file, _param| file),
        );

        let ws = warp::path!("api" / "sock")
            .and(warp::ws())
            .map(Self::handle_websocket);

        let routes = ws.or(content).or(alternate_index);

        warp::serve(routes).run(([127, 0, 0, 1], 8081)).await;
        Ok(())
    }
}
