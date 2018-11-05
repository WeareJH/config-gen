extern crate serde_json;
extern crate serde_yaml;

use from_file::FromFile;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BundleConfig {
    pub bundles: Vec<ConfigItem>,
    pub module_blacklist: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum BundleConfigError {
    SerdeJsonError(serde_json::Error),
    SerdeYamlError(serde_yaml::Error),
}

impl BundleConfig {
    ///
    /// # Examples
    ///
    /// ```
    /// use rjs::*;
    /// use rjs::bundle_config::*;
    /// let input = r#"
    ///     {"bundles": [{"name": "main", "children": [], "urls": []}]}
    /// "#;
    /// let actual = BundleConfig::from_json_string(input).expect("valid json parse");
    /// assert_eq!(actual, BundleConfig{
    ///     bundles: vec![
    ///         ConfigItem {
    ///             name: "main".to_string(),
    ///             urls: vec![],
    ///             children: vec![],
    ///         }
    ///     ],
    ///     module_blacklist: None,
    /// });
    /// ```
    ///
    pub fn from_json_string(input: impl Into<String>) -> Result<BundleConfig, BundleConfigError> {
        serde_json::from_str(&input.into()).map_err(|e| BundleConfigError::SerdeJsonError(e))
    }

    ///
    /// # Examples
    ///
    /// ```
    /// use rjs::*;
    /// use rjs::bundle_config::*;
    /// let input = r#"
    ///     bundles:
    ///       - name: main
    ///         children: []
    ///         urls: []
    /// "#;
    /// let actual = BundleConfig::from_yaml_string(input).expect("valid yaml parse");
    /// assert_eq!(actual, BundleConfig{
    ///     bundles: vec![
    ///         ConfigItem {
    ///             name: "main".to_string(),
    ///             urls: vec![],
    ///             children: vec![],
    ///         }
    ///     ],
    ///     module_blacklist: None,
    /// })
    /// ```
    ///
    pub fn from_yaml_string(input: impl Into<String>) -> Result<BundleConfig, BundleConfigError> {
        serde_yaml::from_str(&input.into()).map_err(|e| BundleConfigError::SerdeYamlError(e))
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct ConfigItem {
    pub name: String,
    pub urls: Vec<String>,
    pub children: Vec<ConfigItem>,
}

impl FromFile for BundleConfig {}
impl Default for BundleConfig {
    fn default() -> BundleConfig {
        BundleConfig {
            bundles: vec![],
            module_blacklist: None,
        }
    }
}

impl<'a> Into<BundleConfig> for &'a str {
    fn into(self) -> BundleConfig {
        let items: BundleConfig = match serde_yaml::from_str(&self) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("{}", e);
                BundleConfig::default()
            }
        };
        items
    }
}
