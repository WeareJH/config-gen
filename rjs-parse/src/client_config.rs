use parse::ConfigParseError;
use parse::ParsedConfig;
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ModuleId = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RequireJsClientConfig {
    pub deps: Vec<ModuleId>,
    pub map: serde_json::Value,
    pub config: serde_json::Value,
    pub shim: serde_json::Value,
    pub paths: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub create: bool,
}

impl RequireJsClientConfig {
    pub fn from_generated_string(
        input: impl Into<String>,
    ) -> Result<RequireJsClientConfig, ConfigParseError> {
        let output = ParsedConfig::from_str(input)?;
        let as_serde = serde_json::to_value(&output).map_err(|_e| ConfigParseError::Serialize)?;
        let as_rjs: RequireJsClientConfig =
            serde_json::from_value(as_serde).map_err(|_e| ConfigParseError::Conversion)?;
        Ok(as_rjs)
    }
    pub fn to_string(&self) -> Result<String, String> {
        match serde_json::to_string_pretty(&self) {
            Ok(s) => Ok(s),
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn update_in_place(
        input: impl Into<String>,
        item: Arc<Mutex<RequireJsClientConfig>>,
    ) -> Result<(), String> {
        match RequireJsClientConfig::from_generated_string(input.into()) {
            Ok(rjs) => {
                let mut w = item
                    .lock()
                    .map_err(|_e| "unwrapped client_config_clone".to_string())?;

                w.deps = rjs.deps.clone();
                w.config = rjs.config.clone();
                w.shim = rjs.shim.clone();
                w.paths = rjs.paths.clone();
                w.map = rjs.map.clone();

                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

impl Default for RequireJsClientConfig {
    fn default() -> RequireJsClientConfig {
        RequireJsClientConfig {
            deps: vec![],
            map: json!({}),
            config: json!({}),
            shim: json!({}),
            paths: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_require_js_config() {
        let input = r#"{
            "deps": [],
            "map": {},
            "config": {},
            "shim": {},
            "paths": {}
        }"#;
        let r = RequireJsClientConfig::default();
        let actual = serde_json::to_value(&r).unwrap();
        let expected: serde_json::Value = serde_json::from_str(&input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_e2e2() {
        let input = include_str!("../test/fixtures/requirejs-config-generated.js");
        let rjx = RequireJsClientConfig::from_generated_string(input).expect("allgood");
        assert_eq!(
            rjx.deps,
            vec![
                "jquery/jquery.mobile.custom",
                "mage/common",
                "mage/dataPost",
                "mage/bootstrap",
                "jquery/jquery-migrate",
                "mage/translate-inline",
                "Magento_Theme/js/responsive",
                "Magento_Theme/js/theme",
            ]
        );
    }
}
