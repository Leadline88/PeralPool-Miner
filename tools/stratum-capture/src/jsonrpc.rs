use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcShape {
    pub method: Option<String>,
    pub id: Option<String>,
    pub params_shape: Option<String>,
    pub result_shape: Option<String>,
    pub error_shape: Option<String>,
}

pub fn extract_shape(value: &Value) -> JsonRpcShape {
    let method = value
        .get("method")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let id = value.get("id").map(|v| match v {
        Value::Null => "null".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        _ => format!("other({})", shape_of(v)),
    });

    // Determine if it's likely a request/notification or a response
    let has_method = method.is_some();
    let has_result = value.get("result").is_some();
    let has_error = value.get("error").is_some();

    let (params_shape, result_shape, error_shape) = if has_method {
        (value.get("params").map(shape_of), None, None)
    } else if has_result || has_error {
        (
            None,
            value.get("result").map(shape_of),
            value.get("error").map(shape_of),
        )
    } else {
        // Fallback for non-standard or malformed messages
        (
            value.get("params").map(shape_of),
            value.get("result").map(shape_of),
            value.get("error").map(shape_of),
        )
    };

    JsonRpcShape {
        method,
        id,
        params_shape,
        result_shape,
        error_shape,
    }
}

fn shape_of(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "bool".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(arr) => format!("array({})", arr.len()),
        Value::Object(obj) => {
            let mut keys: Vec<_> = obj.keys().cloned().collect();
            keys.sort();
            format!("object([{}])", keys.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_subscribe_request() {
        let val = json!({
            "id": 1,
            "method": "mining.subscribe",
            "params": ["PeralPoolMiner/0.1.0"]
        });
        let shape = extract_shape(&val);
        assert_eq!(shape.method.as_deref(), Some("mining.subscribe"));
        assert_eq!(shape.params_shape.as_deref(), Some("array(1)"));
    }

    #[test]
    fn test_extract_authorize_request() {
        let val = json!({
            "id": 2,
            "method": "mining.authorize",
            "params": ["wallet.worker", "password"]
        });
        let shape = extract_shape(&val);
        assert_eq!(shape.method.as_deref(), Some("mining.authorize"));
        assert_eq!(shape.params_shape.as_deref(), Some("array(2)"));
    }

    #[test]
    fn test_extract_notify_notification() {
        let val = json!({
            "id": null,
            "method": "mining.notify",
            "params": ["job_id", "prevhash", "coinb1", "coinb2", [], "version", "nbits", "ntime", true]
        });
        let shape = extract_shape(&val);
        assert_eq!(shape.method.as_deref(), Some("mining.notify"));
        assert_eq!(shape.id.as_deref(), Some("null"));
        assert_eq!(shape.params_shape.as_deref(), Some("array(9)"));
    }

    #[test]
    fn test_extract_set_difficulty() {
        let val = json!({
            "id": null,
            "method": "mining.set_difficulty",
            "params": [1024]
        });
        let shape = extract_shape(&val);
        assert_eq!(shape.method.as_deref(), Some("mining.set_difficulty"));
        assert_eq!(shape.params_shape.as_deref(), Some("array(1)"));
    }

    #[test]
    fn test_extract_submit_request() {
        let val = json!({
            "id": 10,
            "method": "mining.submit",
            "params": ["user", "job", "extranonce2", "ntime", "nonce"]
        });
        let shape = extract_shape(&val);
        assert_eq!(shape.method.as_deref(), Some("mining.submit"));
        assert_eq!(shape.params_shape.as_deref(), Some("array(5)"));
    }

    #[test]
    fn test_extract_accepted_response() {
        let val = json!({
            "id": 10,
            "result": true,
            "error": null
        });
        let shape = extract_shape(&val);
        assert_eq!(shape.id.as_deref(), Some("number"));
        assert_eq!(shape.result_shape.as_deref(), Some("bool"));
        assert_eq!(shape.error_shape.as_deref(), Some("null"));
    }

    #[test]
    fn test_extract_rejected_response() {
        let val = json!({
            "id": 11,
            "result": null,
            "error": [21, "Job not found", null]
        });
        let shape = extract_shape(&val);
        assert_eq!(shape.id.as_deref(), Some("number"));
        assert_eq!(shape.result_shape.as_deref(), Some("null"));
        assert_eq!(shape.error_shape.as_deref(), Some("array(3)"));
    }

    #[test]
    fn test_extract_invalid_json() {
        let val = json!("not an object");
        let shape = extract_shape(&val);
        assert!(shape.method.is_none());
        assert!(shape.params_shape.is_none());
    }
}
