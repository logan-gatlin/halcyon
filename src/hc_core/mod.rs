/*!
    The `core` module contains symbols that are required by the compiler.
    These include the standard types, operators, and wrappers for important
    WebAssembly functionality.
*/

mod terms;
mod traits;
mod types;

pub use terms::CoreTerm;
use traits::core_impls;
pub use types::CoreType;

use enum_iterator::all;
use std::collections::HashSet;

use crate::asm::{
    self,
    emit_array_concat,
    lower_type,
    Encoder,
    Instruction,
    Type as LowerType,
};

use crate::ir::{
    Path,
    ScopeKind,
    Specialization,
};
use crate::operator::{
    BinaryOp,
    Operator,
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
use crate::Artifact;

use Instruction as i;

pub const CORE_MODULE_NAME: &str = "core";

pub fn compile_core_module(symbols: &mut SymbolTable) -> Artifact {
    register_core_primitive_symbols(symbols);

    let resolved = resolve_core_source_module(symbols);

    core_impls().into_iter().for_each(|i| {
        symbols
            .insert_impl(i)
            .unwrap_or_else(|e| unreachable!("{e:?}"))
    });
    register_core_impl_method_terms(symbols);

    let elaborated = crate::ir::elaborate_module(resolved, symbols);
    let mut module = asm::lower_module(elaborated, symbols);
    append_manual_core_definitions(&mut module, symbols);

    Artifact {
        module_name: CORE_MODULE_NAME.to_string(),
        ir_module: None,
        binary: asm::encode(module),
    }
}

fn register_core_primitive_symbols(symbols: &mut SymbolTable) {
    all::<CoreType>().for_each(|symbol| {
        symbols.insert(symbol);
    });
    all::<CoreTerm>().for_each(|symbol| {
        symbols.insert(symbol);
    });
}

fn resolve_core_source_module(symbols: &mut SymbolTable) -> crate::types::ResolvedModule {
    let source = include_str!("core.hc");
    let mut logger = crate::Logger::new();
    let mut file_logger = logger.new_file("core.hc", source);

    let Some(source_file) = crate::parse::parse(source, &mut file_logger) else {
        logger.consume_file(file_logger);
        logger.print_logs();
        panic!("failed to parse bundled core module source");
    };

    let Some(module_node) = source_file.modules().into_iter().next() else {
        logger.consume_file(file_logger);
        logger.print_logs();
        panic!("bundled core module source did not contain a module");
    };

    let prelude = all::<CoreType>()
        .map(|symbol| (symbol.path(), crate::ir::NameSpace::Type))
        .collect::<Vec<_>>();

    let Some(ir_module) = crate::ir::module_with_prelude(module_node, &mut file_logger, &prelude)
    else {
        logger.consume_file(file_logger);
        logger.print_logs();
        panic!("failed to build IR for bundled core module source");
    };

    let resolved =
        crate::types::resolve_module_with_symbols_and_schemes(symbols, ir_module, &mut file_logger);

    logger.consume_file(file_logger);
    if !logger.is_ok() {
        logger.print_logs();
        panic!("failed to typecheck bundled core module source");
    }

    resolved
}

fn append_manual_core_definitions(
    module: &mut asm::Module,
    symbols: &SymbolTable,
) {
    let init_name = module.start.clone();
    let mut init = Encoder {
        module,
        func_name: init_name,
        temporary_salt: 1_000_000,
    };
    define_core_terms(&mut init, symbols);
    define_core_impl_methods(&mut init, symbols);
    define_trait_impl_terms(&mut init, symbols);
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

fn define_core_terms(
    init: &mut Encoder<'_>,
    symbols: &SymbolTable,
) {
    for term in all::<CoreTerm>() {
        match term {
            CoreTerm::EmptyArray => define_empty_array(init, symbols),
            CoreTerm::ArrayConcat => define_array_concat(init, symbols),
            CoreTerm::ArrayPush => define_array_push(init, symbols),
        }
    }
}

fn define_empty_array(
    init: &mut Encoder<'_>,
    _symbols: &SymbolTable,
) {
    let path = CoreTerm::EmptyArray.path();
    init.extend([i::I32Const(0), i::ArrayNewDefault(LowerType::Any)]);
    store_global(init, path, LowerType::Array(LowerType::Any.into()));
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
    define_array_impl_methods(init, symbols);
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
