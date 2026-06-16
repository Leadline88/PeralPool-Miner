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
        _ => "other".to_string(),
    });

    let params_shape = value.get("params").map(shape_of);
    let result_shape = value.get("result").map(shape_of);
    let error_shape = value.get("error").map(shape_of);

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
    fn test_extract_request_shape() {
        let val = json!({
            "id": 1,
            "method": "mining.submit",
            "params": ["user", "job", "extranonce2", "ntime", "nonce"]
        });

        let shape = extract_shape(&val);
        assert_eq!(shape.method.unwrap(), "mining.submit");
        assert_eq!(shape.id.unwrap(), "number");
        assert_eq!(shape.params_shape.unwrap(), "array(5)");
        assert!(shape.result_shape.is_none());
        assert!(shape.error_shape.is_none());
    }

    #[test]
    fn test_extract_response_shape() {
        let val = json!({
            "id": null,
            "result": {
                "status": "ok",
                "hashrate": 100
            },
            "error": null
        });

        let shape = extract_shape(&val);
        assert!(shape.method.is_none());
        assert_eq!(shape.id.unwrap(), "null");
        assert!(shape.params_shape.is_none());
        assert_eq!(shape.result_shape.unwrap(), "object([hashrate, status])");
        assert_eq!(shape.error_shape.unwrap(), "null");
    }
}
