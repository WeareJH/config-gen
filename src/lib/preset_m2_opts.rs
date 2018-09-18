use serde_yaml::Value;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct M2PresetOptions {
    url: String,
    require_path: String,
}

impl From<Value> for M2PresetOptions {
    fn from(v: Value) -> Self {
        let out = M2PresetOptions{..Default::default()};
        out.add_from_value(v)
    }
}

impl M2PresetOptions {
    pub fn add_from_value(mut self, value: Value) -> M2PresetOptions {
        if let Value::Mapping(m) = value {
            for (key, value) in m {
                if let (Value::String(key), Value::String(value)) = (key, value) {
                    match key.as_str() {
                        "url" => self.url = value,
                        "require_path" => self.require_path = value,
                        _ => { /* not supported */ }
                    }
                }
            }
        }
        self
    }
}