use log::{debug, error, info};
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde_json::Value;
use std::rc::Rc;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(BaggageRootContext::default())
    });
}}
#[derive(Default)]
struct BaggageRootContext {
    headers: Rc<Vec<String>>,
}

impl Context for BaggageRootContext {
    fn on_done(&mut self) -> bool {
        info!("[bf] baggage-filter terminated");
        true
    }
}

impl RootContext for BaggageRootContext {
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
                info!("[bf] Configuration successful");
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
        Some(Box::new(BaggageHttpContext {
            headers: Rc::clone(&self.headers),
        }))
    }
}

impl BaggageRootContext {
    fn configure(&mut self, config: &Value) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

struct BaggageHttpContext {
    headers: Rc<Vec<String>>,
}

impl Context for BaggageHttpContext {}

impl HttpContext for BaggageHttpContext {
    fn on_http_request_headers(&mut self, _nheaders: usize, _end_of_stream: bool) -> Action {
        Action::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const TEST_CONFIG: &str = r#"{
        "headers": [
            "x-message-id"
        ]
    }"#;
}
