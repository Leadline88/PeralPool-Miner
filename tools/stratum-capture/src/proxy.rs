use crate::args::Args;
use crate::error::CaptureError;
use crate::fixture::FixtureLine;
use crate::jsonrpc::extract_shape;
use crate::redaction::Redactor;
use chrono::Utc;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch};
use tokio::time::{timeout, Duration};

#[derive(Debug)]
struct LogMessage {
    direction: String,
    raw_line: String,
}

pub async fn run(args: Args) -> Result<(), CaptureError> {
    let listener = TcpListener::bind(&args.listen).await?;
    println!("Listening on {}", args.listen);

    // Accept exactly one connection for now to keep it simple and focused
    let (client_stream, client_addr) = listener.accept().await?;
    println!("Accepted connection from miner at {}", client_addr);

    let pool_stream = TcpStream::connect(&args.pool).await?;
    println!("Connected to upstream pool at {}", args.pool);

    let (mut client_read, mut client_write) = client_stream.into_split();
    let (mut pool_read, mut pool_write) = pool_stream.into_split();

    let (tx, mut rx) = mpsc::channel::<LogMessage>(100);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let shutdown_tx_logger = shutdown_tx.clone();

    let redactor = Arc::new(Redactor::new(&args));
    let session_id = uuid::Uuid::new_v4().to_string();
    let args_arc = Arc::new(args);
    let args_clone1 = args_arc.clone();

    // Logger task
    let logger_session_id = session_id.clone();
    let mut logger_handle = tokio::spawn(async move {
        let mut output_file = tokio::fs::File::create(&args_clone1.output)
            .await
            .map_err(CaptureError::Io)?;

        // Log connection event
        let conn_event = FixtureLine::proxy_event(
            logger_session_id.clone(),
            args_clone1.session_label.clone(),
            "connect",
            Some("miner_connected"),
        );
        output_file
            .write_all(format!("{}\n", serde_json::to_string(&conn_event)?).as_bytes())
            .await?;

        let mut lines_captured = 0;
        let start_time = tokio::time::Instant::now();

        let result: Result<(), CaptureError> = Ok(());

        while let Some(msg) = rx.recv().await {
            lines_captured += 1;

            let mut redacted_line = redactor.redact_raw_line(&msg.raw_line);
            let mut jsonrpc = false;
            let mut method = None;
            let mut id = None;
            let mut params_shape = None;
            let mut result_shape = None;
            let mut error_shape = None;
            let mut redaction_applied = redacted_line != msg.raw_line;

            if let Ok(mut json_val) = serde_json::from_str::<serde_json::Value>(&msg.raw_line) {
                jsonrpc = true;
                if redactor.redact_json_value(&mut json_val) {
                    redaction_applied = true;
                }
                redacted_line = serde_json::to_string(&json_val)?;

                let shape = extract_shape(&json_val);
                method = shape.method;
                id = shape.id;
                params_shape = shape.params_shape;
                result_shape = shape.result_shape;
                error_shape = shape.error_shape;
            }

            let fixture = FixtureLine {
                timestamp_utc: Utc::now().to_rfc3339(),
                session_id: logger_session_id.clone(),
                session_label: args_clone1.session_label.clone(),
                direction: msg.direction.clone(),
                raw_line_redacted: redacted_line.clone(),
                jsonrpc,
                method,
                id,
                params_shape,
                result_shape,
                error_shape,
                redaction_applied,
                notes: None,
            };

            let out_str = serde_json::to_string(&fixture)?;
            output_file
                .write_all(format!("{}\n", out_str).as_bytes())
                .await?;
            output_file.flush().await?;

            if args_clone1.pretty_log {
                println!("[{}] {}", msg.direction, redacted_line);
            }

            let mut stop = false;
            if let Some(max) = args_clone1.max_lines {
                if lines_captured >= max {
                    println!("Reached max lines: {}", max);
                    stop = true;
                }
            }

            if let Some(max_secs) = args_clone1.max_seconds {
                if start_time.elapsed().as_secs() >= max_secs {
                    println!("Reached max seconds: {}", max_secs);
                    stop = true;
                }
            }

            if stop {
                let _ = shutdown_tx_logger.send(true);
                break;
            }
        }

        let disconn_event = FixtureLine::proxy_event(
            logger_session_id.clone(),
            args_clone1.session_label.clone(),
            "disconnect",
            Some("capture_finished"),
        );
        output_file
            .write_all(format!("{}\n", serde_json::to_string(&disconn_event)?).as_bytes())
            .await?;
        output_file.flush().await?;

        println!("Saved {} lines to {}", lines_captured, args_clone1.output);
        result
    });

    let tx_client_to_pool = tx.clone();
    let mut shutdown_client = shutdown_rx.clone();
    let client_to_pool = tokio::spawn(async move {
        let mut reader = BufReader::new(&mut client_read);
        let mut line = String::new();
        loop {
            tokio::select! {
                _ = shutdown_client.changed() => break,
                res = reader.read_line(&mut line) => {
                    match res {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let trim_line = line.trim_end().to_string();
                            if trim_line.is_empty() {
                                continue;
                            }
                            let _ = tx_client_to_pool
                                .send(LogMessage {
                                    direction: "client_to_pool".to_string(),
                                    raw_line: trim_line.clone(),
                                })
                                .await;
                            if let Err(e) = pool_write.write_all(line.as_bytes()).await {
                                eprintln!("Failed to write to pool: {}", e);
                                break;
                            }
                            line.clear();
                        }
                        Err(e) => {
                            eprintln!("Error reading from client: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    });

    let tx_pool_to_client = tx.clone();
    let mut shutdown_pool = shutdown_rx.clone();
    let pool_to_client = tokio::spawn(async move {
        let mut reader = BufReader::new(&mut pool_read);
        let mut line = String::new();
        loop {
            tokio::select! {
                _ = shutdown_pool.changed() => break,
                res = reader.read_line(&mut line) => {
                    match res {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let trim_line = line.trim_end().to_string();
                            if trim_line.is_empty() {
                                continue;
                            }
                            let _ = tx_pool_to_client
                                .send(LogMessage {
                                    direction: "pool_to_client".to_string(),
                                    raw_line: trim_line.clone(),
                                })
                                .await;
                            if let Err(e) = client_write.write_all(line.as_bytes()).await {
                                eprintln!("Failed to write to client: {}", e);
                                break;
                            }
                            line.clear();
                        }
                        Err(e) => {
                            eprintln!("Error reading from pool: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    });

    // Wait for the logger to finish if a limit was hit, or one of the connection directions to close.
    let timeout_dur = if let Some(secs) = args_arc.max_seconds {
        Duration::from_secs(secs + 2)
    } else {
        Duration::from_secs(u64::MAX) // effectively infinite
    };

    let _ = timeout(timeout_dur, async {
        tokio::select! {
            _ = client_to_pool => println!("Miner disconnected or session stopped."),
            _ = pool_to_client => println!("Pool disconnected or session stopped."),
            res = &mut logger_handle => {
                match res {
                    Ok(Ok(_)) => println!("Logger stopped."),
                    Ok(Err(e)) => eprintln!("Logger error: {}", e),
                    Err(e) => eprintln!("Logger task panicked: {}", e),
                }
            },
        }
    })
    .await;

    // Ensure all senders are dropped so rx.recv() returns None
    drop(tx);
    let _ = shutdown_tx.send(true);

    // Wait for the logger to finish processing remaining messages and flush the file
    match logger_handle.await {
        Ok(Ok(_)) => println!("Logger finished safely."),
        Ok(Err(e)) => eprintln!("Logger error during shutdown: {}", e),
        Err(e) => eprintln!("Logger task panicked during shutdown: {}", e),
    }

    println!("Capture session ended.");

    Ok(())
}
