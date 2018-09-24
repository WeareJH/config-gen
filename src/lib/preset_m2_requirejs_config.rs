extern crate serde_json;

use serde_json::{Value, Error};
use std::collections::HashMap;
use preset_m2_config_gen::Module;

type ModuleId = String;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RequireJsMergedConfig {
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub deps: Vec<ModuleId>,
    pub map: serde_json::Value,
    pub config: serde_json::Value,
    pub shim: serde_json::Value,
    pub paths: HashMap<String, String>,
    pub modules: Option<Vec<Module>>
}

#[test]
fn test_parse_incoming_from_browser() {
    let input = include_bytes!("../../test/fixtures/example-post.json");
    let s: RequireJsMergedConfig = serde_json::from_slice(input).unwrap();
    assert_eq!(s.deps, vec![
        "Magento_Theme/js/responsive",
        "Magento_Theme/js/theme"
    ]);
    assert_eq!(s.base_url, "/static/version1517228438/frontend/Magento/luma/en_US/");
    assert_eq!(s.paths.get("jquery/ui"), Some(&"jquery/jquery-ui".to_string()));
}