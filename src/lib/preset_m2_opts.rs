use serde_yaml::Value;

#[derive(Debug, Eq, PartialEq)]
pub struct M2PresetOptions {
    require_path: String,
    bundle_config: Option<String>,
}

impl From<Value> for M2PresetOptions {
    fn from(v: Value) -> Self {
        let out = M2PresetOptions {
            ..Default::default()
        };
        out.add_from_value(v)
    }
}

impl Default for M2PresetOptions {
    fn default() -> Self {
        M2PresetOptions {
            require_path:
                "/static/{version}/frontend/{vendor}/{theme}/{locale}/requirejs/require.js".into(),
            bundle_config: None,
        }
    }
}

impl M2PresetOptions {
    pub fn add_from_value(mut self, value: Value) -> M2PresetOptions {
        if let Value::Mapping(m) = value {
            for (key, value) in m {
                if let (Value::String(key), Value::String(value)) = (key, value) {
                    match key.as_str() {
                        "require_path" => self.require_path = value,
                        "bundle_config" => {
                            self.bundle_config = Some(value);
                        }
                        _ => { /* not supported */ }
                    }
                }
            }
        }
        self
    }
}
