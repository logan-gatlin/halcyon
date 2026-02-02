use super::*;
use crate::token::{
    tokenize,
    Base,
};
use crate::Logger;

fn parse_str(input: &str) -> Vec<ParsedModule> {
    let mut logger = Logger::mock();
    let tokens = tokenize(input.chars(), &mut logger);
    let modules = parse(&mut logger, tokens);
    assert!(
        logger.is_ok(),
        "Parser produced errors: {:?}",
        logger.iter().collect::<Vec<_>>()
    );
    modules
}

#[test]
fn test_empty_module() {
    let modules = parse_str("module Test = end");
    assert_eq!(modules.len(), 1);
    let module = &modules[0];
    assert_eq!(module.name.inner, "Test");
    assert_eq!(module.contents.len(), 0);
}

#[test]
fn test_module_with_let_int() {
    let modules = parse_str("module M = let x = 42 end");
    assert_eq!(modules.len(), 1);
    let module = &modules[0];
    assert_eq!(module.contents.len(), 1);

    match &module.contents[0].inner {
        ModuleStatementKind::Let { assignee, value } => {
            match &assignee.inner {
                PatternExpressionKind::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected identifier pattern"),
            }
            match &value.inner {
                ValueExpressionKind::Literal(Literal::Integer(val, base)) => {
                    assert_eq!(val, "42");
                    assert_eq!(*base, Base::Decimal);
                }
                _ => panic!("Expected integer literal"),
            }
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_module_with_multiple_lets() {
    let input = "module Math = 
        let a = 1
        let b = 2
    end";
    let modules = parse_str(input);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].contents.len(), 2);
}

#[test]
fn test_nested_expressions() {
    // Test precedence and binary ops implicitly via structure check
    let input = "module Calc = let res = 1 + 2 * 3 end";
    let modules = parse_str(input);
    let module = &modules[0];

    match &module.contents[0].inner {
        ModuleStatementKind::Let { value, .. } => {
            // Should be 1 + (2 * 3) because * has higher precedence
            match &value.inner {
                ValueExpressionKind::Binary { op, left, right } => {
                    // Expect + at top level
                    match op {
                        BinaryOp::Plus => (),
                        _ => panic!("Expected Plus at top level, got {:?}", op),
                    }

                    // Left should be 1
                    match &left.inner {
                        ValueExpressionKind::Literal(Literal::Integer(val, _)) => {
                            assert_eq!(val, "1")
                        }
                        _ => panic!("Expected 1"),
                    }

                    // Right should be 2 * 3
                    match &right.inner {
                        ValueExpressionKind::Binary { op, left, right } => {
                            match op {
                                BinaryOp::Star => (),
                                _ => panic!("Expected Star, got {:?}", op),
                            }
                            match &left.inner {
                                ValueExpressionKind::Literal(Literal::Integer(val, _)) => {
                                    assert_eq!(val, "2")
                                }
                                _ => panic!("Expected 2"),
                            }
                            match &right.inner {
                                ValueExpressionKind::Literal(Literal::Integer(val, _)) => {
                                    assert_eq!(val, "3")
                                }
                                _ => panic!("Expected 3"),
                            }
                        }
                        _ => panic!("Expected binary expression on right"),
                    }
                }
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected let"),
    }
}

#[test]
fn test_function_definition() {
    let input = "module Funcs = let id = fn x => x end";
    let modules = parse_str(input);
    let module = &modules[0];

    match &module.contents[0].inner {
        ModuleStatementKind::Let { value, .. } => {
            match &value.inner {
                ValueExpressionKind::FunctionDef {
                    parameters, body, ..
                } => {
                    assert_eq!(parameters.len(), 1);
                    assert_eq!(parameters[0].inner, "x");

                    match &body.inner {
                        ValueExpressionKind::Identifier(name) => assert_eq!(name, "x"),
                        _ => panic!("Expected identifier in body"),
                    }
                }
                _ => panic!("Expected function definition"),
            }
        }
        _ => panic!("Expected let"),
    }
}
