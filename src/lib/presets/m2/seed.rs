use from_file::FromFile;
use presets::m2::module_meta_data::ModuleData;
use presets::m2::requirejs_config::RequireJsClientConfig;

#[derive(Serialize, Deserialize, Default)]
pub struct SeedData {
    pub rjs_client_config: RequireJsClientConfig,
    pub req_log: Vec<ModuleData>,
}

impl FromFile for SeedData {}
