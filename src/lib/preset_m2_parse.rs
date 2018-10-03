use ratel::grammar::*;
use ratel::parser::parse;
use ratel::operator::OperatorKind::*;
use ratel::grammar::Statement::{VariableDeclaration};
use ratel::grammar::Expression;


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
            }
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
                    println!("d={:?}", d.name);
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
