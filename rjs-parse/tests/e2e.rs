extern crate rjs;
extern crate from_file;
extern crate serde_json;

use rjs::{RequireJsBuildConfig, RequireJsClientConfig, bundle_config};

#[test]
fn test_all_strings() {
    let rjs_config = r#"
    (function() {
       var config = {
         shim: {
           backbone: ["jquery"]
         }
       }
    })();
    "#;
    let bundle_config = r#"
        {"bundles": [{"name": "main", "children": [], "urls": ["/"]}]}
    "#;
    let req_log = r#"
        [
            {
                "url": "https://example.com/jquery",
                "id": "jquery",
                "referrer": "/"
            }
        ]
    "#;
    let expected = r#"
    {
      "generateSourceMaps": true,
      "inlineText": true,
      "optimize": "uglify",
      "deps": [],
      "map": {},
      "config": {},
      "shim": {
        "backbone": [
          "jquery"
        ]
      },
      "paths": {},
      "modules": [
        {
          "name": "requirejs/require",
          "include": [],
          "exclude": [],
          "create": false
        },
        {
          "name": "main",
          "include": [
            "jquery"
          ],
          "exclude": [
            "requirejs/require"
          ],
          "create": true
        }
      ]
    }
    "#;

    let rjs_build = RequireJsBuildConfig::from_generated_string(rjs_config).expect("sup");
    let config = bundle_config::BundleConfig::from_json_string(bundle_config).expect("bundle config parse");
    let req_log: Vec<rjs::modules::ModuleData> = serde_json::from_str(req_log).expect("serde");

    let next_build = rjs_build.create_modules(&config, &req_log);
    let as_string = next_build.to_string().expect("must serialize");

    let actual_as_value: serde_json::Value = serde_json::from_str(&as_string).expect("serde actual");
    let expected_as_value: serde_json::Value = serde_json::from_str(&expected).expect("serde expected");

    assert_eq!(actual_as_value, expected_as_value);
}