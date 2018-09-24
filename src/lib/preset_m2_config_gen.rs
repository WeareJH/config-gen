#![allow(unused_variables)]
#![allow(unused_imports)]

extern crate serde_json;

use preset_m2::ModuleData;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

pub type Items = Vec<ModuleData>;

pub fn collect_items(
    mut target: Vec<Module>,
    conf_items: &Vec<ConfigItem>,
    items: &Items,
    prev: &mut Vec<String>,
    exclude: &mut Vec<String>,
) -> Vec<Module> {
    for conf_item in conf_items {
        let mut outgoing: Vec<String> = vec![];
        for item in items {
            match conf_item.urls.iter().find(|x| **x == item.referrer) {
                Some(t) => {
                    let next_id = create_entry_point(&item);
                    if let None = prev.iter().find(|x| **x == next_id) {
                        outgoing.push(next_id);
                    }
                }
                None => {}
            }
        }
        outgoing.sort();
        outgoing.dedup();
        let module = Module {
            name: conf_item.name.to_string(),
            include: outgoing.clone(),
            exclude: exclude.clone(),
        };
        target.push(module);
        if conf_item.children.len() > 0 {
            prev.extend(outgoing);
            exclude.push(conf_item.name.to_string());
            return collect_items(target, &conf_item.children, items, prev, exclude);
        }
    }
    target
}

pub fn run(items: Items, config: impl Into<ConfigItems>) -> Vec<Module> {
    let h: Vec<Module> = vec![];
    let conf = config.into();
    collect_items(h, &conf.items, &items, &mut vec![], &mut vec![])
}

pub fn to_string(bundles: Vec<Module>) -> String {
    match serde_json::to_string_pretty(&Outgoing { bundles }) {
        Ok(output) => output,
        Err(err) => {
            println!("Could not create bundles, {:?}", err);
            r#""Could not create bundles""#.into()
        }
    }
}

#[test]
fn test_create_entry_point() {
    let item = ModuleData {
        url: "one/two/three.html".into(),
        id: "one/two/three".into(),
        referrer: String::new(),
    };
    assert_eq!(create_entry_point(&item), "text!one/two/three.html");
}

fn create_entry_point(item: &ModuleData) -> String {
    match PathBuf::from(&item.url).extension() {
        Some(t) => match t.to_str() {
            Some("html") => format!("text!{}.html", item.id),
            _ => item.id.to_string(),
        },
        None => item.id.to_string(),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigItem {
    name: String,
    urls: Vec<String>,
    children: Vec<ConfigItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigItems {
    pub items: Vec<ConfigItem>,
}

impl<'a> Into<ConfigItems> for &'a str {
    fn into(self) -> ConfigItems {
        let items: Vec<ConfigItem> = match serde_yaml::from_str(&self) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("{}", e);
                vec![]
            }
        };
        ConfigItems { items }
    }
}

#[test]
fn test_parse_config() {
    let c: ConfigItems = r#"
    [
      {
        "name": "requirejs/require",
        "urls": [
          "/",
          "/nav/home-fragrance.html"
        ],
        "children": []
      }
    ]
    "#.into();

    assert_eq!("requirejs/require", c.items[0].name);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Module {
    name: String,
    include: Vec<String>,
    exclude: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Outgoing {
    bundles: Vec<Module>,
}
