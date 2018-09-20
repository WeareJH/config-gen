use serde_yaml::Value;

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct M2PresetOptions {
    pub require_path: String,
    pub bundle_config: Option<String>,
    pub auth_basic: Option<AuthBasic>,
}

impl From<Value> for M2PresetOptions {
    fn from(v: Value) -> Self {
        let out = M2PresetOptions {
            ..Default::default()
        };
        out.parse_preset_options(v)
    }
}

impl Default for M2PresetOptions {
    fn default() -> Self {
        M2PresetOptions {
            require_path:
                "/static/{version}/frontend/{vendor}/{theme}/{locale}/requirejs/require.js".into(),
            bundle_config: None,
            auth_basic: None,
        }
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct AuthBasic {
    username: String,
    password: String,
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
    pub fn parse_preset_options(mut self, value: Value) -> M2PresetOptions {
        if let Value::Mapping(m) = value {
            for (key, value) in m {
                if let (Value::String(ref key), Value::String(ref value)) = (key, value) {
                    match key.as_str() {
                        "require_path" => self.require_path = value.clone(),
                        "bundle_config" => {
                            self.bundle_config = Some(value.clone());
                        }
                        _ => { /* not supported */ }
                    }
                }
            }
        }
        self
    }
}

#[test]
fn test_parse_preset_options() {
    let i = r#"
require_path: /js/require.js
bundle_config: file:test/fixtures/bundle-config.yaml
auth_basic:
    username: shane
    password: other
    "#;
    let m = M2PresetOptions::default();
    let y = serde_yaml::from_str(&i).unwrap();
    let out = m.parse_preset_options(y);
    println!("{:#?}", out);
}
