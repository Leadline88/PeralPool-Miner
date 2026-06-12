use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use futures::{SinkExt, StreamExt};
use tracing::{error, info};
use crate::protocol::{JsonRpcRequest, JsonRpcResponse, StratumMessage};
use serde_json::json;

pub struct MockStratumServer {
    addr: String,
}

impl MockStratumServer {
    pub async fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_string(),
        }
    }

    pub async fn run(&self) {
        let listener = TcpListener::bind(&self.addr).await.expect("Failed to bind");
        info!("Mock pool listening on {}", self.addr);

        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(stream).await {
                    error!("Mock pool client error: {}", e);
                }
            });
        }
    }

    async fn handle_client(stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let (reader, writer) = stream.into_split();
        let mut framed_reader = FramedRead::new(reader, LinesCodec::new());
        let mut framed_writer = FramedWrite::new(writer, LinesCodec::new());

        while let Some(result) = framed_reader.next().await {
            let line = result?;
            let msg: StratumMessage = serde_json::from_str(&line)?;

            if let StratumMessage::Request(req) = msg {
                match req.method.as_str() {
                    "mining.subscribe" => {
                        let res = JsonRpcResponse {
                            id: req.id,
                            result: Some(json!([[], "extra_nonce_1", 4])),
                            error: None,
                        };
                        framed_writer.send(serde_json::to_string(&res)?).await?;

                        // Send difficulty
                        let diff_req = JsonRpcRequest {
                            id: None,
                            method: "mining.set_difficulty".to_string(),
                            params: json!([10.0]),
                        };
                        framed_writer.send(serde_json::to_string(&diff_req)?).await?;

                        // Send job
                        let job_req = JsonRpcRequest {
                            id: None,
                            method: "mining.notify".to_string(),
                            params: json!(["job_1", "prev_hash", "coinbase1", "coinbase2", [], "version", "nbits", "ntime", true]),
                        };
                        framed_writer.send(serde_json::to_string(&job_req)?).await?;
                    }
                    "mining.authorize" => {
                        let res = JsonRpcResponse {
                            id: req.id,
                            result: Some(json!(true)),
                            error: None,
                        };
                        framed_writer.send(serde_json::to_string(&res)?).await?;
                    }
                    "mining.submit" => {
                        let res = JsonRpcResponse {
                            id: req.id,
                            result: Some(json!(true)),
                            error: None,
                        };
                        framed_writer.send(serde_json::to_string(&res)?).await?;
                    }
                    _ => {
                        let res = JsonRpcResponse {
                            id: req.id,
                            result: None,
                            error: Some(json!(["Method not found", 20, null])),
                        };
                        framed_writer.send(serde_json::to_string(&res)?).await?;
                    }
                }
            }
        }
        Ok(())
    }
}
