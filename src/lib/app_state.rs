use config::ProgramConfig;
use options::ProgramOptions;
use preset::RewriteFns;
use presets::m2::module_meta_data::ModuleData;
use std::fmt;
use std::sync::{Arc, Mutex};
use rjs::RequireJsClientConfig;

pub struct AppState {
    pub program_config: ProgramConfig,
    pub opts: ProgramOptions,
    pub rewrites: RewriteFns,
    pub req_log: Mutex<Vec<ModuleData>>,
    pub rjs_client_config: Arc<Mutex<RequireJsClientConfig>>,
}

impl AppState {
    pub fn new(host: impl Into<String>, scheme: impl Into<String>) -> AppState {
        AppState {
            opts: ProgramOptions::new(host, scheme),
            program_config: ProgramConfig::default(),
            rewrites: vec![],
            req_log: Mutex::new(vec![]),
            rjs_client_config: Arc::new(Mutex::new(RequireJsClientConfig::default())),
        }
    }
}

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AppState {{
    program_config: {:?},
    opts: {:?},
    rewrites: {} rewrite fns,
    req_log: Mutex<Vec<ModuleData>>,
    rjs_client_config: Arc<Mutex<RequireJsClientConfig>>
}}
        ",
            self.program_config,
            self.opts,
            self.rewrites.len()
        )
    }
}
