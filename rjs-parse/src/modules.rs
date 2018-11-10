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

#[derive(Debug)]
pub enum ModuleDataError {
    SerdeError(serde_json::Error)
}

impl ModuleDataError {
    pub fn to_string(&self) -> String {
        match self {
            ModuleDataError::SerdeError(e) => format!("{}", e.to_string()),
        }
    }
}

impl ModuleData {
    ///
    pub fn from_json_string(input: impl Into<String>) -> Result<Vec<ModuleData>, ModuleDataError> {
        serde_json::from_str(&input.into()).map_err(|e| ModuleDataError::SerdeError(e))
    }
}

///
/// generate just the 'modules' part of the RequireJS Optimizer configuration.
/// It requires just 2 things - the request log & the bundle_config provided by the user
///
pub fn generate_modules(
    req_log: &Vec<ModuleData>,
    config: &BundleConfig,
) -> Vec<BuildModule> {
    let mut modules: Vec<BuildModule> = vec![BuildModule {
        name: "requirejs/require".into(),
        include: vec![],
        exclude: vec![],
        create: false,
    }];
    collect(
        &mut modules,
        req_log,
        &config.bundles,
        &mut vec![],
        vec!["requirejs/require".into()],
    );
    modules.to_vec()
}

///
/// Flatten the nested bundle_config
///
pub fn collect<'a>(
    modules: &'a mut Vec<BuildModule>,
    req_log: &Vec<ModuleData>,
    children: &Vec<ConfigItem>,
    prev: &mut Vec<String>,
    exclude: Vec<String>,
) {
    children.iter().for_each(|conf_item| {
        let mut include: Vec<String> = vec![];
        req_log.iter().for_each(|item| {
            if let Some(..) = conf_item.urls.iter().find(|x| **x == item.referrer) {
                let next_id = create_entry_point(&item);
                if let None = prev.iter().find(|x| **x == next_id) {
                    include.push(next_id);
                }
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
        prev.extend(include);
        collect(modules, req_log, &conf_item.children, prev, exclude);
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
        serde_json::from_str(include_str!("../test/fixtures/example-reqs.json")).unwrap();
    let out = generate_modules(&reqs, &c);

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
