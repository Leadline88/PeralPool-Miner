use std::fmt;

#[derive(Debug)]
pub enum CaptureError {
    InvalidArgs(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    #[allow(dead_code)]
    Proxy(String),
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaptureError::InvalidArgs(msg) => write!(f, "Invalid arguments: {}", msg),
            CaptureError::Io(err) => write!(f, "IO error: {}", err),
            CaptureError::Json(err) => write!(f, "JSON error: {}", err),
            CaptureError::Proxy(msg) => write!(f, "Proxy error: {}", msg),
        }
    }
}

impl std::error::Error for CaptureError {}

impl From<std::io::Error> for CaptureError {
    fn from(err: std::io::Error) -> Self {
        CaptureError::Io(err)
    }
}

impl From<serde_json::Error> for CaptureError {
    fn from(err: serde_json::Error) -> Self {
        CaptureError::Json(err)
    }
}
