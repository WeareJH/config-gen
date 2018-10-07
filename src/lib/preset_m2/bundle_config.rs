extern crate serde_yaml;

use from_file::FromFile;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BundleConfig {
    pub bundles: Vec<ConfigItem>,
    pub module_blacklist: Option<Vec<String>>,
}

impl FromFile for BundleConfig {}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ConfigItem {
    pub name: String,
    pub urls: Vec<String>,
    pub children: Vec<ConfigItem>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Outgoing {
    pub bundles: Vec<Module>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub create: bool,
}
