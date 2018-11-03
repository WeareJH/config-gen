#![allow(unused_variables)]
#![allow(unused_imports)]

extern crate serde_json;

use bundle_config::{BundleConfig, ConfigItem};
use serde_json::Error;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

///
/// This struct is used for each 'module' as defined in the RequireJS documentation
///
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BuildModule {
    pub name: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub create: bool,
}

// Just a type alias for clarity
pub type BuildModuleId = String;

///
/// This is the data type that is comes from each request
/// in a query param
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
pub struct ModuleData {
    pub url: String,
    pub id: String,
    pub referrer: String,
}

///
/// generate just the 'modules' part of the RequireJS Optimizer configuration.
/// It requires just 2 things - the request log & the bundle_config provided by the user
///
pub fn generate_modules(
    items: Vec<ModuleData>,
    config: impl Into<BundleConfig>,
) -> Vec<BuildModule> {
    let mut modules: Vec<BuildModule> = vec![BuildModule {
        name: "requirejs/require".into(),
        include: vec![],
        exclude: vec![],
        create: false,
    }];
    let conf = config.into();
    collect(
        &mut modules,
        &items,
        &conf.bundles,
        vec!["requirejs/require".into()],
    );
    modules.to_vec()
}

///
/// Flatten the nested bundle_config
///
pub fn collect<'a>(
    modules: &'a mut Vec<BuildModule>,
    items: &Vec<ModuleData>,
    children: &Vec<ConfigItem>,
    exclude: Vec<String>,
) {
    children.iter().for_each(|conf_item| {
        let mut include: Vec<String> = vec![];
        items.iter().for_each(|item| {
            if let Some(..) = conf_item.urls.iter().find(|x| **x == item.referrer) {
                include.push(create_entry_point(&item));
            }
        });
        include.sort();
        include.dedup();
        let this_item = BuildModule {
            name: conf_item.name.to_string(),
            include: include.to_vec(),
            exclude: exclude.to_vec(),
            create: true,
        };
        modules.push(this_item);
        let mut exclude = exclude.clone();
        exclude.push(conf_item.name.to_string());
        collect(modules, items, &conf_item.children, exclude);
    });
}

///
/// Create an entry point for a module.
///
/// If was a JS file, it's just the ID
/// but if it was a .html file, then prepend text! to enable
/// inlining.
///
/// # Examples
///
/// ```
/// use rjs::modules::*;
/// let item = ModuleData {
///     url: "one/two/three.html".into(),
///     id: "one/two/three".into(),
///     referrer: String::new(),
/// };
/// assert_eq!(create_entry_point(&item), "text!one/two/three.html");
/// ```
///
pub fn create_entry_point(item: &ModuleData) -> String {
    match PathBuf::from(&item.url).extension() {
        Some(t) => match t.to_str() {
            Some("html") => format!("text!{}.html", item.id),
            _ => item.id.to_string(),
        },
        None => item.id.to_string(),
    }
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