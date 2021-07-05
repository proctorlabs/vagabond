use super::*;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use futures::{FutureExt, StreamExt};
use warp::Filter;

// use tide::Request;
// use tide::prelude::*;

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

    // async fn test_request(_: Request<()>) -> tide::Result {
    //     Ok("Request received".into())
    // }
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
        let root_path = warp::path::end().map(|| "Root path");
        let index_path = warp::path("other").map(|| "Other path");
        let readme = warp::path("readme").and(warp::fs::file("./README.md"));
        let content = warp::path("content").and(warp::fs::dir("./static/"));
        let ws = warp::path!("ws")
            // The `ws()` filter will prepare the Websocket handshake.
            .and(warp::ws())
            .map(|ws: warp::ws::Ws| {
                // And then our closure will be called when it completes...
                ws.on_upgrade(|websocket| {
                    // Just echo all messages back...
                    let (tx, rx) = websocket.split();
                    rx.forward(tx).map(|result| {
                        if let Err(e) = result {
                            eprintln!("websocket error: {:?}", e);
                        }
                    })
                })
            });
        let routes = warp::get()
            .and(root_path.or(index_path).or(readme).or(content))
            .or(ws);

        warp::serve(routes).run(([127, 0, 0, 1], 8081)).await;
        Ok(())
    }

    // async fn start(&self) -> Result<()> {
    //     let mut app = tide::new();
    //     app.at("/testget").get(Self::test_request);
    //     app.at("/").serve_dir("static/")?;
    //     app.at("/").serve_file("static/index.html")?;
    //     debug!("Starting HTTP service...");
    //     app.listen("127.0.0.1:8081").await?;
    //     Ok(())
    // }
}
