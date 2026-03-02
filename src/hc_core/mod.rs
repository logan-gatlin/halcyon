/*!
    The `core` module contains symbols that are required by the compiler.
    These include the standard types, operators, and wrappers for important
    WebAssembly functionality.
*/

mod terms;
mod traits;
mod types;

pub use terms::CoreTerm;
pub use traits::CoreTrait;
use traits::core_impls;
pub use types::CoreType;

use enum_iterator::all;
use std::collections::HashSet;

use crate::asm::custom_section::TypeSignatureSection;
use crate::asm::{
    self,
    Encoder,
    Instruction,
    NumberOperation,
    Type as LowerType,
    emit_array_concat,
    emit_string_compare,
    lower_type,
};

use crate::Artifact;
use crate::ir::{
    Path,
    ScopeKind,
    Specialization,
};
use crate::operator::{
    BinaryOp,
    Operator,
    UnaryOp,
};
use crate::types::symbol_table::{
    Symbol,
    SymbolKind,
};
use crate::types::{
    SymbolTable,
    TraitDef,
    Type,
    TypeScheme,
};

use Instruction as i;

pub const CORE_MODULE_NAME: &str = "core";

pub fn compile_core_module(symbols: &mut SymbolTable) -> Artifact {
    all::<CoreTerm>().for_each(|s| {
        symbols.insert(s);
    });
    all::<CoreType>().for_each(|s| {
        symbols.insert(s);
    });
    all::<CoreTrait>().for_each(|s| {
        symbols.insert(s);
    });
    core_impls().into_iter().for_each(|i| {
        symbols
            .insert_impl(i)
            .unwrap_or_else(|e| unreachable!("{e:?}"))
    });

    register_core_impl_method_terms(symbols);

    compile_core_artifact(symbols)
}

pub(crate) fn core_impl_arguments(arguments: &[Type]) -> Vec<Type> {
    arguments.iter().map(normalize_impl_argument).collect()
}

pub(crate) fn core_impl_path(
    method_path: &Path,
    arguments: &[Type],
) -> Path {
    let args = core_impl_arguments(arguments);
    let arg_key = args.iter().map(type_key).collect::<Vec<_>>().join("_");
    let minor = if arg_key.is_empty() {
        format!("[impl] {} {}", method_path.major, method_path.minor)
    } else {
        format!(
            "[impl] {} {} {}",
            method_path.major, method_path.minor, arg_key
        )
    };
    Path::new(CORE_MODULE_NAME, minor)
}

pub(crate) fn core_impl_method_path(
    method_path: &Path,
    argument: &Type,
) -> Path {
    let normalized = normalize_impl_argument(argument);
    Path::new(
        CORE_MODULE_NAME,
        format!(
            "[impl method] {} {} {}",
            method_path.major,
            method_path.minor,
            type_key(&normalized)
        ),
    )
}

pub(crate) fn normalize_impl_argument(type_: &Type) -> Type {
    let mut current = type_.clone();
    while let Type::ForAll(body) = current {
        current = *body;
    }
    current
}

fn type_key(type_: &Type) -> String {
    type_
        .pretty()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn register_core_impl_method_terms(symbols: &mut SymbolTable) {
    let trait_defs = symbols.trait_defs().clone();
    let trait_impls = symbols.trait_impls().clone();
    for (trait_name, impls) in trait_impls {
        let Some(def) = trait_defs.get(&trait_name) else {
            continue;
        };
        let methods = ordered_trait_methods(def);
        for trait_impl in impls {
            for (method_path, method_scheme) in methods.iter() {
                let Some(impl_path) = trait_impl.methods.get(method_path).cloned() else {
                    continue;
                };
                let Some(type_) =
                    instantiate_scheme_type(&method_scheme.type_, &trait_impl.head.arguments)
                else {
                    continue;
                };
                symbols.insert_term(impl_path, TypeScheme::new(type_));
            }
        }
    }
}

fn instantiate_scheme_type(
    type_: &Type,
    arguments: &[Type],
) -> Option<Type> {
    arguments
        .iter()
        .try_fold(type_.clone(), |current, argument| {
            current.open_forall(argument)
        })
}

fn compile_core_artifact(symbols: &SymbolTable) -> Artifact {
    let mut module = asm::Module::new(CORE_MODULE_NAME.to_string());
    let init_name = Path::new(CORE_MODULE_NAME, "[init]");
    let mut init = module.new_function(init_name.clone());

    define_core_terms(&mut init, symbols);
    define_core_impl_methods(&mut init, symbols);
    define_trait_impl_terms(&mut init, symbols);

    module.start = init_name;
    module.sig = TypeSignatureSection::new(CORE_MODULE_NAME, symbols);

    Artifact {
        module_name: CORE_MODULE_NAME.to_string(),
        ir_module: None,
        binary: asm::encode(module),
    }
}

fn define_core_terms(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    for term in all::<CoreTerm>() {
        if let Some(definition) = core_term_definition(term) {
            (definition.build)(init, symbols);
            continue;
        }
        match term {
            CoreTerm::BinaryOp(op) => {
                define_trait_dispatch(init, symbols, &op.path());
            }
            CoreTerm::UnaryOp(op) => {
                define_trait_dispatch(init, symbols, &op.path());
            }
            _ => unreachable!(),
        }
    }
}

type CoreTermBuilder = fn(&mut Encoder<'_>, &SymbolTable);

struct CoreTermDefinition {
    term: CoreTerm,
    build: CoreTermBuilder,
}

const CORE_TERM_DEFINITIONS: &[CoreTermDefinition] = &[
    CoreTermDefinition {
        term: CoreTerm::BinaryOp(BinaryOp::ComposeLeft),
        build: define_compose_left,
    },
    CoreTermDefinition {
        term: CoreTerm::BinaryOp(BinaryOp::ComposeRight),
        build: define_compose_right,
    },
    CoreTermDefinition {
        term: CoreTerm::BinaryOp(BinaryOp::Apply),
        build: define_apply,
    },
    CoreTermDefinition {
        term: CoreTerm::BinaryOp(BinaryOp::Semicolon),
        build: define_semicolon,
    },
    CoreTermDefinition {
        term: CoreTerm::BinaryOp(BinaryOp::LessEqual),
        build: define_less_equal,
    },
    CoreTermDefinition {
        term: CoreTerm::BinaryOp(BinaryOp::GreaterEqual),
        build: define_greater_equal,
    },
    CoreTermDefinition {
        term: CoreTerm::EmptyArray,
        build: define_empty_array,
    },
    CoreTermDefinition {
        term: CoreTerm::ArrayConcat,
        build: define_array_concat,
    },
    CoreTermDefinition {
        term: CoreTerm::ArrayPush,
        build: define_array_push,
    },
];

fn core_term_definition(term: CoreTerm) -> Option<&'static CoreTermDefinition> {
    CORE_TERM_DEFINITIONS
        .iter()
        .find(|definition| definition.term == term)
}

fn define_empty_array(
    init: &mut Encoder<'_>,
    _symbols: &SymbolTable,
) {
    let path = CoreTerm::EmptyArray.path();
    init.extend([i::I32Const(0), i::ArrayNewDefault(LowerType::Any)]);
    store_global(init, path, LowerType::Array(LowerType::Any.into()));
}

fn define_less_equal(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    define_compare_equal_wrapper(init, symbols, BinaryOp::Less);
}

fn define_greater_equal(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    define_compare_equal_wrapper(init, symbols, BinaryOp::Greater);
}

fn store_global(
    init: &mut Encoder<'_>,
    path: Path,
    type_: LowerType,
) {
    init.new_register(path.clone(), ScopeKind::Global, type_);
    init.push(i::Set(path));
}

fn store_global_closure(
    init: &mut Encoder<'_>,
    path: Path,
) {
    store_global(init, path, LowerType::closure_type());
}

fn define_curried_binary(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
    path: Path,
    left_type: Type,
    right_type: Type,
    body: impl for<'b> FnOnce(&mut Encoder<'b>, &Path, &Path),
) {
    let left_name = init.temporary_name("left");
    init.create_closure(symbols, left_name.clone(), left_type.clone(), vec![], {
        let left_name = left_name.clone();
        move |outer, symbols| {
            let right_name = outer.temporary_name("right");
            outer.create_closure(
                symbols,
                right_name.clone(),
                right_type,
                vec![(left_name.clone(), left_type)],
                {
                    let left_name = left_name.clone();
                    move |inner, _symbols| body(inner, &left_name, &right_name)
                },
            );
        }
    });
    store_global_closure(init, path);
}

fn define_curried_binary_to_unary(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
    path: Path,
    first_type: Type,
    second_type: Type,
    value_type: Type,
    body: impl for<'b> FnOnce(&mut Encoder<'b>, &Path, &Path, &Path),
) {
    let first_name = init.temporary_name("first");
    init.create_closure(symbols, first_name.clone(), first_type.clone(), vec![], {
        let first_name = first_name.clone();
        move |outer, symbols| {
            let second_name = outer.temporary_name("second");
            outer.create_closure(
                symbols,
                second_name.clone(),
                second_type.clone(),
                vec![(first_name.clone(), first_type.clone())],
                {
                    let first_name = first_name.clone();
                    let value_type = value_type.clone();
                    move |outer, symbols| {
                        let value_name = outer.temporary_name("value");
                        outer.create_closure(
                            symbols,
                            value_name.clone(),
                            value_type,
                            vec![
                                (first_name.clone(), first_type),
                                (second_name.clone(), second_type),
                            ],
                            {
                                let first_name = first_name.clone();
                                let second_name = second_name.clone();
                                move |inner, _symbols| {
                                    body(inner, &first_name, &second_name, &value_name)
                                }
                            },
                        );
                    }
                },
            );
        }
    });
    store_global_closure(init, path);
}

fn define_trait_dispatch(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
    method_path: &Path,
) {
    let Some((methods, method_index)) = ordered_methods_for(symbols, method_path) else {
        return;
    };
    let dict_type = dictionary_type(&methods);
    let dict_fields = lowered_struct_fields(&dict_type, symbols);
    let dict_name = init.temporary_name("dict");

    init.create_closure(symbols, dict_name.clone(), dict_type, vec![], {
        let dict_name = dict_name.clone();
        move |inner, _symbols| {
            inner.extend([
                i::Get(dict_name.clone()),
                i::StructGet(dict_fields.clone(), method_index),
            ]);
        }
    });

    store_global_closure(init, method_path.clone());
}

fn define_compare_equal_wrapper(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
    compare_method: BinaryOp,
) {
    let path = match compare_method {
        BinaryOp::Less => BinaryOp::LessEqual.path(),
        BinaryOp::Greater => BinaryOp::GreaterEqual.path(),
        _ => return,
    };

    let compare_trait = Path::core("compare");
    let equal_trait = Path::core("equal");
    let Some(compare_def) = symbols.trait_defs().get(&compare_trait) else {
        return;
    };
    let Some(equal_def) = symbols.trait_defs().get(&equal_trait) else {
        return;
    };

    let compare_methods = ordered_trait_methods(compare_def);
    let equal_methods = ordered_trait_methods(equal_def);
    let compare_type = dictionary_type(&compare_methods);
    let equal_type = dictionary_type(&equal_methods);
    let compare_fields = lowered_struct_fields(&compare_type, symbols);
    let equal_fields = lowered_struct_fields(&equal_type, symbols);
    let bool_fields = lowered_struct_fields(&Type::Boolean, symbols);
    let compare_index = compare_methods
        .iter()
        .position(|(method_path, _)| method_path == &compare_method.path())
        .unwrap_or_else(|| unreachable!());
    let equal_index = equal_methods
        .iter()
        .position(|(method_path, _)| method_path == &BinaryOp::DoubleEqual.path())
        .unwrap_or_else(|| unreachable!());

    let compare_dict = init.temporary_name("compare_dict");
    init.create_closure(
        symbols,
        compare_dict.clone(),
        compare_type.clone(),
        vec![],
        {
            let compare_dict = compare_dict.clone();
            let equal_type = equal_type.clone();
            move |outer, symbols| {
                let equal_dict = outer.temporary_name("equal_dict");
                outer.create_closure(
                    symbols,
                    equal_dict.clone(),
                    equal_type.clone(),
                    vec![(compare_dict.clone(), compare_type.clone())],
                    {
                        let compare_dict = compare_dict.clone();
                        move |outer, symbols| {
                            let left = outer.temporary_name("left");
                            let value_type = Type::v(0);
                            outer.create_closure(
                                symbols,
                                left.clone(),
                                value_type.clone(),
                                vec![
                                    (compare_dict.clone(), compare_type.clone()),
                                    (equal_dict.clone(), equal_type.clone()),
                                ],
                                {
                                    let compare_dict = compare_dict.clone();
                                    let equal_dict = equal_dict.clone();
                                    let value_type = value_type.clone();
                                    move |outer, symbols| {
                                        let right = outer.temporary_name("right");
                                        outer.create_closure(
                                            symbols,
                                            right.clone(),
                                            value_type,
                                            vec![
                                                (compare_dict.clone(), compare_type.clone()),
                                                (equal_dict.clone(), equal_type.clone()),
                                                (left.clone(), Type::v(0)),
                                            ],
                                            {
                                                let compare_dict = compare_dict.clone();
                                                let equal_dict = equal_dict.clone();
                                                let left = left.clone();
                                                let right = right.clone();
                                                move |inner, _symbols| {
                                                    let compare_result =
                                                        inner.temporary_name("compare_result");
                                                    inner.new_register(
                                                        compare_result.clone(),
                                                        ScopeKind::Local,
                                                        LowerType::Struct(bool_fields.clone()),
                                                    );

                                                    inner.extend([
                                                        i::Get(left.clone()),
                                                        i::Get(compare_dict.clone()),
                                                        i::StructGet(
                                                            compare_fields.clone(),
                                                            compare_index,
                                                        ),
                                                    ]);
                                                    inner.call_closure();
                                                    inner.push(i::Get(right.clone()));
                                                    inner.call_closure();
                                                    inner.push(i::RefCastStruct(
                                                        bool_fields.clone(),
                                                    ));
                                                    inner.push(i::Set(compare_result.clone()));

                                                    inner.extend([
                                                        i::Get(compare_result),
                                                        i::StructGet(bool_fields.clone(), 0),
                                                        i::If(Some(LowerType::Struct(
                                                            bool_fields.clone(),
                                                        ))),
                                                        i::I32Const(1),
                                                        i::StructNew(bool_fields.clone()),
                                                        i::Else,
                                                    ]);

                                                    inner.extend([
                                                        i::Get(left),
                                                        i::Get(equal_dict),
                                                        i::StructGet(
                                                            equal_fields.clone(),
                                                            equal_index,
                                                        ),
                                                    ]);
                                                    inner.call_closure();
                                                    inner.push(i::Get(right));
                                                    inner.call_closure();
                                                    inner.push(i::RefCastStruct(
                                                        bool_fields.clone(),
                                                    ));
                                                    inner.push(i::End);
                                                }
                                            },
                                        );
                                    }
                                },
                            );
                        }
                    },
                );
            }
        },
    );

    store_global_closure(init, path);
}

fn define_compose_left(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let path = BinaryOp::ComposeLeft.path();
    let first_type = Type::func(Type::v(2), Type::v(1));
    let second_type = Type::func(Type::v(1), Type::v(0));
    define_curried_binary_to_unary(
        init,
        symbols,
        path,
        first_type,
        second_type,
        Type::v(2),
        move |inner, first, second, value| {
            inner.extend([i::Get(value.clone()), i::Get(first.clone())]);
            inner.call_closure();
            inner.push(i::Get(second.clone()));
            inner.call_closure();
        },
    );
}

fn define_compose_right(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let path = BinaryOp::ComposeRight.path();
    let first_type = Type::func(Type::v(1), Type::v(0));
    let second_type = Type::func(Type::v(2), Type::v(1));
    define_curried_binary_to_unary(
        init,
        symbols,
        path,
        first_type,
        second_type,
        Type::v(2),
        move |inner, first, second, value| {
            inner.extend([i::Get(value.clone()), i::Get(second.clone())]);
            inner.call_closure();
            inner.push(i::Get(first.clone()));
            inner.call_closure();
        },
    );
}

fn define_apply(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let path = BinaryOp::Apply.path();
    let value_type = Type::v(1);
    let function_type = Type::func(Type::v(1), Type::v(0));
    define_curried_binary(
        init,
        symbols,
        path,
        value_type,
        function_type,
        move |inner, value, function| {
            inner.extend([i::Get(value.clone()), i::Get(function.clone())]);
            inner.call_closure();
        },
    );
}

fn define_semicolon(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let path = BinaryOp::Semicolon.path();
    define_curried_binary(
        init,
        symbols,
        path,
        Type::Unit,
        Type::v(0),
        move |inner, _unit, kept| {
            inner.push(i::Get(kept.clone()));
        },
    );
}

fn define_array_concat(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let path = CoreTerm::ArrayConcat.path();
    let array_type = Type::Array(Type::v(0).into());
    define_curried_binary(
        init,
        symbols,
        path,
        array_type.clone(),
        array_type,
        move |inner, left, right| emit_array_concat(inner, left, right, LowerType::Any),
    );
}

fn define_array_push(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let path = CoreTerm::ArrayPush.path();
    let array_type = Type::Array(Type::v(0).into());
    define_curried_binary(
        init,
        symbols,
        path,
        Type::v(0),
        array_type,
        move |inner, value, array| {
            let singleton = inner.temporary_name("singleton");
            inner.new_register(
                singleton.clone(),
                ScopeKind::Local,
                LowerType::Array(LowerType::Any.into()),
            );
            inner.extend([
                i::Get(value.clone()),
                i::ArrayNewFixed {
                    inner_type: LowerType::Any,
                    length: 1,
                },
                i::Set(singleton.clone()),
                i::Get(array.clone()),
                i::Get(CoreTerm::ArrayConcat.path()),
            ]);
            inner.call_closure();
            inner.push(i::Get(singleton));
            inner.call_closure();
        },
    );
}

fn define_core_impl_methods(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    define_unit_compare_impl_methods(init, symbols);
    define_integer_impl_methods(init, symbols);
    define_real_impl_methods(init, symbols);
    define_boolean_impl_methods(init, symbols);
    define_glyph_impl_methods(init, symbols);
    define_string_impl_methods(init, symbols);
    define_array_impl_methods(init, symbols);
}

fn define_unit_compare_impl_methods(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let true_path = core_impl_method_path(&BinaryOp::DoubleEqual.path(), &Type::Unit);
    define_constant_boolean_method(init, symbols, true_path, true);

    let false_paths = [
        core_impl_method_path(&BinaryOp::BangEqual.path(), &Type::Unit),
        core_impl_method_path(&BinaryOp::Less.path(), &Type::Unit),
        core_impl_method_path(&BinaryOp::Greater.path(), &Type::Unit),
    ];
    for path in false_paths {
        define_constant_boolean_method(init, symbols, path, false);
    }
}

fn define_integer_impl_methods(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let integer = Type::Integer;
    for (method, op, boolean_result) in [
        (BinaryOp::Plus.path(), NumberOperation::Add, false),
        (BinaryOp::Minus.path(), NumberOperation::Sub, false),
        (BinaryOp::Star.path(), NumberOperation::Mul, false),
        (BinaryOp::Slash.path(), NumberOperation::Div, false),
        (BinaryOp::Percent.path(), NumberOperation::Rem, false),
        (BinaryOp::DoubleEqual.path(), NumberOperation::Eq, true),
        (BinaryOp::BangEqual.path(), NumberOperation::Ne, true),
        (BinaryOp::Less.path(), NumberOperation::Lt, true),
        (BinaryOp::Greater.path(), NumberOperation::Gt, true),
        (BinaryOp::And.path(), NumberOperation::And, false),
        (BinaryOp::Or.path(), NumberOperation::Or, false),
        (BinaryOp::Xor.path(), NumberOperation::Xor, false),
    ] {
        let path = core_impl_method_path(&method, &integer);
        define_i64_binary_method(init, symbols, path, op, boolean_result);
    }

    let neg_path = core_impl_method_path(&UnaryOp::Minus.path(), &integer);
    define_i64_unary_method(init, symbols, neg_path, |inner, value, fields| {
        inner.extend([
            i::Const(crate::ir::ImmediateValue::Integer(0)),
            i::Get(value.clone()),
            i::StructGet(fields.clone(), 0),
            i::I64Op(NumberOperation::Sub),
            i::StructNew(fields.clone()),
        ]);
    });

    let not_path = core_impl_method_path(&UnaryOp::Not.path(), &integer);
    define_i64_unary_method(init, symbols, not_path, |inner, value, fields| {
        inner.extend([
            i::Get(value.clone()),
            i::StructGet(fields.clone(), 0),
            i::Const(crate::ir::ImmediateValue::Integer(-1)),
            i::I64Op(NumberOperation::Xor),
            i::StructNew(fields.clone()),
        ]);
    });
}

fn define_real_impl_methods(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let real = Type::Real;
    for (method, op, boolean_result) in [
        (BinaryOp::Plus.path(), NumberOperation::Add, false),
        (BinaryOp::Minus.path(), NumberOperation::Sub, false),
        (BinaryOp::Star.path(), NumberOperation::Mul, false),
        (BinaryOp::Slash.path(), NumberOperation::Div, false),
        (BinaryOp::DoubleEqual.path(), NumberOperation::Eq, true),
        (BinaryOp::BangEqual.path(), NumberOperation::Ne, true),
        (BinaryOp::Less.path(), NumberOperation::Lt, true),
        (BinaryOp::Greater.path(), NumberOperation::Gt, true),
    ] {
        let path = core_impl_method_path(&method, &real);
        define_f64_binary_method(init, symbols, path, op, boolean_result);
    }

    let neg_path = core_impl_method_path(&UnaryOp::Minus.path(), &real);
    define_f64_unary_method(init, symbols, neg_path, |inner, value, fields| {
        inner.extend([
            i::Const(crate::ir::ImmediateValue::Real(0.0)),
            i::Get(value.clone()),
            i::StructGet(fields.clone(), 0),
            i::F64Op(NumberOperation::Sub),
            i::StructNew(fields.clone()),
        ]);
    });
}

fn define_boolean_impl_methods(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let boolean = Type::Boolean;
    for (method, op, boolean_result) in [
        (BinaryOp::DoubleEqual.path(), NumberOperation::Eq, true),
        (BinaryOp::BangEqual.path(), NumberOperation::Ne, true),
        (BinaryOp::Less.path(), NumberOperation::Lt, true),
        (BinaryOp::Greater.path(), NumberOperation::Gt, true),
        (BinaryOp::And.path(), NumberOperation::And, false),
        (BinaryOp::Or.path(), NumberOperation::Or, false),
        (BinaryOp::Xor.path(), NumberOperation::Xor, false),
    ] {
        let path = core_impl_method_path(&method, &boolean);
        define_i32_binary_method(init, symbols, path, Type::Boolean, op, boolean_result);
    }

    let not_path = core_impl_method_path(&UnaryOp::Not.path(), &boolean);
    define_i32_unary_method(init, symbols, not_path, |inner, value, fields| {
        inner.extend([
            i::Get(value.clone()),
            i::StructGet(fields.clone(), 0),
            i::I32Const(1),
            i::I32Op(NumberOperation::Xor),
            i::StructNew(fields.clone()),
        ]);
    });
}

fn define_glyph_impl_methods(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let glyph = Type::Glyph;
    for (method, op) in [
        (BinaryOp::DoubleEqual.path(), NumberOperation::Eq),
        (BinaryOp::BangEqual.path(), NumberOperation::Ne),
        (BinaryOp::Less.path(), NumberOperation::Lt),
        (BinaryOp::Greater.path(), NumberOperation::Gt),
    ] {
        let path = core_impl_method_path(&method, &glyph);
        define_i32_binary_method(init, symbols, path, Type::Glyph, op, true);
    }
}

fn define_string_impl_methods(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let string = Type::String;
    let add_path = core_impl_method_path(&BinaryOp::Plus.path(), &string);
    define_curried_binary(
        init,
        symbols,
        add_path,
        Type::String,
        Type::String,
        move |inner, left, right| emit_array_concat(inner, left, right, LowerType::I8),
    );

    for (method, op) in [
        (BinaryOp::DoubleEqual.path(), NumberOperation::Eq),
        (BinaryOp::BangEqual.path(), NumberOperation::Ne),
        (BinaryOp::Less.path(), NumberOperation::Lt),
        (BinaryOp::Greater.path(), NumberOperation::Gt),
    ] {
        let path = core_impl_method_path(&method, &string);
        let bool_fields = lowered_struct_fields(&Type::Boolean, symbols);
        define_curried_binary(
            init,
            symbols,
            path,
            Type::String,
            Type::String,
            move |inner, left, right| {
                emit_string_compare(inner, left, right, op, bool_fields.clone())
            },
        );
    }
}

fn define_array_impl_methods(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let path = core_impl_method_path(&BinaryOp::Plus.path(), &Type::array());
    let array_concat = CoreTerm::ArrayConcat.path();
    init.new_register(path.clone(), ScopeKind::Global, LowerType::closure_type());
    if array_concat.major != init.module.name {
        init.module
            .imports
            .entry(array_concat.clone())
            .or_insert_with(|| {
                symbols
                    .terms()
                    .get(&array_concat)
                    .map(|scheme| lower_type(&scheme.type_, symbols))
                    .unwrap_or_else(LowerType::closure_type)
            });
    }
    init.extend([i::Get(array_concat), i::Set(path)]);
}

fn define_constant_boolean_method(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
    path: Path,
    value: bool,
) {
    let bool_fields = lowered_struct_fields(&Type::Boolean, symbols);
    define_curried_binary(
        init,
        symbols,
        path,
        Type::Unit,
        Type::Unit,
        move |inner, _left, _right| {
            inner.extend([
                i::I32Const(if value { 1 } else { 0 }),
                i::StructNew(bool_fields.clone()),
            ]);
        },
    );
}

fn define_i64_binary_method(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
    path: Path,
    op: NumberOperation,
    boolean_result: bool,
) {
    let value_fields: Box<[LowerType]> = [LowerType::I64].into();
    let result_fields = if boolean_result {
        lowered_struct_fields(&Type::Boolean, symbols)
    } else {
        value_fields.clone()
    };
    define_curried_binary(
        init,
        symbols,
        path,
        Type::Integer,
        Type::Integer,
        move |inner, left, right| {
            inner.extend([
                i::Get(left.clone()),
                i::StructGet(value_fields.clone(), 0),
                i::Get(right.clone()),
                i::StructGet(value_fields.clone(), 0),
                i::I64Op(op),
                i::StructNew(result_fields.clone()),
            ]);
        },
    );
}

fn define_i32_binary_method(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
    path: Path,
    argument_type: Type,
    op: NumberOperation,
    boolean_result: bool,
) {
    let value_fields: Box<[LowerType]> = [LowerType::I32].into();
    let result_fields = if boolean_result {
        lowered_struct_fields(&Type::Boolean, symbols)
    } else {
        value_fields.clone()
    };
    define_curried_binary(
        init,
        symbols,
        path,
        argument_type.clone(),
        argument_type,
        move |inner, left, right| {
            inner.extend([
                i::Get(left.clone()),
                i::StructGet(value_fields.clone(), 0),
                i::Get(right.clone()),
                i::StructGet(value_fields.clone(), 0),
                i::I32Op(op),
                i::StructNew(result_fields.clone()),
            ]);
        },
    );
}

fn define_f64_binary_method(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
    path: Path,
    op: NumberOperation,
    boolean_result: bool,
) {
    let value_fields: Box<[LowerType]> = [LowerType::F64].into();
    let result_fields = if boolean_result {
        lowered_struct_fields(&Type::Boolean, symbols)
    } else {
        value_fields.clone()
    };
    define_curried_binary(
        init,
        symbols,
        path,
        Type::Real,
        Type::Real,
        move |inner, left, right| {
            inner.extend([
                i::Get(left.clone()),
                i::StructGet(value_fields.clone(), 0),
                i::Get(right.clone()),
                i::StructGet(value_fields.clone(), 0),
                i::F64Op(op),
                i::StructNew(result_fields.clone()),
            ]);
        },
    );
}

fn define_curried_unary(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
    path: Path,
    argument_type: Type,
    body: impl for<'b> FnOnce(&mut Encoder<'b>, &Path),
) {
    let value_name = init.temporary_name("value");
    init.create_closure(symbols, value_name.clone(), argument_type, vec![], {
        let value_name = value_name.clone();
        move |inner, _symbols| body(inner, &value_name)
    });
    store_global_closure(init, path);
}

fn define_i64_unary_method(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
    path: Path,
    body: impl for<'b> FnOnce(&mut Encoder<'b>, &Path, &Box<[LowerType]>),
) {
    let value_fields: Box<[LowerType]> = [LowerType::I64].into();
    define_curried_unary(init, symbols, path, Type::Integer, move |inner, value| {
        body(inner, value, &value_fields)
    });
}

fn define_i32_unary_method(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
    path: Path,
    body: impl for<'b> FnOnce(&mut Encoder<'b>, &Path, &Box<[LowerType]>),
) {
    let value_fields: Box<[LowerType]> = [LowerType::I32].into();
    define_curried_unary(init, symbols, path, Type::Boolean, move |inner, value| {
        body(inner, value, &value_fields)
    });
}

fn define_f64_unary_method(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
    path: Path,
    body: impl for<'b> FnOnce(&mut Encoder<'b>, &Path, &Box<[LowerType]>),
) {
    let value_fields: Box<[LowerType]> = [LowerType::F64].into();
    define_curried_unary(init, symbols, path, Type::Real, move |inner, value| {
        body(inner, value, &value_fields)
    });
}

fn define_trait_impl_terms(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    let mut seen = HashSet::new();
    for (trait_name, impls) in symbols.trait_impls() {
        let Some(def) = symbols.trait_defs().get(trait_name) else {
            continue;
        };
        let methods = ordered_trait_methods(def);
        for trait_impl in impls {
            let arguments = core_impl_arguments(&trait_impl.head.arguments);
            for (method_path, _) in methods.iter() {
                let specialized_path = core_impl_path(method_path, &arguments);
                if !seen.insert(specialized_path.clone()) {
                    continue;
                }
                init.lower_specialization(
                    &Specialization {
                        method_path: method_path.clone(),
                        arguments: arguments.clone(),
                        specialized_path,
                    },
                    symbols,
                );
            }
        }
    }

    define_compare_specializations(init, symbols, &mut seen);
}

fn define_compare_specializations(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
    seen: &mut HashSet<Path>,
) {
    let compare_trait = Path::core("compare");
    let equal_trait = Path::core("equal");
    let compare_args = trait_impl_argument_heads(symbols, &compare_trait);
    let equal_args = trait_impl_argument_heads(symbols, &equal_trait);

    for argument in compare_args {
        if !equal_args.contains(&argument) {
            continue;
        }
        for method in [BinaryOp::LessEqual.path(), BinaryOp::GreaterEqual.path()] {
            let specialized_path = core_impl_path(&method, std::slice::from_ref(&argument));
            if !seen.insert(specialized_path.clone()) {
                continue;
            }
            init.lower_specialization(
                &Specialization {
                    method_path: method,
                    arguments: vec![argument.clone()],
                    specialized_path,
                },
                symbols,
            );
        }
    }
}

fn trait_impl_argument_heads(
    symbols: &SymbolTable,
    trait_name: &Path,
) -> Vec<Type> {
    symbols
        .trait_impls()
        .get(trait_name)
        .into_iter()
        .flat_map(|impls| impls.iter())
        .filter_map(|impl_| impl_.head.arguments.first().cloned())
        .map(|arg| core_impl_arguments(&[arg])[0].clone())
        .collect()
}

fn ordered_methods_for(
    symbols: &SymbolTable,
    method_path: &Path,
) -> Option<(Vec<(Path, TypeScheme)>, usize)> {
    symbols.trait_defs().values().find_map(|def| {
        let methods = ordered_trait_methods(def);
        methods
            .iter()
            .position(|(path, _)| path == method_path)
            .map(|index| (methods, index))
    })
}

fn ordered_trait_methods(def: &TraitDef) -> Vec<(Path, TypeScheme)> {
    let mut methods = def
        .methods
        .iter()
        .map(|(path, scheme)| (path.clone(), scheme.clone()))
        .collect::<Vec<_>>();
    methods.sort_by(|(left, _), (right, _)| method_key(left).cmp(&method_key(right)));
    methods
}

fn method_key(path: &Path) -> (String, String) {
    (path.major.clone(), path.minor.clone())
}

fn dictionary_type(methods: &[(Path, TypeScheme)]) -> Type {
    Type::Struct {
        fields: methods
            .iter()
            .map(|(path, scheme)| (path.minor.clone(), scheme.type_.clone()))
            .collect(),
    }
}

fn lowered_struct_fields(
    type_: &Type,
    symbols: &SymbolTable,
) -> Box<[LowerType]> {
    match lower_type(type_, symbols) {
        LowerType::Struct(fields) => fields,
        _ => unreachable!(),
    }
}
