use ratel;
use ratel::grammar::Expression;
use ratel::grammar::*;
use ratel::grammar::{ObjectKey, ObjectMember, Value};
use ratel::owned_slice::OwnedSlice;
use ratel::parser::parse;
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

///
/// This struct represents the fields that should
/// be extracted from the generated requirejs-config.js file
/// that Magento generates at run time.
///
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct ParsedConfig {
    pub paths: HashMap<String, String>,
    pub map: HashMap<String, HashMap<String, String>>,
    pub deps: Vec<String>,
    pub config: HashMap<String, HashMap<String, HashMap<String, serde_json::Value>>>,
    pub shim: HashMap<String, serde_json::Value>,
}

impl ParsedConfig {
    ///
    /// Parse a Magento 2 generated requirejs-config.js file pulling out
    /// only the pieces of 'config' and ignoring any actual JS
    ///
    /// # Examples
    ///
    /// ```
    /// # use rjs::parse::*;
    /// let input = r#"
    ///   (function() {
    ///        var config = {
    ///            deps: ["one", "two"]
    ///        };
    ///        require.config(config);
    ///    })();
    ///    require(["jquery"], function($) {
    ///        $.noConflict();
    ///    });
    /// "#;
    /// let rjs_cfg = ParsedConfig::from_str(input).expect("should parse");
    /// assert_eq!(rjs_cfg.deps,
    ///     vec!["one".to_string(), "two".to_string()]
    /// );
    /// ```
    ///
    pub fn from_str(input: impl Into<String>) -> Result<ParsedConfig, ConfigParseError> {
        let mut o = ParsedConfig {
            ..ParsedConfig::default()
        };
        let parsed = parse(input.into()).map_err(|e| ConfigParseError::ParseJs(e))?;
        parse_body(parsed.body, &mut o);
        Ok(o)
    }
}

#[derive(Debug)]
pub enum ConfigParseError {
    ParseJs(ratel::error::ParseError),
    Serialize,
    Conversion,
}

impl ConfigParseError {
    pub fn to_string(&self) -> String {
        match self {
            ConfigParseError::ParseJs(e) => format!("{}", e),
            ConfigParseError::Serialize => "OutputError::Serialize".into(),
            ConfigParseError::Conversion => "OutputError::Conversion".into(),
        }
    }
}

impl fmt::Display for ConfigParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

fn parse_body(items: Vec<Statement>, output: &mut ParsedConfig) {
    for statement in items.iter() {
        match statement {
            Statement::VariableDeclaration { declarators, .. } => {
                for d in declarators.iter().filter(|d| d.name.as_str() == "config") {
                    if let Some(Expression::Object(ref xs)) = d.value {
                        process_deps(&xs, output);
                        process_map(&xs, output);
                        process_paths(&xs, output);
                        process_config(&xs, output);
                        process_shim(&xs, output);
                    }
                }
            }
            Statement::Expression { value } => {
                match value {
                    Expression::Call { callee, .. } => {
                        let fun = (**callee).clone();
                        match fun {
                            Expression::Function { body, .. } => {
                                parse_body(body, output);
                            }
                            _ => { /* */ }
                        }
                    }
                    _ => { /* */ }
                }
            }
            _ => { /* */ }
        }
    }
}

fn process_shim(xs: &Vec<ObjectMember>, output: &mut ParsedConfig) {
    match get_object_value(&xs, "shim") {
        Some(Expression::Object(vs)) => {
            for v in vs {
                match v {
                    ObjectMember::Value {
                        key: ObjectKey::Literal(s),
                        value,
                    } => {
                        match value {
                            Expression::Array(vs) => {
                                let as_serde: Vec<serde_json::Value> = vs
                                    .into_iter()
                                    .filter_map(|e: Expression| match e {
                                        Expression::Literal(Value::String(s)) => {
                                            Some(strip_literal(s).to_string())
                                        }
                                        _ => None,
                                    })
                                    .map(|s| serde_json::Value::String(s))
                                    .collect();

                                output
                                    .shim
                                    .insert(strip_literal(s), serde_json::Value::Array(as_serde));
                            }
                            Expression::Object(vs) => {
                                let mut m = serde_json::Map::new();
                                for v in vs {
                                    match v {
                                        ObjectMember::Value {
                                            key: ObjectKey::Literal(k),
                                            value: Expression::Literal(Value::String(v)),
                                        } => {
                                            m.insert(
                                                strip_literal(k),
                                                serde_json::Value::String(
                                                    strip_literal(v).to_string(),
                                                ),
                                            );
                                        }
                                        ObjectMember::Value {
                                            key: ObjectKey::Literal(k),
                                            value: Expression::Array(items),
                                        } => {
                                            let as_serde: Vec<serde_json::Value> = items
                                                .into_iter()
                                                .filter_map(|e: Expression| match e {
                                                    Expression::Literal(Value::String(s)) => {
                                                        Some(strip_literal(s).to_string())
                                                    }
                                                    _ => None,
                                                })
                                                .map(|s| serde_json::Value::String(s))
                                                .collect();
                                            m.insert(
                                                strip_literal(k),
                                                serde_json::Value::Array(as_serde),
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                                output
                                    .shim
                                    .insert(strip_literal(s), serde_json::Value::Object(m));
                            }
                            _ => { /* */ }
                        }
                    }
                    _a => println!("s={:?}", _a),
                }
            }
        }
        _ => { /* */ }
    }
}
fn process_config(xs: &Vec<ObjectMember>, output: &mut ParsedConfig) {
    match get_object_value(&xs, "config") {
        Some(Expression::Object(vs)) => {
            for v in vs {
                match v {
                    ObjectMember::Value {
                        key: ObjectKey::Literal(s),
                        value,
                    } => {
                        let mut map_item = output
                            .config
                            .entry(strip_literal(s).to_string())
                            .or_insert(HashMap::new());

                        match value {
                            Expression::Object(vs) => {
                                for v in vs {
                                    match v {
                                        ObjectMember::Value {
                                            key: ObjectKey::Literal(k),
                                            value: Expression::Object(vs),
                                        } => {
                                            let mut inner_map_item = map_item
                                                .entry(strip_literal(k))
                                                .or_insert(HashMap::new());

                                            for v in vs {
                                                match v {
                                                    ObjectMember::Value {
                                                        key: ObjectKey::Literal(k),
                                                        value,
                                                    } => match value {
                                                        Expression::Literal(Value::True) => {
                                                            inner_map_item.insert(
                                                                strip_literal(k),
                                                                serde_json::Value::Bool(true),
                                                            );
                                                        }
                                                        _ => {
                                                            inner_map_item.insert(
                                                                strip_literal(k),
                                                                json!({}),
                                                            );
                                                        }
                                                    },
                                                    _ => { /* */ }
                                                }
                                            }
                                        }
                                        _ => { /* */ }
                                    }
                                }
                            }
                            _ => { /* */ }
                        };
                    }
                    _ => { /* */ }
                };
            }
        }
        _ => { /* */ }
    };
}
fn process_paths(xs: &Vec<ObjectMember>, output: &mut ParsedConfig) {
    match get_object_value(&xs, "paths") {
        Some(Expression::Object(vs)) => {
            for v in vs {
                match v {
                    ObjectMember::Value {
                        key: ObjectKey::Literal(k),
                        value: Expression::Literal(Value::String(v)),
                    } => {
                        output.paths.insert(strip_literal(k), strip_literal(v));
                    }
                    _ => { /* */ }
                };
            }
        }
        _ => { /* */ }
    };
}

fn process_deps(xs: &Vec<ObjectMember>, output: &mut ParsedConfig) {
    match get_object_value(&xs, "deps") {
        Some(Expression::Array(vs)) => {
            for v in vs {
                match v {
                    Expression::Literal(Value::String(s)) => {
                        let len = s.len();
                        let next_s = &s[1..len - 1];
                        output.deps.push(next_s.to_string());
                    }
                    _ => { /* */ }
                }
            }
        }
        _ => { /* */ }
    };
}

fn process_map(xs: &Vec<ObjectMember>, output: &mut ParsedConfig) {
    match get_object_value(&xs, "map") {
        Some(Expression::Object(vs)) => {
            for v in vs {
                match v {
                    ObjectMember::Value {
                        key: ObjectKey::Literal(s),
                        value,
                    } => {
                        let mut map_item =
                            output.map.entry(strip_literal(s)).or_insert(HashMap::new());

                        match value {
                            Expression::Object(vs) => {
                                for v in vs {
                                    match v {
                                        ObjectMember::Value {
                                            key: ObjectKey::Literal(k),
                                            value: Expression::Literal(Value::String(v)),
                                        } => {
                                            map_item.insert(
                                                strip_literal(k).to_string(),
                                                strip_literal(v),
                                            );
                                        }
                                        _ => { /* */ }
                                    }
                                }
                            }
                            _ => { /* */ }
                        };
                    }
                    _ => { /* */ }
                };
            }
        }
        _ => { /* */ }
    };
}

fn strip_literal(o: OwnedSlice) -> String {
    match &o[0..1] {
        "\"" | "'" => {
            let len = o.len();
            let next_key = &o[1..len - 1];
            next_key.to_string()
        }
        _ => o.to_string(),
    }
}

fn get_object_value(xs: &Vec<ObjectMember>, name: &str) -> Option<Expression> {
    xs.iter()
        .find(|x| filter_items(*x, name))
        .and_then(|x| match x {
            ObjectMember::Value { value, .. } => Some(value.clone()),
            _ => None,
        })
        .or(None)
}

fn filter_items(x: &ObjectMember, name: &str) -> bool {
    match x {
        ObjectMember::Value { key, .. } => match key {
            ObjectKey::Literal(s) => {
                let as_str = s.as_str();
                let len = as_str.len();
                let stripped = &as_str[1..len - 1];
                let possible = vec![as_str, stripped];
                possible.contains(&name)
            }
            _ => false,
        },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use RequireJsClientConfig;

    #[test]
    fn test_js_errors() {
        let o = ParsedConfig::from_str(
            r#"
        var config();
        "#,
        );
        match o {
            Err(e) => {
                println!("{}", e);
            }
            _ => { /**/ }
        }
    }

    #[test]
    fn test_parse_all() {
        let input = r#"
        (function() {
            var config = {
                shim: {
                    "jquery/jquery-migrate": {
                      "deps": [
                        "jquery"
                      ]
                    }
                },
                config: {
                    mixins: {
                        "jquery/jstree/jquery.jstree": {
                            "mage/backend/jstree-mixin": true
                        }
                    }
                },
                paths: {
                    'trackingCode': 'Dotdigitalgroup_Email/js/trackingCode'
                },
                map: {
                    '*': {
                        checkoutBalance:    'Magento_Customer/js/checkout-balance',
                    }
                },
                deps: ["jquery"]
            };
            require.config(config);
        })();
        (function() {
            var config = {
                shim: {
                    paypalInContextExpressCheckout: {
                        exports: 'paypal',
                        deps: ['jquery']
                    },
                    "MutationObserver": ['es6-collections']
                },
                config: {
                    mixins: {
                        "jquery/jstree/jquery.jstree": {
                            "mage/backend/jstree-mixin2": {},
                        },
                        "Magento_Checkout/js/action/place-order": {
                            "Magento_CheckoutAgreements/js/model/place-order-mixin": true
                        }
                    }
                },
                paths: {
                    'trackingCode': 'Dotdigitalgroup_Email/js/trackingCode-alt'
                },
                map: {
                    '*': {
                        'checkoutBalance':    'Magento_Customer/js/checkout-balance-alt',
                        checkoutBalance2:    'Magento_Customer/js/checkout-balance2',
                    },
                    "other-map": {
                        someName: 'some/name'
                    }
                }
            };
            require.config(config);
        })();
        (function() {
            var config = {
                shim: {
                    "jquery/jquery-migrate": {
                      "deps": [
                        "jquery",
                        'jquery/jquery.cookie'
                      ]
                    }
                }
            }
        })();
        "#;

        let o = ParsedConfig::from_str(input).expect("parses fixture");

        let from = r#"
        {
          "config": {
            "mixins": {
              "Magento_Checkout/js/action/place-order": {
                "Magento_CheckoutAgreements/js/model/place-order-mixin": true
              },
              "jquery/jstree/jquery.jstree": {
                "mage/backend/jstree-mixin": true,
                "mage/backend/jstree-mixin2": {}
              }
            }
          },
          "shim": {
            "jquery/jquery-migrate": {
              "deps": [
                "jquery",
                "jquery/jquery.cookie"
              ]
            },
            "paypalInContextExpressCheckout": {
                "exports": "paypal",
                "deps": ["jquery"]
            },
            "MutationObserver": ["es6-collections"]
          },
          "paths": {
            "trackingCode": "Dotdigitalgroup_Email/js/trackingCode-alt"
          },
          "map": {
            "other-map": {
              "someName": "some/name"
            },
            "*": {
              "checkoutBalance2": "Magento_Customer/js/checkout-balance2",
              "checkoutBalance": "Magento_Customer/js/checkout-balance-alt"
            }
          },
          "deps": [
            "jquery"
          ]
        }
        "#;

        let expected: serde_json::Value =
            serde_json::from_str(&from).expect("serde from (fixture)");
        let actual = serde_json::to_value(&o).expect("Output serialized");

        assert_eq!(actual, expected);

        let _as_require: RequireJsClientConfig =
            serde_json::from_value(actual).expect("from value");
    }
}
