#[cfg(test)]
mod stratum_tests {
    use crate::adapter::PoolAdapter;
    use crate::client::{StratumClient, StratumEvent};
    use crate::miner_loop::MinerLoop;
    use crate::mock_server::MockStratumServer;
    use chrono::Utc;
    use shares::ShareCandidate;
    use stats::StatsManager;
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    struct TestAdapter;
    impl PoolAdapter for TestAdapter {
        fn name(&self) -> &str {
            "Test"
        }
        fn build_subscribe_request(&self) -> crate::protocol::JsonRpcRequest {
            crate::protocol::JsonRpcRequest {
                id: None,
                method: "mining.subscribe".to_string(),
                params: serde_json::json!([]),
            }
        }
        fn parse_subscribe_response(
            &self,
            _res: &crate::protocol::JsonRpcResponse,
        ) -> Result<(), String> {
            Ok(())
        }
        fn build_login_request(
            &self,
            wallet: &str,
            worker: &str,
        ) -> crate::protocol::JsonRpcRequest {
            crate::protocol::JsonRpcRequest {
                id: None,
                method: "mining.authorize".to_string(),
                params: serde_json::json!([wallet, worker]),
            }
        }
        fn parse_login_response(
            &self,
            _res: &crate::protocol::JsonRpcResponse,
        ) -> Result<bool, String> {
            Ok(true)
        }
        fn parse_job(&self, req: &crate::protocol::JsonRpcRequest) -> Option<serde_json::Value> {
            if req.method == "mining.notify" {
                Some(req.params.clone())
            } else {
                None
            }
        }
        fn parse_difficulty(&self, req: &crate::protocol::JsonRpcRequest) -> Option<f64> {
            if req.method == "mining.set_difficulty" {
                req.params.as_array()?.first()?.as_f64()
            } else {
                None
            }
        }
        fn build_share_submit(&self, share: &ShareCandidate) -> Result<crate::protocol::JsonRpcRequest, crate::adapter::PoolAdapterError> {
            Ok(crate::protocol::JsonRpcRequest {
                id: None,
                method: "mining.submit".to_string(),
                params: serde_json::json!([share.job_id, share.nonce]),
            })
        }
        fn parse_share_response(
            &self,
            _res: &crate::protocol::JsonRpcResponse,
        ) -> Result<bool, String> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_full_pipeline() {
        let addr = "127.0.0.1:5566";
        let server = MockStratumServer::new(addr).await;
        tokio::spawn(async move { server.run().await });

        // Wait for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let adapter = Arc::new(TestAdapter);
        let client = StratumClient::new(addr, adapter.clone());
        let client_handle = client.clone();
        let cancel_token = CancellationToken::new();
        let cancel_token_clone = cancel_token.clone();
        tokio::spawn(async move { client_handle.run(cancel_token_clone).await });

        let stats = Arc::new(StatsManager::new());
        let (share_tx, share_rx) = mpsc::channel(10);
        let miner = MinerLoop::new(
            client.clone(),
            adapter,
            "wallet".to_string(),
            "worker".to_string(),
            stats.clone(),
        );
        let tracker = miner.get_tracker();

        let cancel_token_miner = cancel_token.clone();
        let miner_handle = tokio::spawn(async move {
            miner.run(share_rx, cancel_token_miner).await;
        });

        // Wait for connection and events
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        {
            let tracker_lock = tracker.lock().await;
            assert!(tracker_lock.current_difficulty > 0.0);
        }

        // Submit a share
        let share = ShareCandidate {
            job_id: "job_1".to_string(),
            nonce: "1234".to_string(),
            result: "hash".to_string(),
            worker: "worker".to_string(),
            timestamp: Utc::now(),
        };
        share_tx.send(share).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        {
            let tracker_lock = tracker.lock().await;
            assert_eq!(tracker_lock.accepted_count, 1);
        }

        let current_stats = stats.get_stats().await;
        assert_eq!(current_stats.shares_submitted, 1);
        assert_eq!(current_stats.pool_accepted_shares, 1);

        cancel_token.cancel();
        let _ = miner_handle.await;
    }

    #[tokio::test]
    async fn test_url_stripping() {
        let addr = "127.0.0.1:5567";
        let server = MockStratumServer::new(addr).await;
        tokio::spawn(async move { server.run().await });
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let adapter = Arc::new(TestAdapter);
        // This should not fail to connect even with prefix
        let url = format!("stratum+tcp://{}", addr);
        let client = StratumClient::new(&url, adapter.clone());

        let client_handle = client.clone();
        let (done_tx, mut done_rx) = mpsc::channel(1);
        let cancel_token = CancellationToken::new();

        tokio::spawn(async move {
            let mut ev_rx = client_handle.subscribe();
            tokio::spawn(async move { client_handle.run(cancel_token).await });
            loop {
                if let Ok(StratumEvent::Connected) = ev_rx.recv().await {
                    let _ = done_tx.send(()).await;
                    break;
                }
            }
        });

        tokio::select! {
            _ = done_rx.recv() => {},
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => {
                panic!("Failed to connect with stratum+tcp:// prefix");
            }
        }
    }
}
