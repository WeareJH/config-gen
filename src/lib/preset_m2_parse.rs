use ratel::grammar::*;
use ratel::parser::parse;
use ratel::grammar::Expression;
use ratel::grammar::{Value, ObjectMember, ObjectKey};

#[test]
fn test_get_deps() {
    let input = r#"
    (function() {
        /**
         * Copyright © Magento, Inc. All rights reserved.
         * See COPYING.txt for license details.
         */

        var config = {
            map: {
                '*': {
                    checkoutBalance:    'Magento_Customer/js/checkout-balance',
                    address:            'Magento_Customer/address',
                    changeEmailPassword: 'Magento_Customer/change-email-password',
                    passwordStrengthIndicator: 'Magento_Customer/js/password-strength-indicator',
                    zxcvbn: 'Magento_Customer/js/zxcvbn',
                    addressValidation: 'Magento_Customer/js/addressValidation'
                }
            },
            deps: ["first", "second"]
        };

        require.config(config);
    })();
    (function() {
        var config = {
            deps: ["third", "forth"]
        };
        require.config(config);
    })();
    (function() {
        var config = {
            'deps': [
                'jquery/jquery-single'
            ],
        };
        require.config(config);
    })();
    (function() {
        var config = {
            "deps": [
                'jquery/jquery-double'
            ],
        };
        require.config(config);
    })();
    "#;


    let out = parse(input.into()).unwrap();
    let mut deps: Vec<String> = vec![];
    parse_body(out.body, &mut deps);
//    println!("deps={:?}", deps);
    assert_eq!(deps, vec![
        "first".to_string(),
        "second".to_string(),
        "third".to_string(),
        "forth".to_string(),
        "jquery/jquery-single".to_string(),
        "jquery/jquery-double".to_string(),
    ]);
}

#[test]
fn test_from_large_file() {
    let input = include_str!("../../test/fixtures/requirejs-config-generated.js");

    let out = parse(input.into()).unwrap();
    let mut deps: Vec<String> = vec![];
    parse_body(out.body, &mut deps);

    assert_eq!(deps, vec![
        "jquery/jquery.mobile.custom",
        "mage/common",
        "mage/dataPost",
        "mage/bootstrap",
        "jquery/jquery-migrate",
        "mage/translate-inline",
        "Magento_Theme/js/responsive",
        "Magento_Theme/js/theme"
    ]);
}

fn parse_body(items: Vec<Statement>, deps: &mut Vec<String>) {
    for statement in items.iter() {
        match statement {
            Statement::VariableDeclaration { declarators, ..} => {
                for d in declarators.iter().filter(|d| d.name.as_str() == "config") {
                    match d.value {
                        Some(Expression::Object(ref xs)) => {
                            if let Some(Expression::Array(vs)) = get_object_value(&xs, "deps") {
                                for v in vs {
                                    match v {
                                        Expression::Literal(Value::String(s)) => {
                                            let len = s.len();
                                            let next_s = &s[1..len-1];
                                            deps.push(next_s.to_string());
                                        }
                                        _ => { /* */ }
                                    }
                                }
                            }
                        },
                        Some(_) => { /* */ },
                        None => { /* */ }
                    }
                }
            },
            Statement::Expression { value } => {
                match value {
                    Expression::Call { callee, .. } => {
                        let fun = (**callee).clone();
                        match fun {
                            Expression::Function { body, .. } => {
                                parse_body(body, deps);
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

fn get_object_value(xs: &Vec<ObjectMember>, name: &str) -> Option<Expression> {
    xs.iter().find(|x| filter_deps(*x, name))
        .and_then(|x| {
            match x {
                ObjectMember::Value { value, .. } => {
                    Some(value.clone())
                }
                _ => None
            }
        })
        .or(None)
}

fn filter_deps(x: &ObjectMember, name: &str) -> bool {
    match x {
        ObjectMember::Value { key, .. } => {
            match key {
                ObjectKey::Literal(s) => {
                    let as_str = s.as_str();
                    let len = as_str.len();
                    let stripped = &as_str[1..len-1];
                    let possible = vec![as_str, stripped];
                    possible.contains(&name)
                }
                _ => false
            }
        }
        _ => false
    }
}