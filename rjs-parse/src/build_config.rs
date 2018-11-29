use bundle_config::BundleConfig;
use modules;
use modules::BuildModuleId;
use parse::ConfigParseError;
use serde_json;
use std::collections::HashMap;
use BuildModule;
use RequireJsClientConfig;
use modules::ModuleData;

///
/// This struct is a combination of RequireJsClientConfig
/// with some added fields to enable the RequireJS optimizer
/// to run
///
/// # Examples
///
/// ```rust
/// extern crate serde_json;
/// extern crate rjs;
///
/// use serde_json;
/// use rjs::RequireJsBuildConfig;
///
/// let rjs = RequireJsBuildConfig::default();
/// let actual = serde_json::to_value(&rjs).unwrap();
///
/// let expected = r#"{
///    "generateSourceMaps": true,
///    "inlineText": true,
///    "optimize": "uglify",
///    "deps": [],
///    "map": {},
///    "config": {},
///    "shim": {},
///    "paths": {},
///    "modules": [
///      {
///        "name": "requirejs/require",
///        "include": [],
///        "exclude": [],
///        "create": false
///      }
///    ]
/// }"#;
///
/// let expected: serde_json::Value = serde_json::from_str(&expected).unwrap();
///
/// assert_eq!(actual, expected);
/// ```
///
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RequireJsBuildConfig {
    #[serde(rename = "generateSourceMaps")]
    pub generate_source_maps: Option<bool>,

    #[serde(default = "default_inline_text", rename = "inlineText")]
    pub inline_text: Option<bool>,

    #[serde(default = "default_optimize")]
    pub optimize: Option<String>,

    //
    // These fields come from `RequireJsClientConfig`
    //
    pub deps: Vec<BuildModuleId>,

    #[serde(default = "default_modules")]
    pub modules: Option<Vec<BuildModule>>,

    pub map: serde_json::Value,
    pub config: serde_json::Value,
    pub shim: serde_json::Value,
    pub paths: HashMap<String, String>,
}

impl RequireJsBuildConfig {
    ///
    /// This is the top-level api - it accepts any Javascript
    /// input, and if the structure matches that seen
    /// in the magento-generated file, then it's parsed
    /// and converted into a [RequireJsBuildConfig]  that
    /// is valid to be used with the optimizer
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
    /// let rjs_cfg = RequireJsBuildConfig::from_generated_string(input).expect("should parse");
    /// assert_eq!(rjs_cfg.paths.get("paypal").expect("unwrap"), "empty:");
    /// ```
    ///
    pub fn from_generated_string(
        input: impl Into<String>,
    ) -> Result<RequireJsBuildConfig, ConfigParseError> {
        let client = RequireJsClientConfig::from_generated_string(input)?;
        let mut output = RequireJsBuildConfig::default();
        output.paths = RequireJsBuildConfig::strip_paths(&client.paths);
        output.shim = client.shim;
        output.config = client.config;
        output.map = client.map;
        output.deps = client.deps;
        Ok(output)
    }
    ///
    /// Add a bundle_config data structure - this will popuplate
    /// the `modules` field on [RequireJsBuildConfig]
    ///
    /// # Examples
    ///
    /// ```
    /// use rjs::*;
    /// use rjs::bundle_config::*;
    ///
    /// let input = include_str!("../test/fixtures/requirejs-config-generated.js");
    /// let rjx = RequireJsBuildConfig::from_generated_string(input).expect("parsed");
    /// let bundle_config_str = r#"
    /// bundles:
    ///     - name: bundles/main
    ///       children: []
    ///       urls: ["/"]
    /// "#;
    /// let rjx2 = rjx.create_modules(&bundle_config_str.into(), &vec![]);
    /// assert_eq!(rjx2.modules.expect("has modules"), vec![
    ///     BuildModule {
    ///         name: "requirejs/require".to_string(),
    ///         include: vec![],
    ///         exclude: vec![],
    ///         create: false
    ///     },
    ///     BuildModule {
    ///         name: "bundles/main".to_string(),
    ///         include: vec![],
    ///         exclude: vec![
    ///             "requirejs/require".to_string()
    ///         ],
    ///         create: true
    ///     }
    /// ]);
    /// ```
    ///
    pub fn create_modules(
        mut self,
        bundle_config: &BundleConfig,
        req_log: &Vec<ModuleData>
    ) -> RequireJsBuildConfig {
        self.modules = Some(modules::generate_modules(req_log, bundle_config));
        self
    }

    ///
    /// Just a passthrough for `from_generated_string` above
    ///
    pub fn from_str(input: impl Into<String>) -> Result<RequireJsBuildConfig, ConfigParseError> {
        RequireJsBuildConfig::from_generated_string(input)
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
    /// For the build process to work, any 'paths' within the config
    /// must be replaced with `empty:`
    ///
    /// # Examples
    ///
    /// ```
    /// # use rjs::*;
    /// # use std::collections::HashMap;
    /// let mut paths: HashMap<String, String> = HashMap::new();
    /// paths.insert("one".into(), "one/one".into());
    /// paths.insert("two".into(), "http://two.com/two".into());
    ///
    /// let mut expected: HashMap<String, String> = HashMap::new();
    /// expected.insert("one".into(), "one/one".into());
    /// expected.insert("two".into(), "empty:".into());
    ///
    /// let actual = RequireJsBuildConfig::strip_paths(&paths);
    /// assert_eq!(actual, expected);
    /// ```
    ///
    pub fn strip_paths(paths: &HashMap<String, String>) -> HashMap<String, String> {
        let mut hm: HashMap<String, String> = HashMap::new();

        for (key, value) in paths.iter() {
            if value.starts_with("http://")
                || value.starts_with("https://")
                || value.starts_with("//")
            {
                hm.insert(key.clone(), "empty:".to_string());
            } else {
                hm.insert(key.clone(), value.clone());
            }
        }

        hm
    }
    ///
    /// Convert a module list into the bundle-loader code that's
    /// needed to load 'additional' bundles.
    ///
    /// Note: there's special logic here to *not* include
    /// any modules that have mixins declared against them, as this
    /// would cause a bug at run time since the module in question
    /// would not trigger the custom `mixins` code added by Magento at run time.
    ///
    /// # Examples
    ///
    /// ```
    /// use rjs::{RequireJsBuildConfig, BuildModule};
    /// # use std::collections::HashMap;
    /// let list = RequireJsBuildConfig::bundle_loaders(
    ///     vec!["js/shane".to_string()],
    ///     vec![
    ///         BuildModule {
    ///             name: String::from("requirejs/require"),
    ///             include: vec![],
    ///             exclude: vec![],
    ///             create: false,
    ///         },
    ///         BuildModule {
    ///             name: String::from("bundle/base"),
    ///             include: vec!["js/shane".to_string(), "js/kittie".to_string()],
    ///             exclude: vec![],
    ///             create: true,
    ///         },
    ///         BuildModule {
    ///             name: String::from("bundle/product"),
    ///             include: vec!["js/gallery".to_string(), "js/zoomer".to_string()],
    ///             exclude: vec![],
    ///             create: true,
    ///         },
    ///     ],
    /// );
    /// let expected = r#"require.config({
    ///   bundles: {
    ///     "bundle/base": [
    ///          // mixin trigger: "js/shane",
    ///         "js/kittie",
    ///     ]
    ///   }
    /// });
    /// require.config({
    ///   bundles: {
    ///     "bundle/product": [
    ///         "js/gallery",
    ///         "js/zoomer",
    ///     ]
    ///   }
    /// });"#;
    /// assert_eq!(list, expected);
    /// ```
    ///
    pub fn bundle_loaders(mixins: Vec<String>, modules: Vec<BuildModule>) -> String {
        let items: Vec<String> = modules
            .iter()
            .filter(|m| m.name.as_str() != "requirejs/require")
            .map(|module| {
                let module_list: Vec<String> = module
                    .include
                    .iter()
                    .map(|name| {
                        let is_mixin_trigger = mixins.contains(&name);
                        match is_mixin_trigger {
                            true => format!("         // mixin trigger: \"{}\",", name),
                            false => format!("        \"{}\",", name),
                        }
                    })
                    .collect();

                format!(
                    "require.config({{\n  bundles: {{\n    \"{}\": [\n{}\n    ]\n  }}\n}});",
                    module.name,
                    module_list.join("\n")
                )
            })
            .collect();

        if items.len() > 0 {
            items.join("\n")
        } else {
            include_str!("./js/no-loaders.js").to_string()
        }
    }
    ///
    /// Walk the mixins fields and flatten to a simple Vec<String>
    ///
    /// # Examples
    ///
    /// ```
    /// use rjs::{RequireJsBuildConfig};
    /// let input = include_str!("../test/fixtures/requirejs-config-generated.js");
    /// let s = RequireJsBuildConfig::from_generated_string(input).expect("fixture unwrap");
    /// assert_eq!(
    ///     RequireJsBuildConfig::collect_mixins(&s.config),
    ///     vec![
    ///         "Magento_Checkout/js/action/place-order",
    ///         "Magento_Checkout/js/action/set-payment-information",
    ///         "jquery/jstree/jquery.jstree",
    ///     ]
    /// );
    /// ```
    ///
    pub fn collect_mixins(val: &serde_json::Value) -> Vec<String> {
        match *val {
            serde_json::Value::Object(ref v) => match v.get("mixins") {
                Some(f) => match f {
                    serde_json::Value::Object(ref v) => {
                        let names: Vec<String> = v.iter().map(|(key, _)| key.to_string()).collect();
                        names
                    }
                    _ => vec![],
                },
                None => vec![],
            },
            _ => vec![],
        }
    }
}

impl Default for RequireJsBuildConfig {
    fn default() -> RequireJsBuildConfig {
        RequireJsBuildConfig {
            deps: vec![],
            map: json!({}),
            config: json!({}),
            shim: json!({}),
            paths: HashMap::new(),
            generate_source_maps: Some(true),
            inline_text: Some(true),
            optimize: Some("uglify".into()),
            modules: Some(vec![BuildModule {
                name: String::from("requirejs/require"),
                include: Vec::new(),
                exclude: Vec::new(),
                create: false,
            }]),
        }
    }
}

fn default_optimize() -> Option<String> {
    Some("uglify".to_string())
}
fn default_inline_text() -> Option<bool> {
    Some(true)
}
fn default_modules() -> Option<Vec<BuildModule>> {
    Some(vec![])
}
