use from_file::FromFile;
use presets::m2::module_meta_data::ModuleData;
use presets::m2::requirejs_config::RequireJsClientConfig;

#[derive(Serialize, Deserialize, Default)]
pub struct SeedData {
    pub client_config: RequireJsClientConfig,
    pub module_items: Vec<ModuleData>,
}

impl FromFile for SeedData {}
