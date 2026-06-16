use crate::error::CaptureError;
use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Standalone Stratum JSON-RPC proxy and capture tool", long_about = None)]
pub struct Args {
    /// Local address to listen on (e.g., 127.0.0.1:3333)
    #[arg(long, default_value = "127.0.0.1:3333")]
    pub listen: String,

    /// Upstream pool address (e.g., pearlpool.cloud:5566)
    #[arg(long)]
    pub pool: String,

    /// Output JSONL file path
    #[arg(long)]
    pub output: String,

    /// Wallet address to redact
    #[arg(long)]
    pub redact_wallet: Option<String>,

    /// Worker name to redact
    #[arg(long)]
    pub redact_worker: Option<String>,

    /// Password to redact
    #[arg(long)]
    pub redact_password: Option<String>,

    /// Optional label for the capture session
    #[arg(long)]
    pub session_label: Option<String>,

    /// Maximum number of lines to capture before stopping
    #[arg(long)]
    pub max_lines: Option<usize>,

    /// Maximum number of seconds to capture before stopping
    #[arg(long)]
    pub max_seconds: Option<u64>,

    /// Print pretty-formatted lines to stdout
    #[arg(long, default_value_t = false)]
    pub pretty_log: bool,

    /// Run without connecting or listening (validates args)
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

impl Args {
    pub fn validate(&self) -> Result<(), CaptureError> {
        // --listen must parse as a socket address
        self.listen
            .parse::<SocketAddr>()
            .map_err(|e| CaptureError::InvalidArgs(format!("Invalid listen address: {}", e)))?;

        if self.listen.starts_with("0.0.0.0") || self.listen.starts_with("[::]") {
            eprintln!("WARNING: Listening on all interfaces ({}) is not recommended for a capture tool. It is safer to use 127.0.0.1.", self.listen);
        }

        // --pool must be non-empty and contain host:port
        if self.pool.is_empty() {
            return Err(CaptureError::InvalidArgs(
                "Pool address is empty".to_string(),
            ));
        }
        if !self.pool.contains(':') {
            return Err(CaptureError::InvalidArgs(
                "Pool address must contain port (host:port)".to_string(),
            ));
        }

        // --output must be non-empty
        if self.output.is_empty() {
            return Err(CaptureError::InvalidArgs(
                "Output file path is empty".to_string(),
            ));
        }

        // --max-lines, if set, must be greater than 0
        if let Some(max) = self.max_lines {
            if max == 0 {
                return Err(CaptureError::InvalidArgs(
                    "max-lines must be greater than 0".to_string(),
                ));
            }
        }

        // --max-seconds, if set, must be greater than 0
        if let Some(max) = self.max_seconds {
            if max == 0 {
                return Err(CaptureError::InvalidArgs(
                    "max-seconds must be greater than 0".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_args() -> Args {
        Args {
            listen: "127.0.0.1:3333".to_string(),
            pool: "pool:5566".to_string(),
            output: "capture.jsonl".to_string(),
            redact_wallet: None,
            redact_worker: None,
            redact_password: None,
            session_label: None,
            max_lines: None,
            max_seconds: None,
            pretty_log: false,
            dry_run: false,
        }
    }

    #[test]
    fn test_valid_args() {
        let args = basic_args();
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_invalid_listen() {
        let mut args = basic_args();
        args.listen = "invalid".to_string();
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_empty_pool() {
        let mut args = basic_args();
        args.pool = "".to_string();
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_pool_missing_port() {
        let mut args = basic_args();
        args.pool = "localhost".to_string();
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_empty_output() {
        let mut args = basic_args();
        args.output = "".to_string();
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_invalid_limits() {
        let mut args = basic_args();
        args.max_lines = Some(0);
        assert!(args.validate().is_err());

        args.max_lines = None;
        args.max_seconds = Some(0);
        assert!(args.validate().is_err());
    }
}
