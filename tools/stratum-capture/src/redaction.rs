use crate::args::Args;
use serde_json::Value;

pub struct Redactor {
    wallet: Option<String>,
    worker: Option<String>,
    password: Option<String>,
}

impl Redactor {
    pub fn new(args: &Args) -> Self {
        Self {
            wallet: args.redact_wallet.clone(),
            worker: args.redact_worker.clone(),
            password: args.redact_password.clone(),
        }
    }

    /// Redacts exact strings in a raw string line.
    pub fn redact_raw_line(&self, line: &str) -> String {
        let mut redacted = line.to_string();

        if let Some(w) = &self.wallet {
            if !w.is_empty() {
                redacted = redacted.replace(w, "<REDACTED_WALLET>");
            }
        }

        if let Some(w) = &self.worker {
            if !w.is_empty() {
                redacted = redacted.replace(w, "<REDACTED_WORKER>");
            }
        }

        if let Some(p) = &self.password {
            if !p.is_empty() {
                redacted = redacted.replace(p, "<REDACTED_PASSWORD>");
            }
        }

        redacted
    }

    /// Recursively redacts sensitive JSON keys.
    pub fn redact_json_value(&self, value: &mut Value) -> bool {
        let mut mutated = false;
        match value {
            Value::Object(map) => {
                let keys_to_redact = [
                    "user", "pass", "password", "worker", "wallet", "login", "address",
                ];
                for (k, v) in map.iter_mut() {
                    if keys_to_redact.contains(&k.to_lowercase().as_str()) {
                        let redacted_val = if k.to_lowercase().contains("pass") {
                            "<REDACTED_PASSWORD>"
                        } else if k.to_lowercase().contains("worker") {
                            "<REDACTED_WORKER>"
                        } else if k.to_lowercase().contains("wallet")
                            || k.to_lowercase().contains("address")
                        {
                            "<REDACTED_WALLET>"
                        } else {
                            "<REDACTED>"
                        };

                        if *v != Value::String(redacted_val.to_string()) {
                            *v = Value::String(redacted_val.to_string());
                            mutated = true;
                        }
                    } else {
                        if self.redact_json_value(v) {
                            mutated = true;
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for v in arr.iter_mut() {
                    if self.redact_json_value(v) {
                        mutated = true;
                    }
                }
            }
            Value::String(s) => {
                // Also check if the string contains any exact secrets
                let redacted = self.redact_raw_line(s);
                if redacted != *s {
                    *value = Value::String(redacted);
                    mutated = true;
                }
            }
            _ => {}
        }
        mutated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_args() -> Args {
        Args {
            listen: "127.0.0.1:3333".to_string(),
            pool: "pool".to_string(),
            output: "out".to_string(),
            redact_wallet: Some("my_secret_wallet".to_string()),
            redact_worker: Some("my_worker".to_string()),
            redact_password: Some("super_secret".to_string()),
            session_label: None,
            max_lines: None,
            max_seconds: None,
            pretty_log: false,
            dry_run: false,
        }
    }

    #[test]
    fn test_raw_line_redaction() {
        let redactor = Redactor::new(&test_args());
        let line = "User my_secret_wallet and worker my_worker with pass super_secret";
        let redacted = redactor.redact_raw_line(line);
        assert_eq!(
            redacted,
            "User <REDACTED_WALLET> and worker <REDACTED_WORKER> with pass <REDACTED_PASSWORD>"
        );
    }

    #[test]
    fn test_json_key_redaction() {
        let redactor = Redactor::new(&test_args());
        let mut val = json!({
            "id": 1,
            "method": "mining.authorize",
            "params": {
                "user": "some_user",
                "password": "some_password",
                "wallet": "some_wallet",
                "other": "safe_value"
            }
        });

        assert!(redactor.redact_json_value(&mut val));

        let expected = json!({
            "id": 1,
            "method": "mining.authorize",
            "params": {
                "user": "<REDACTED>",
                "password": "<REDACTED_PASSWORD>",
                "wallet": "<REDACTED_WALLET>",
                "other": "safe_value"
            }
        });

        assert_eq!(val, expected);
    }

    #[test]
    fn test_json_string_value_redaction() {
        let redactor = Redactor::new(&test_args());
        let mut val = json!({
            "id": 1,
            "method": "mining.authorize",
            "params": ["my_secret_wallet.my_worker", "super_secret"]
        });

        assert!(redactor.redact_json_value(&mut val));

        let expected = json!({
            "id": 1,
            "method": "mining.authorize",
            "params": ["<REDACTED_WALLET>.<REDACTED_WORKER>", "<REDACTED_PASSWORD>"]
        });

        assert_eq!(val, expected);
    }
}
