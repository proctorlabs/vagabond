use super::*;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tide::Request;
use tokio::sync::Mutex;
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

    async fn test_request(_: Request<()>) -> tide::Result {
        Ok("Request received".into())
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
        let mut app = tide::new();
        app.at("/").get(Self::test_request);
        debug!("Starting HTTP service...");
        app.listen("127.0.0.1:8081").await?;
        Ok(())
    }
}
