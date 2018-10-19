use config::ProgramConfig;
use options::ProgramOptions;
use preset::RewriteFns;
use presets::m2::module_meta_data::ModuleData;
use presets::m2::requirejs_config::RequireJsClientConfig;
use std::fmt;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub program_config: ProgramConfig,
    pub opts: ProgramOptions,
    pub rewrites: RewriteFns,
    pub req_log: Mutex<Vec<ModuleData>>,
    pub rjs_client_config: Arc<Mutex<RequireJsClientConfig>>,
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
