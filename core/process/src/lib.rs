use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{info, warn};

pub trait LogParser: Send + Sync {
    fn parse_line(&mut self, line: &str);
}

#[derive(Default, Debug)]
pub struct DefaultMinerLogParser {
    pub accepted: usize,
    pub rejected: usize,
    pub hashrate_lines: usize,
    pub gpu_temp_found: usize,
    pub gpu_name_found: usize,
}

impl LogParser for DefaultMinerLogParser {
    fn parse_line(&mut self, line: &str) {
        let line_lower = line.to_lowercase();

        if line_lower.contains("accepted") {
            self.accepted += 1;
        } else if line_lower.contains("rejected") {
            self.rejected += 1;
        }

        if line_lower.contains("h/s") {
            self.hashrate_lines += 1;
        }

        if line_lower.contains("temp") || line_lower.contains("temperature") {
            self.gpu_temp_found += 1;
        }

        if line_lower.contains("gpu") {
            self.gpu_name_found += 1;
        }
    }
}

pub struct ProcessManager;

impl ProcessManager {
    pub async fn spawn(binary_path: &str, args: &[String]) -> Result<Child, std::io::Error> {
        info!("Starting miner process: {} {:?}", binary_path, args);

        let mut child = Command::new(binary_path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().expect("Failed to open stdout");
        let stderr = child.stderr.take().expect("Failed to open stderr");

        tokio::spawn(async move {
            let mut parser = DefaultMinerLogParser::default();
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                parser.parse_line(&line);
                info!("[MINER OUT] {}", line);
            }
        });

        tokio::spawn(async move {
            let mut parser = DefaultMinerLogParser::default();
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                parser.parse_line(&line);
                warn!("[MINER ERR] {}", line);
            }
        });

        Ok(child)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_parser_accepted_rejected() {
        let mut parser = DefaultMinerLogParser::default();
        parser.parse_line("[INFO] Share accepted! (123 ms)");
        parser.parse_line("Share Accepted (45 ms)");
        parser.parse_line("share rejected: invalid nonce");

        assert_eq!(parser.accepted, 2);
        assert_eq!(parser.rejected, 1);
        assert_eq!(parser.hashrate_lines, 0);
    }

    #[test]
    fn test_log_parser_tolerant() {
        let mut parser = DefaultMinerLogParser::default();
        parser.parse_line("Current hashrate: 145 Mh/s");
        parser.parse_line("Speed: 1000 H/s");
        parser.parse_line("GPU0 temp: 65C");
        parser.parse_line("Detected GPU: RTX 3080");
        parser.parse_line("Random unparseable garbage 123 !@#");

        assert_eq!(parser.hashrate_lines, 2);
        assert_eq!(parser.gpu_temp_found, 1);
        assert_eq!(parser.gpu_name_found, 2); // 'GPU0' and 'GPU' both contain 'gpu'
    }
}
