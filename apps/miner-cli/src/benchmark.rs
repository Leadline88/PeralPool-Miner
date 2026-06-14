use crate::args::Args;
use chrono::Utc;
use mining::MiningAlgorithm;
use std::sync::Arc;
use tracing::info;

pub async fn run(args: &Args) {
    info!("Benchmarking Native CPU (SYNTHETIC / REFERENCE-ONLY)...");
    let algo = algo_pearl::PearlAlgorithm;
    let dummy_job_json = r#"{"id":"bench","blob":"00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff","target":1000000}"#;

    // Benchmark Job Parsing
    let parse_start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = algo.parse_job(dummy_job_json).unwrap();
    }
    let parse_time_avg = parse_start.elapsed().as_secs_f64() / 1000.0;

    // Benchmark Work Package Creation
    let work_start = std::time::Instant::now();
    let job = algo.parse_job(dummy_job_json).unwrap();
    for _ in 0..1000 {
        let _ = algo.create_work(
            &job,
            mining::NonceRange {
                start: 0,
                end: 1000,
            },
        );
    }
    let work_time_avg = work_start.elapsed().as_secs_f64() / 1000.0;

    // Benchmark Share Verification
    let verify_start = std::time::Instant::now();
    let blob = vec![0u8; 32];
    for i in 0..10000 {
        let _ = algo_pearl::PearlVerifier::verify(&blob, i as u64, 1000000);
    }
    let verify_time_avg = verify_start.elapsed().as_secs_f64() / 10000.0;

    // Benchmark Throughput - Single-threaded
    info!("Running single-threaded throughput benchmark (5 seconds)...");
    let start = std::time::Instant::now();
    let mut count = 0;
    let target = 0x00000000FFFFFFFF;
    while start.elapsed().as_secs() < 5 {
        for _ in 0..10000 {
            algo_pearl::PearlVerifier::verify(&blob, count, target);
            count += 1;
        }
    }
    let elapsed_single = start.elapsed().as_secs_f64();
    let hashrate_single = count as f64 / elapsed_single;
    info!("Single-threaded result: {:.2} H/s", hashrate_single);

    // Benchmark Throughput - Multi-threaded
    let threads = args.threads.unwrap_or_else(num_cpus::get);
    info!(
        "Running multi-threaded ({} threads) throughput benchmark (5 seconds)...",
        threads
    );
    let start = std::time::Instant::now();
    let total_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let mut handles = Vec::new();

    for i in 0..threads {
        let total_count = total_count.clone();
        let blob = blob.clone();
        handles.push(std::thread::spawn(move || {
            let mut local_count = 0;
            let mut nonce = i as u64;
            let thread_start = std::time::Instant::now();
            while thread_start.elapsed().as_secs() < 5 {
                for _ in 0..10000 {
                    algo_pearl::PearlVerifier::verify(&blob, nonce, target);
                    nonce += threads as u64;
                    local_count += 1;
                }
            }
            total_count.fetch_add(local_count, std::sync::atomic::Ordering::Relaxed);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed_multi = start.elapsed().as_secs_f64();
    let hashrate_multi =
        total_count.load(std::sync::atomic::Ordering::Relaxed) as f64 / elapsed_multi;
    info!("Multi-threaded result: {:.2} H/s", hashrate_multi);

    let report = serde_json::json!({
        "backend": "native-cpu-synthetic",
        "hashrate_single": hashrate_single,
        "hashrate_multi": hashrate_multi,
        "unit": "H/s",
        "threads": threads,
        "duration_secs": 5.0,
        "avg_job_parse_secs": parse_time_avg,
        "avg_work_package_creation_secs": work_time_avg,
        "avg_share_verify_secs": verify_time_avg,
        "timestamp": Utc::now().to_rfc3339()
    });

    println!("{}", serde_json::to_string_pretty(&report).unwrap());

    // Write report to file
    let report_path = args
        .benchmark_output
        .as_deref()
        .unwrap_or("benchmark_report.json");
    if let Ok(content) = serde_json::to_string_pretty(&report) {
        if let Err(e) = std::fs::write(report_path, content) {
            tracing::error!("Failed to save benchmark report to {}: {}", report_path, e);
        } else {
            info!("Benchmark report saved to {}", report_path);
        }
    }

    info!("Benchmark complete!");
}
