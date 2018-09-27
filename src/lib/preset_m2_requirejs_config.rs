extern crate serde_json;

use serde_json::{Value, Error};
use std::collections::HashMap;
use preset_m2_config_gen::Module;
use url::Url;

type ModuleId = String;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct RequireJsMergedConfig {

    pub dir: Option<String>,

    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,

    #[serde(rename = "generateSourceMaps")]
    pub generate_source_maps: Option<bool>,

    #[serde(default = "default_inline_text")]
    pub inline_text: Option<bool>,

    #[serde(default = "default_optimize")]
    pub optimize: Option<String>,

    pub deps: Vec<ModuleId>,
    pub map: serde_json::Value,
    pub config: serde_json::Value,
    pub shim: serde_json::Value,
    pub paths: HashMap<String, String>,
    pub modules: Option<Vec<Module>>,
}

impl RequireJsMergedConfig {
    pub fn mixins(&self) -> Vec<String> {
        match self.config {
            serde_json::Value::Object(ref v) => {
                match v.get("mixins") {
                    Some(f) => {
                        match f {
                            serde_json::Value::Object(ref v) => {
                                let names: Vec<String> = v.iter().map(|(key, value)| {
                                    key.to_string()
                                }).collect();
                                names
                            }
                            _ => vec![]
                        }
                    }
                    None => vec![]
                }
            }
            _ => vec![]
        }
    }
}

fn default_optimize() -> Option<String> { Some("none".to_string()) }
fn default_inline_text() -> Option<bool> { Some(true) }

#[test]
fn test_parse_incoming_from_browser() {
    let input = include_bytes!("../../test/fixtures/example-post.json");
    let s: RequireJsMergedConfig = serde_json::from_slice(input).unwrap();
    assert_eq!(s.deps, vec![
        "Magento_Theme/js/responsive",
        "Magento_Theme/js/theme"
    ]);
    assert_eq!(s.base_url, Some("/static/version1517228438/frontend/Magento/luma/en_US/".to_string()));
    assert_eq!(s.paths.get("jquery/ui"), Some(&"jquery/jquery-ui".to_string()));
}

#[test]
fn test_filter_mixins() {
    let input = include_bytes!("../../test/fixtures/example-post.json");
    let s: RequireJsMergedConfig = serde_json::from_slice(input).unwrap();
    assert_eq!(s.mixins(), vec!["jquery/jstree/jquery.jstree"]);

    let s2 = RequireJsMergedConfig::default();
    let expected: Vec<String> = vec![];

    assert_eq!(s2.mixins(), expected);
}

#[derive(Debug)]
pub struct BaseDirs {
    pub dir: String,
    pub base_url: String
}

pub fn base_to_dirs(input: &str) -> Result<BaseDirs, String> {
    match Url::parse(input) {
        Ok(mut url) => {
            url.path_segments_mut().map_err(|_| "cannot be base").expect("url").pop_if_empty();
            let mut segs = url.path_segments().map(|c| c.collect::<Vec<_>>());
            let mut last = segs.clone().unwrap().pop().expect("can take last").to_string();
            let last_for_dir = last.clone();

            let mut base_output = vec!["static"];
            let mut dir_output = vec!["static"];

            for (i, item) in segs.expect("can iter over segs").iter().enumerate().skip(2) {
                if *item != last.as_str() {
                    base_output.push(item);
                    dir_output.push(item);
                }
            }

            dir_output.push(&last_for_dir);
            last.push_str("_src");
            base_output.push(&last);

            Ok(BaseDirs{ dir: dir_output.join("/"), base_url: base_output.join("/") })
        },
        Err(err) => {
            Err(err.to_string())
        }
    }
}

#[test]
fn test_base_to_dirs() {
    let bd = base_to_dirs("https://127.0.0.1:8080/static/version1538053013/frontend/Graham/default/en_GB/");
    println!("{:?}", bd)
}
