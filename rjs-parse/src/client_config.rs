use parse::ConfigParseError;
use parse::ParsedConfig;
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

///
/// This struct represents all of the fields that would be collected by
/// parsing the requirejs-config.js file
///
///
/// # Examples
///
/// ```rust
/// extern crate serde_json;
/// extern crate rjs;
///
/// use serde_json;
/// use rjs::RequireJsClientConfig;
///
/// let rjs = RequireJsClientConfig::default();
/// let actual = serde_json::to_value(&rjs).unwrap();
///
/// let expected = r#"{
///     "deps": [],
///     "map": {},
///     "config": {},
///     "shim": {},
///     "paths": {}
/// }"#;
///
/// let expected: serde_json::Value = serde_json::from_str(&expected).unwrap();
///
/// assert_eq!(actual, expected);
/// ```
///
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RequireJsClientConfig {
    pub deps: Vec<BuildModuleId>,
    pub map: serde_json::Value,
    pub config: serde_json::Value,
    pub shim: serde_json::Value,
    pub paths: HashMap<String, String>,
}

impl RequireJsClientConfig {
    ///
    /// This is the top-level api for creating a client config
    /// from the magento-generated file. Note: This will *not* apply any build-specific
    /// modifications. It just gathers/merges the data in a last-one-wins style.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rjs::*;
    /// let input = r#"
    ///     (function () {
    ///         var config = {
    ///             paths: {
    ///                 paypal: "http://example.com/paypal"
    ///             },
    ///             deps: ["one", "two"]
    ///         };
    ///         require.config(config);
    ///     })();
    ///     (function () {
    ///         var config = {
    ///             deps: ["three"]
    ///         };
    ///         require.config(config);
    ///     })();
    /// "#;
    ///
    /// let rjs_cfg = RequireJsClientConfig::from_generated_string(input).expect("should parse");
    ///
    /// // assert that `deps` were merged
    /// assert_eq!(rjs_cfg.deps,
    ///     vec![
    ///         "one".to_string(),
    ///         "two".to_string(),
    ///         "three".to_string(),
    ///     ]
    /// );
    ///
    /// // assert that the path exists, but was not altered
    /// assert_eq!(rjs_cfg.paths.get("paypal").expect("unwrap"), "http://example.com/paypal");
    /// ```
    ///
    /// ```
    /// # use rjs::*;
    /// let input = include_str!("../test/fixtures/requirejs-config-generated.js");
    /// let rjx = RequireJsClientConfig::from_generated_string(input).expect("parsed");
    /// assert_eq!(
    ///     rjx.deps,
    ///     vec![
    ///         "jquery/jquery.mobile.custom",
    ///         "mage/common",
    ///         "mage/dataPost",
    ///         "mage/bootstrap",
    ///         "jquery/jquery-migrate",
    ///         "mage/translate-inline",
    ///         "Magento_Theme/js/responsive",
    ///         "Magento_Theme/js/theme",
    ///     ]
    /// );
    /// ```
    ///
    pub fn from_generated_string(
        input: impl Into<String>,
    ) -> Result<RequireJsClientConfig, ConfigParseError> {
        let output = ParsedConfig::from_str(input)?;
        let as_serde = serde_json::to_value(&output).map_err(|_e| ConfigParseError::Serialize)?;
        let as_rjs: RequireJsClientConfig =
            serde_json::from_value(as_serde).map_err(|_e| ConfigParseError::Conversion)?;
        Ok(as_rjs)
    }
    ///
    /// Helper to always give a string output, even for errors
    ///
    pub fn to_string(&self) -> Result<String, String> {
        match serde_json::to_string_pretty(&self) {
            Ok(s) => Ok(s),
            Err(e) => Err(e.to_string()),
        }
    }
    ///
    /// Helper to update the client config in place, when it would
    /// be wrapped in Arc<Mutex<..>>
    ///
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BuildModule {
    pub name: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub create: bool,
}

pub type BuildModuleId = String;
