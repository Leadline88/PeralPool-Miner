use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureLine {
    pub timestamp_utc: String,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_label: Option<String>,
    pub direction: String,
    pub raw_line_redacted: String,
    pub jsonrpc: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params_shape: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_shape: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_shape: Option<String>,
    pub redaction_applied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureEvent {
    pub metadata: EventMetadata,
}
