use config::ProgramConfig;

#[derive(Deserialize, Debug)]
pub struct M2PresetOptions {
    #[serde(default = "default_require_path")]
    pub require_path: Option<String>,
    pub bundle_config: Option<String>,
    pub auth_basic: Option<AuthBasic>,
    pub module_blacklist: Option<Vec<String>>,
}

fn default_require_path() -> Option<String> {
    Some("/static/{version}/frontend/{vendor}/{theme}/{locale}/requirejs/require.js".into())
}

impl Default for M2PresetOptions {
    fn default() -> Self {
        M2PresetOptions {
            require_path: None,
            bundle_config: None,
            auth_basic: None,
            module_blacklist: None,
        }
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct AuthBasic {
    pub username: String,
    pub password: String,
}

impl Default for AuthBasic {
    fn default() -> Self {
        AuthBasic {
            username: String::new(),
            password: String::new(),
        }
    }
}

impl M2PresetOptions {
    pub fn get_opts(prog_config: &ProgramConfig) -> Option<M2PresetOptions> {
        serde_yaml::from_value(prog_config.get_opts("m2")?).ok()?
    }
}

#[test]
fn test_parse_preset_options_all_given() {
    let i = r#"
require_path: /js/require.js
bundle_config: file:test/fixtures/bundle-config.yaml
auth_basic:
    username: shane
    password: other
    "#;
    let y: M2PresetOptions = serde_yaml::from_str(&i).unwrap();
    assert_eq!(y.require_path, Some("/js/require.js".to_string()));
}

#[test]
fn test_defaults() {
    let i = r#"
    bundle_config: "here"
    "#;
    let y: M2PresetOptions = serde_yaml::from_str(&i).unwrap();
    assert_eq!(y.bundle_config, Some("here".to_string()));
}
