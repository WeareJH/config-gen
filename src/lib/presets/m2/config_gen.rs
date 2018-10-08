#![allow(unused_variables)]
#![allow(unused_imports)]

extern crate serde_json;

use from_file::FromFile;
use presets::m2::bundle_config::{BundleConfig, ConfigItem, Module};
use presets::m2::preset_m2::ModuleData;
use serde_json::Error;
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
            create: true,
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

pub fn generate_modules(items: Items, config: impl Into<BundleConfig>) -> Vec<Module> {
    let h: Vec<Module> = vec![Module {
        name: "requirejs/require".into(),
        include: vec![],
        exclude: vec![],
        create: false,
    }];
    let conf = config.into();
    collect_items(
        h,
        &conf.bundles,
        &items,
        &mut vec![],
        &mut vec!["requirejs/require".to_string()],
    )
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

#[test]
fn test_parse_config() {
    let c: BundleConfig = r#"
    {
      "bundles": [
          {
            "name": "requirejs/require",
            "urls": [
              "/",
              "/nav/home-fragrance.html"
            ],
            "children": []
          }
      ]
    }
    "#.into();

    assert_eq!("requirejs/require", c.bundles[0].name);
}

#[test]
fn test_create_modules() {
    let c: BundleConfig = r#"
    {
        "module_blacklist": ["mage/bootstrap"],
        "bundles": [
          {
            "name": "bundles/main",
            "urls": [
              "/",
              "/nav/home-fragrance.html"
            ],
            "children": [
              {
                "name": "bundles/basket",
                "urls": [
                  "/index.php/checkout/cart/"
                ],
                "children": [
                  {
                    "name": "bundles/checkout",
                    "urls": [
                      "/index.php/checkout/"
                    ],
                    "children": [
                      {
                        "name": "bundles/checkout-success",
                        "urls": [
                          "/index.php/checkout/onepage/success/"
                        ],
                        "children": []
                      }
                    ]
                  }
                ]
              }
            ]
          }
        ]
    }
    "#.into();
    let reqs: Vec<ModuleData> =
        serde_json::from_str(include_str!("../../../../test/fixtures/example-reqs.json")).unwrap();
    let out = generate_modules(reqs, c);
    assert_eq!(
        out[0],
        Module {
            include: vec![],
            exclude: vec![],
            name: "requirejs/require".to_string(),
            create: false,
        }
    );
    assert_eq!(out[1].create, true);
}
