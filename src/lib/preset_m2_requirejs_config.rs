use preset_m2_config_gen::Module;
use std::collections::HashMap;
use url::Url;
use from_file::FromFile;
use preset_m2::ModuleData;

type ModuleId = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RequireJsClientConfig {
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
    pub deps: Vec<ModuleId>,
    pub map: serde_json::Value,
    pub config: serde_json::Value,
    pub shim: serde_json::Value,
    pub paths: HashMap<String, String>,
}

impl Default for RequireJsClientConfig {
    fn default() -> RequireJsClientConfig {
        RequireJsClientConfig {
            base_url: Some("".into()),
            deps: vec![],
            map: json!({}),
            config: json!({}),
            shim: json!({}),
            paths: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RequireJsBuildConfig {

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



    #[serde(default = "default_modules")]
    pub modules: Option<Vec<Module>>,
}

impl RequireJsBuildConfig {
    pub fn strip_paths(paths: &HashMap<String, String>) -> HashMap<String, String> {
        let mut hm: HashMap<String, String> = HashMap::new();

        for (key, value) in paths.iter() {
            if value.starts_with("http://") ||
                value.starts_with("https://") ||
                value.starts_with("//") {
                hm.insert(key.clone(), "empty:".to_string());
            } else {
                hm.insert(key.clone(), value.clone());
            }
        }

        hm
    }
    pub fn fix_shims(shim: &mut serde_json::Value) -> &mut serde_json::Value {
        match shim {
            serde_json::Value::Object(m) => {
                for (_, value) in m.into_iter().filter(|(key, _)| key.as_str() == "jquery/jquery-migrate") {

                    *value.pointer_mut("/deps").unwrap() =
                        serde_json::Value::Array(vec![
                            serde_json::Value::String("jquery".to_string()),
                            serde_json::Value::String("jquery/jquery.cookie".to_string())
                        ]);
                }
            },
            _ => println!("no")
        };

        shim
    }
    pub fn drop_blacklisted(modules: &Vec<ModuleData>, blacklist: &Vec<String>) -> Vec<ModuleData> {
        let mut output = vec![];

        for m in modules.iter() {
            if !blacklist.contains(&m.id) {
                output.push(m.clone());
            }
        }

        output
    }
}

#[test]
fn test_strip_paths() {
    let mut ps: HashMap<String, String> = HashMap::new();
    ps.insert("one".into(), "one/one".into());
    ps.insert("two".into(), "http://two.com/two".into());

    let mut expected: HashMap<String, String> = HashMap::new();
    expected.insert("one".into(), "one/one".into());
    expected.insert("two".into(), "empty:".into());

    let actual = RequireJsBuildConfig::strip_paths(&ps);

    assert_eq!(actual, expected);
}

impl Default for RequireJsBuildConfig {
    fn default() -> RequireJsBuildConfig {
        RequireJsBuildConfig {
            deps: vec![],
            map: json!({}),
            config: json!({}),
            shim: json!({}),
            paths: HashMap::new(),
            generate_source_maps: Some(true),
            inline_text: Some(true),
            optimize: Some("none".into()),
            modules: Some(vec![])
        }
    }
}

impl FromFile for RequireJsClientConfig {}

impl RequireJsClientConfig {
    pub fn mixins(val: &serde_json::Value) -> Vec<String> {
        match *val {
            serde_json::Value::Object(ref v) => match v.get("mixins") {
                Some(f) => match f {
                    serde_json::Value::Object(ref v) => {
                        let names: Vec<String> =
                            v.iter().map(|(key, _)| key.to_string()).collect();
                        names
                    }
                    _ => vec![],
                },
                None => vec![],
            },
            _ => vec![],
        }
    }
    pub fn module_list(mixins: Vec<String>, modules: Vec<Module>) -> String {
        let items: Vec<String> = modules
            .iter()
            .map(|module| {
                let module_list: Vec<String> = module
                    .include
                    .iter()
                    .map(|name| {
                        let is_mixin_trigger = mixins.contains(&name);
                        match is_mixin_trigger {
                            true => format!("         // mixin trigger: \"{}\",", name),
                            false => format!("        \"{}\",", name),
                        }
                    })
                    .collect();

                format!(
                    "require.config({{\n  bundles: {{\n    \"{}\": [\n{}\n    ]\n  }}\n}});",
                    module.name,
                    module_list.join("\n")
                )
            })
            .collect();
        items.join("\n")
    }
}

fn default_optimize() -> Option<String> {
    Some("none".to_string())
}
fn default_inline_text() -> Option<bool> {
    Some(true)
}
fn default_modules() -> Option<Vec<Module>> {
    Some(vec![])
}

#[test]
fn test_default_require_js_config() {
    let r = RequireJsClientConfig::default();
    let as_string = serde_json::to_string_pretty(&r).unwrap();
    assert_eq!(as_string, r#"{
  "baseUrl": "",
  "deps": [],
  "map": {},
  "config": {},
  "shim": {},
  "paths": {}
}"#);
}

#[test]
fn test_parse_incoming_from_browser() {
    let input = include_bytes!("../../test/fixtures/example-post.json");
    let s: RequireJsClientConfig = serde_json::from_slice(input).unwrap();
    assert_eq!(
        s.deps,
        vec!["Magento_Theme/js/responsive", "Magento_Theme/js/theme"]
    );
    assert_eq!(
        s.base_url,
        Some(
            "http://127.0.0.1:9090/static/version1517228438/frontend/Magento/luma/en_US/"
                .to_string()
        )
    );
    assert_eq!(
        s.paths.get("jquery/ui"),
        Some(&"jquery/jquery-ui".to_string())
    );
}

#[test]
fn test_filter_mixins() {
    let input = include_bytes!("../../test/fixtures/example-post.json");
    let s: RequireJsClientConfig = serde_json::from_slice(input).unwrap();
    assert_eq!(
        RequireJsClientConfig::mixins(&s.config),
        vec![
            "Magento_Checkout/js/action/place-order",
            "Magento_Checkout/js/action/set-payment-information",
            "jquery/jstree/jquery.jstree",
        ]
    );
}

#[test]
fn test_require_build_shim() {
    let input = r#"
    {
        "shim": {
            "jquery/jquery-migrate": {
                "deps": [
                  "jquery"
                ]
            }
        },
        "deps": [],
        "map": {},
        "config": {},
        "paths": {}
    }
    "#;
    let expected = r#"
    {
        "shim": {
            "jquery/jquery-migrate": {
                "deps": [
                  "jquery",
                  "jquery/jquery.cookie"
                ]
            }
        },
        "deps": [],
        "map": {},
        "config": {},
        "paths": {}
    }
    "#;
    let mut s: RequireJsClientConfig = serde_json::from_str(input).unwrap();
    let expected: RequireJsClientConfig = serde_json::from_str(expected).unwrap();
    {
        RequireJsBuildConfig::fix_shims(&mut s.shim);
    }
    assert_eq!(s, expected);
}

#[test]
fn test_hydrate() {
    let input = include_bytes!("../../test/fixtures/example-config.json");
    let _s: RequireJsClientConfig = serde_json::from_slice(input).unwrap();
}

#[test]
fn test_module_list() {
    let list = RequireJsClientConfig::module_list(
        vec!["js/shane".to_string()],
        vec![
            Module {
                name: String::from("bundle/base"),
                include: vec!["js/shane".to_string(), "js/kittie".to_string()],
                exclude: vec![],
                create: true,
            },
            Module {
                name: String::from("bundle/product"),
                include: vec!["js/gallery".to_string(), "js/zoomer".to_string()],
                exclude: vec![],
                create: true,
            },
        ],
    );
    let expected = r#"require.config({
  bundles: {
    "bundle/base": [
         // mixin trigger: "js/shane",
        "js/kittie",
    ]
  }
});
require.config({
  bundles: {
    "bundle/product": [
        "js/gallery",
        "js/zoomer",
    ]
  }
});"#;
    //    println!("{}", list);
    assert_eq!(list, expected);
}

#[derive(Debug)]
pub struct BaseDirs {
    pub dir: String,
    pub base_url: String,
}

pub fn base_to_dirs(input: &str) -> Result<BaseDirs, String> {
    match Url::parse(input) {
        Ok(mut url) => {
            url.path_segments_mut()
                .map_err(|_| "cannot be base")
                .expect("url")
                .pop_if_empty();
            let mut segs = url.path_segments().map(|c| c.collect::<Vec<_>>());
            let mut last = segs
                .clone()
                .unwrap()
                .pop()
                .expect("can take last")
                .to_string();
            let last_for_dir = last.clone();

            let mut base_output = vec!["static"];
            let mut dir_output = vec!["static"];

            for (_, item) in segs.expect("can iter over segs").iter().enumerate().skip(2) {
                if *item != last.as_str() {
                    base_output.push(item);
                    dir_output.push(item);
                }
            }

            dir_output.push(&last_for_dir);
            last.push_str("_src");
            base_output.push(&last);

            Ok(BaseDirs {
                dir: dir_output.join("/"),
                base_url: base_output.join("/"),
            })
        }
        Err(err) => Err(err.to_string()),
    }
}

#[test]
fn test_base_to_dirs() {
    let bd = base_to_dirs(
        "https://127.0.0.1:8080/static/version1538053013/frontend/Graham/default/en_GB/",
    );
    println!("{:?}", bd)
}
