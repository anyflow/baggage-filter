use log::{debug, error, info};
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde_json::Value;
use std::collections::HashMap;
use std::rc::Rc;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(BaggageRoot::default())
    });
}}

#[derive(Default)]
struct BaggageRoot {
    headers: Rc<Vec<String>>,
}

impl Context for BaggageRoot {
    fn on_done(&mut self) -> bool {
        info!("[bf] baggage-filter terminated");
        true
    }
}

impl RootContext for BaggageRoot {
    fn on_vm_start(&mut self, _vm_configuration_size: usize) -> bool {
        info!("[bf] baggage-filter initialized");
        true
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn on_configure(&mut self, _: usize) -> bool {
        debug!("[bf] Configuring baggage-filter");
        let config_bytes = match self.get_plugin_configuration() {
            Some(bytes) => bytes,
            None => {
                error!("[bf] No plugin configuration found");
                return false;
            }
        };

        let config_str = match String::from_utf8(config_bytes) {
            Ok(s) => s,
            Err(e) => {
                error!("[bf] Failed to convert bytes to UTF-8 string: {}", e);
                return false;
            }
        };

        let config: Value = match serde_json::from_str(&config_str) {
            Ok(v) => v,
            Err(e) => {
                error!("[bf] Failed to parse JSON configuration: {}", e);
                return false;
            }
        };

        match self.configure(&config) {
            Ok(_) => {
                info!(
                    "[bf] Configuration successful, configured headers: {:?}",
                    self.headers
                );
                true
            }
            Err(e) => {
                error!("[bf] Configuration failed: {}", e);
                false
            }
        }
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        debug!("[bf] Creating HTTP context");
        Some(Box::new(BaggageFilter {
            headers: Rc::clone(&self.headers),
        }))
    }
}

impl BaggageRoot {
    fn configure(&mut self, config: &Value) -> Result<(), Box<dyn std::error::Error>> {
        self.headers = Rc::new(
            config["headers"]
                .as_array()
                .ok_or("Missing 'headers' array in configuration")?
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>(),
        );
        Ok(())
    }
}

struct BaggageFilter {
    headers: Rc<Vec<String>>,
}

impl Context for BaggageFilter {}

impl HttpContext for BaggageFilter {
    fn on_http_request_headers(&mut self, _nheaders: usize, _end_of_stream: bool) -> Action {
        let mut baggage_map = match self.get_http_request_header("baggage") {
            Some(baggage) => self.get_baggage_value(&baggage),
            None => HashMap::new(),
        };
        for header in self.headers.iter() {
            if let Some(value) = self.get_http_request_header(header) {
                baggage_map.insert(header.clone(), value);
            }
        }

        let new_baggage = self.create_baggage_value(&baggage_map);
        self.set_http_request_header("baggage", Some(&new_baggage));
        debug!("[bf] Set new baggage: {}", new_baggage);

        Action::Continue
    }
}

impl BaggageFilter {
    fn get_baggage_value(&mut self, baggage: &str) -> HashMap<String, String> {
        baggage
            .split(',')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                match (parts.next(), parts.next()) {
                    (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                    _ => {
                        debug!("[bf] Invalid baggage pair: {}", pair);
                        None
                    }
                }
            })
            .collect()
    }

    fn create_baggage_value(&mut self, baggage_map: &HashMap<String, String>) -> String {
        baggage_map
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const TEST_CONFIG: &str = r#"{
        "headers": [
            "user-id",
            "trace-id"
        ]
    }"#;

    // Existing test for valid configuration
    #[test]
    fn test_configure() {
        let mut root_ctx = BaggageRoot::default();
        root_ctx
            .configure(&serde_json::from_str(TEST_CONFIG).unwrap())
            .unwrap();
        assert_eq!(
            root_ctx.headers,
            Rc::new(vec!["user-id".to_string(), "trace-id".to_string()])
        );
    }

    // Test for configuration with missing "headers" field
    #[test]
    fn test_configure_no_headers() {
        let mut root_ctx = BaggageRoot::default();
        let config = r#"{}"#;
        let result = root_ctx.configure(&serde_json::from_str(config).unwrap());
        assert!(
            result.is_err(),
            "Expected an error when 'headers' is missing"
        );
    }

    // Test for configuration where "headers" is not an array
    #[test]
    fn test_configure_headers_not_array() {
        let mut root_ctx = BaggageRoot::default();
        let config = r#"{"headers": "not an array"}"#;
        let result = root_ctx.configure(&serde_json::from_str(config).unwrap());
        assert!(
            result.is_err(),
            "Expected an error when 'headers' is not an array"
        );
    }

    // Test parsing a baggage header into a map
    #[test]
    fn test_get_baggage_value() {
        let mut filter = BaggageFilter {
            headers: Rc::new(vec![]),
        };
        let baggage = "key1=value1,key2=value2";
        let map = filter.get_baggage_value(baggage);
        assert_eq!(map.get("key1"), Some(&"value1".to_string()));
        assert_eq!(map.get("key2"), Some(&"value2".to_string()));
        assert_eq!(map.len(), 2, "Expected exactly 2 key-value pairs");
    }

    // Test edge cases for baggage parsing
    #[test]
    fn test_get_baggage_value_edge_cases() {
        let mut filter = BaggageFilter {
            headers: Rc::new(vec![]),
        };

        // Empty baggage string
        let map = filter.get_baggage_value("");
        assert!(map.is_empty(), "Expected empty map for empty baggage");

        // Single key-value pair
        let map = filter.get_baggage_value("key=value");
        assert_eq!(map.get("key"), Some(&"value".to_string()));
        assert_eq!(map.len(), 1, "Expected exactly 1 key-value pair");

        // Malformed pair (no value)
        let map = filter.get_baggage_value("key=value,invalid");
        assert_eq!(map.get("key"), Some(&"value".to_string()));
        assert_eq!(map.get("invalid"), None);
        assert_eq!(map.len(), 1, "Expected only valid pairs to be parsed");

        // Pair with multiple equals signs
        let map = filter.get_baggage_value("key=value=extra");
        assert_eq!(map.get("key"), Some(&"value=extra".to_string()));
        assert_eq!(
            map.len(),
            1,
            "Expected 1 pair with remaining string as value"
        );
    }

    // Test creating a baggage string from a map
    #[test]
    fn test_create_baggage_value() {
        let mut filter = BaggageFilter {
            headers: Rc::new(vec![]),
        };
        let mut map = HashMap::new();
        map.insert("key1".to_string(), "value1".to_string());
        map.insert("key2".to_string(), "value2".to_string());
        let baggage = filter.create_baggage_value(&map);
        // Since HashMap order is not guaranteed, check both possible outcomes
        assert!(
            baggage == "key1=value1,key2=value2" || baggage == "key2=value2,key1=value1",
            "Unexpected baggage string: {}",
            baggage
        );
    }

    // Test creating a baggage string from an empty map
    #[test]
    fn test_create_baggage_value_empty() {
        let mut filter = BaggageFilter {
            headers: Rc::new(vec![]),
        };
        let map = HashMap::new();
        let baggage = filter.create_baggage_value(&map);
        assert_eq!(baggage, "", "Expected empty string for empty map");
    }
}
