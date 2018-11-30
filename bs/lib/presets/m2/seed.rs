use from_file::FromFile;
use rjs::modules::ModuleData;
use rjs::RequireJsClientConfig;

#[derive(Serialize, Deserialize, FromFile, Default)]
pub struct SeedData {
    pub rjs_client_config: RequireJsClientConfig,
    pub req_log: Vec<ModuleData>,
}
