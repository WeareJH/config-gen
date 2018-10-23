use actix_web::App;
use app_state::AppState;
use config::ProgramConfig;
use from_file::FromFile;
use options::ProgramOptions;
use preset::Preset;
use presets::m2::opts::M2PresetOptions;
use presets::m2::preset_m2::M2Preset;
use presets::m2::requirejs_config::RequireJsClientConfig;
use presets::m2::seed::SeedData;
use proxy_transform::proxy_transform;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub type PresetsMap = HashMap<usize, Box<Preset<AppState>>>;

pub fn apply_presets(
    mut app: App<AppState>,
    program_config: &ProgramConfig,
    presets_map: &PresetsMap,
) -> App<AppState> {
    // before middlewares
    for (index, _) in program_config.presets.iter().enumerate() {
        let subject_preset = presets_map.get(&index).expect("Missing preset");
        app = subject_preset.add_before_middleware(app);
    }

    // enhances
    for (index, _) in program_config.presets.iter().enumerate() {
        let subject_preset = presets_map.get(&index).expect("Missing preset");
        app = subject_preset.enhance(app);
    }

    app.default_resource(|r| r.f(proxy_transform))
}

pub fn state_and_presets(
    opts: &ProgramOptions,
    program_config: &ProgramConfig,
    maybe_seed: &Option<String>,
) -> (AppState, PresetsMap) {
    //
    // Use a HashMap + index lookup for anything
    // that implements Preset
    //
    let mut presets_map: PresetsMap = HashMap::new();

    //
    // Loop through any presets and create an instance
    // that's stored in the hashmap based on it's index
    //
    // This is done so that we can use the index later
    // to lookup this item in order
    //
    for (index, p) in program_config.presets.iter().enumerate() {
        match p.name.as_str() {
            "m2" => {
                let preset_opts: M2PresetOptions =
                    serde_json::from_value(p.options.clone()).unwrap();
                let preset = M2Preset::new(preset_opts);
                presets_map.insert(index, Box::new(preset));
            }
            _ => println!("unsupported"),
        }
    }

    let mut app_state = create_state(maybe_seed.clone(), program_config.clone(), opts.clone());

    // Add rewrites phase
    for (index, _) in program_config.presets.iter().enumerate() {
        let subject_preset = presets_map.get(&index).expect("Missing preset");
        app_state.rewrites.extend(subject_preset.rewrites());
    }

    (app_state, presets_map)
}

///
/// Build up the application state based on a potential
/// incoming seed
///
pub fn create_state(
    maybe_seed: Option<String>,
    program_config: ProgramConfig,
    opts: ProgramOptions,
) -> AppState {
    let (req_log, rjs_client_config) = match maybe_seed {
        Some(ref s) => match SeedData::from_file(&s) {
            Ok(seed) => (seed.req_log, seed.rjs_client_config),
            Err(e) => {
                eprintln!("Could not read seed, {:?}", e);
                (vec![], RequireJsClientConfig::default())
            }
        },
        None => (vec![], RequireJsClientConfig::default()),
    };

    AppState {
        program_config,
        opts,
        rewrites: vec![],
        req_log: Mutex::new(req_log),
        rjs_client_config: Arc::new(Mutex::new(rjs_client_config)),
    }
}
