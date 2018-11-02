#![allow(unused_variables)]
#![allow(unused_imports)]

extern crate serde_json;

use from_file::FromFile;
use presets::m2::bundle_config::{BundleConfig, ConfigItem};
use presets::m2::module_meta_data::ModuleData;
use rjs::BuildModule;
use serde_json::Error;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

pub type Items = Vec<ModuleData>;

pub fn collect(
    prev: &mut Vec<BuildModule>,
    items: &Vec<ModuleData>,
    children: &Vec<ConfigItem>,
    exclude: &mut Vec<String>,
) -> Vec<BuildModule> {
    children.into_iter().fold(
        prev.to_vec(),
        |mut acc: Vec<BuildModule>, conf_item: &ConfigItem| {
            let mut include: Vec<String> = vec![];
            for item in items {
                if let Some(t) = conf_item.urls.iter().find(|x| **x == item.referrer) {
                    include.push(create_entry_point(&item));
                }
            }
            include.sort();
            include.dedup();
            let this_item = BuildModule {
                name: conf_item.name.to_string(),
                include: include.to_vec(),
                exclude: exclude.to_vec(),
                create: true,
            };
            acc.push(this_item);
            let mut exclude = exclude.clone();
            exclude.push(conf_item.name.to_string());
            collect(&mut acc, items, &conf_item.children, &mut exclude)
        },
    )
}

pub fn generate_modules(items: Items, config: impl Into<BundleConfig>) -> Vec<BuildModule> {
    let mut initial: Vec<BuildModule> = vec![BuildModule {
        name: "requirejs/require".into(),
        include: vec![],
        exclude: vec![],
        create: false,
    }];
    let conf = config.into();
    collect(
        &mut initial,
        &items,
        &conf.bundles,
        &mut vec!["requirejs/require".into()],
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
              },
              {
                "name": "bundles/basket-other",
                "urls": [
                  "/index.php/checkout/cart-other/"
                ],
                "children": []
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
        BuildModule {
            include: vec![],
            exclude: vec![],
            name: "requirejs/require".to_string(),
            create: false,
        }
    );

    assert_eq!(out[1].create, true);
    let out_names: Vec<String> = out.iter().map(|item| item.name.to_string()).collect();

    assert_eq!(
        out_names,
        vec![
            "requirejs/require",
            "bundles/main",
            "bundles/basket",
            "bundles/checkout",
            "bundles/checkout-success",
            "bundles/basket-other",
        ].iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    );

    assert_eq!(out[5].exclude, vec!["requirejs/require", "bundles/main"]);
}
