use crate::adapter::PoolAdapter;
use crate::protocol::{JsonRpcRequest, JsonRpcResponse, StratumMessage};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

#[derive(Debug, Error)]
pub enum StratumError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Codec error: {0}")]
    Codec(#[from] tokio_util::codec::LinesCodecError),
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Request timeout")]
    Timeout,
    #[error("Pool error: {0}")]
    PoolError(String),
}

#[derive(Debug, Clone)]
pub enum StratumEvent {
    Connected,
    Disconnected,
    Job(serde_json::Value),
    Difficulty(f64),
}

pub struct StratumClient {
    url: String,
    adapter: Arc<dyn PoolAdapter>,
    request_id: Mutex<u64>,
    pending_requests: Arc<Mutex<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>>,
    message_tx: Mutex<Option<mpsc::UnboundedSender<StratumMessage>>>,
    event_tx: broadcast::Sender<StratumEvent>,
}

impl StratumClient {
    pub fn new(url: &str, adapter: Arc<dyn PoolAdapter>) -> Arc<Self> {
        let (event_tx, _) = broadcast::channel(100);
        Arc::new(Self {
            url: url.to_string(),
            adapter,
            request_id: Mutex::new(0),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            message_tx: Mutex::new(None),
            event_tx,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<StratumEvent> {
        self.event_tx.subscribe()
    }

    pub async fn run(self: Arc<Self>, cancel_token: CancellationToken) {
        let mut backoff = Duration::from_secs(1);
        let max_backoff = Duration::from_secs(60);

        let clean_url = self.url.strip_prefix("stratum+tcp://").unwrap_or(&self.url);

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    info!("Stratum client shutting down...");
                    return;
                }
                res = TcpStream::connect(clean_url) => {
                    match res {
                        Ok(stream) => {
                            backoff = Duration::from_secs(1);
                            info!("Connected to {}", self.url);
                            let _ = self.event_tx.send(StratumEvent::Connected);

                            tokio::select! {
                                _ = cancel_token.cancelled() => {
                                    info!("Stratum client shutting down...");
                                    return;
                                }
                                res = self.handle_connection(stream) => {
                                    if let Err(e) = res {
                                        error!("Connection error: {}", e);
                                    }
                                }
                            }

                            let _ = self.event_tx.send(StratumEvent::Disconnected);
                            warn!("Disconnected from {}", self.url);

                            // Clear pending requests on disconnect
                            let mut pending = self.pending_requests.lock().await;
                            pending.clear();
                        }
                        Err(e) => {
                            error!("Failed to connect to {}: {}", self.url, e);
                        }
                    }
                }
            }

            info!("Reconnecting in {:?}...", backoff);
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    info!("Stratum client shutting down...");
                    return;
                }
                _ = tokio::time::sleep(backoff) => {}
            }
            backoff = std::cmp::min(backoff * 2, max_backoff);
        }
    }

    async fn handle_connection(&self, stream: TcpStream) -> Result<(), StratumError> {
        let (reader, writer) = stream.into_split();
        let mut framed_reader = FramedRead::new(reader, LinesCodec::new());
        let mut framed_writer = FramedWrite::new(writer, LinesCodec::new());

        let (message_tx, mut message_rx) = mpsc::unbounded_channel::<StratumMessage>();
        {
            let mut tx_lock = self.message_tx.lock().await;
            *tx_lock = Some(message_tx);
        }

        let pending_requests = self.pending_requests.clone();
        let adapter = self.adapter.clone();
        let event_tx = self.event_tx.clone();

        loop {
            tokio::select! {
                msg = message_rx.recv() => {
                    match msg {
                        Some(msg) => {
                            let json = match msg {
                                StratumMessage::Request(req) => serde_json::to_string(&req)?,
                                StratumMessage::Response(res) => serde_json::to_string(&res)?,
                            };
                            framed_writer.send(json).await?;
                        }
                        None => break,
                    }
                }
                result = framed_reader.next() => {
                    match result {
                        Some(Ok(line)) => {
                            match serde_json::from_str::<StratumMessage>(&line) {
                                Ok(StratumMessage::Response(res)) => {
                                    if let Some(id) = res.id {
                                        let mut pending = pending_requests.lock().await;
                                        if let Some(tx) = pending.remove(&id) {
                                            let _ = tx.send(res);
                                        }
                                    }
                                }
                                Ok(StratumMessage::Request(req)) => {
                                    if req.id.is_none() {
                                        if let Some(diff) = adapter.parse_difficulty(&req) {
                                            let _ = event_tx.send(StratumEvent::Difficulty(diff));
                                        } else if let Some(job) = adapter.parse_job(&req) {
                                            let _ = event_tx.send(StratumEvent::Job(job));
                                        }
                                    }
                                }
                                Err(e) => error!("Failed to parse message: {}. Line: {}", e, line),
                            }
                        }
                        Some(Err(e)) => return Err(e.into()),
                        None => break,
                    }
                }
            }
        }

        let mut tx_lock = self.message_tx.lock().await;
        *tx_lock = None;
        Ok(())
    }

    pub async fn send_request(
        &self,
        mut req: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, StratumError> {
        let (tx, rx) = oneshot::channel();
        let id = {
            let mut id_gen = self.request_id.lock().await;
            *id_gen += 1;
            *id_gen
        };
        req.id = Some(id);

        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id, tx);
        }

        let message_tx = {
            let tx_lock = self.message_tx.lock().await;
            tx_lock.clone()
        };

        if let Some(tx) = message_tx {
            tx.send(StratumMessage::Request(req))
                .map_err(|_| StratumError::ConnectionClosed)?;
        } else {
            return Err(StratumError::ConnectionClosed);
        }

        rx.await.map_err(|_| StratumError::ConnectionClosed)
    }
}
