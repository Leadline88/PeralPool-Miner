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

impl FixtureLine {
    pub fn proxy_event(
        session_id: String,
        session_label: Option<String>,
        event: &str,
        reason: Option<&str>,
    ) -> Self {
        let note = if let Some(r) = reason {
            format!("event: {}, reason: {}", event, r)
        } else {
            format!("event: {}", event)
        };

        Self {
            timestamp_utc: chrono::Utc::now().to_rfc3339(),
            session_id,
            session_label,
            direction: "proxy_event".to_string(),
            raw_line_redacted: "".to_string(),
            jsonrpc: false,
            method: None,
            id: None,
            params_shape: None,
            result_shape: None,
            error_shape: None,
            redaction_applied: false,
            notes: Some(note),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_line_required_fields() {
        let fixture = FixtureLine {
            timestamp_utc: "2023-01-01T00:00:00Z".to_string(),
            session_id: "test-session".to_string(),
            session_label: None,
            direction: "client_to_pool".to_string(),
            raw_line_redacted: "{}".to_string(),
            jsonrpc: true,
            method: None,
            id: None,
            params_shape: None,
            result_shape: None,
            error_shape: None,
            redaction_applied: false,
            notes: None,
        };
        let json = serde_json::to_string(&fixture).unwrap();
        assert!(json.contains("\"timestamp_utc\""));
        assert!(json.contains("\"session_id\""));
        assert!(json.contains("\"direction\""));
        assert!(json.contains("\"raw_line_redacted\""));
        assert!(json.contains("\"jsonrpc\""));
        assert!(json.contains("\"redaction_applied\""));
    }

    #[test]
    fn test_proxy_event_labels() {
        let fixture = FixtureLine::proxy_event(
            "test-session".to_string(),
            None,
            "connect",
            Some("miner_connected"),
        );
        assert_eq!(fixture.direction, "proxy_event");
        assert!(fixture.notes.unwrap().contains("connect"));
    }
}
