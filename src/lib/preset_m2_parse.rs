use ratel::grammar::*;
use ratel::parser::parse;
use ratel::operator::OperatorKind::*;
use ratel::grammar::Statement::{VariableDeclaration};
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
    "#;


    let out = parse(input.into()).unwrap();
    parse_body(out.body);
}

fn parse_body(items: Vec<Statement>) {
    for statement in items.iter() {
        match statement {
            Statement::VariableDeclaration {kind, declarators} => {
                for d in declarators.iter().filter(|d| d.name.as_str() == "config") {
                    match d.value {
                        Some(ref d) => match d {
                            Expression::Object(xs) => {
                                if let Some(v) = get_object_value(xs, "deps") {
                                    match v {
                                        Expression::Array(vs) => {
                                            for v in vs {
                                                match v {
                                                    Expression::Literal(Value::String(s)) => {
                                                        println!("s={}", s);
                                                    }
                                                    _ => println!("no")
                                                }
                                            }
                                        }
                                        _ => println!("no")
                                    }
                                }
                            },
                            _ => println!("no")
                        },
                        None => println!("no")
                    }
                }
            },
            Statement::Expression { value } => {
                match value {
                    Expression::Call { callee, arguments } => {
                        let fun = (**callee).clone();
                        match fun {
                            Expression::Function { name, params, body } => {
                                parse_body(body);
                            }
                            _ => println!("none-expression")
                        }
                    }
                    _ => println!("none-expression")
                }
            }
            _ => println!("none-variable")
        }
    }
}

fn get_object_value(xs: &Vec<ObjectMember>, name: &str) -> Option<Expression> {
    xs.iter().find(|x| filter_deps(*x, "deps"))
        .and_then(|x| {
            match x {
                ObjectMember::Value { key, value } => {
                    Some(value.clone())
                }
                _ => None
            }
        })
        .or(None)
}

fn filter_deps(x: &ObjectMember, name: &str) -> bool {

    match x {
        ObjectMember::Value { key, value } => {
            match key {
                ObjectKey::Literal(s) => {
                    s.as_str() == name
                }
                _ => false
            }
        }
        _ => false
    }
}