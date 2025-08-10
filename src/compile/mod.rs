mod asm_types;
mod code_generation;
mod function_encoder;
mod lower;

use std::collections::HashMap;
use wasm_encoder::Instruction::*;
use wasm_encoder::*;

use crate::{ir::*, operator::*, semantic::*};

use function_encoder::*;

#[derive(Debug, Clone)]
struct Import {
    major: String,
    minor: String,
    entity: EntityType,
}

#[derive(Debug, Clone)]
struct Export {
    name: String,
    kind: ExportKind,
    index: u32,
}

#[derive(Debug, Clone, Copy)]
enum FunctionKind {
    Native(u32),
    Import(u32),
}

#[derive(Debug, Clone)]
enum RegisteredType {
    Function(FuncType),
    Array(StorageType),
    Struct(Vec<StorageType>),
}

#[derive(Debug, Clone, Default)]
pub struct ModuleEncoder {
    pub main_fn: u32,
    type_map: HashMap<Type, u32>,
    raw_type_map: HashMap<Type, u32>,
    global_map: HashMap<Path, u32>,
    global_section: Vec<u32>,
    import_section: Vec<Import>,
    export_section: Vec<Export>,
    type_section: Vec<RegisteredType>,
    function_section: Vec<u32>,
    elements_section: Vec<FunctionKind>,
    code_section: Vec<FunctionEncoder>,
}

impl ModuleEncoder {
    pub fn new() -> Self {
        let mut this = Self::default();
        let main = this.new_main_function();
        this.main_fn = main;
        this
    }

    pub fn encode_ir(&mut self, mut ir: IrModule) {
        for item in ir.items.clone() {
            match item {
                ModuleItem::Let(mangle, ptr) => {
                    mangle.iter_names(&mut |n, t| {
                        self.new_global(n.clone(), t.clone());
                    });
                    let value_t = self.get_asm_type(mangle.type_.clone());
                    let temporary = self.func_mut(self.main_fn).new_temporary(value_t.val);
                    lower::lower(&mut ir, ptr, self, self.main_fn);
                    self.push(self.main_fn, LocalSet(temporary));
                    lower::lower_pattern(mangle, self, temporary, self.main_fn, true);
                }
                ModuleItem::Constructor(
                    name,
                    Constructor {
                        variant,
                        in_type,
                        out_type,
                    },
                ) => {
                    let ftype = Type::func(in_type.clone(), out_type.clone());
                    let global_id = self.new_global(name, ftype.clone());
                    let parameter: Path = "a".into();
                    let f = self.new_function(ftype.clone(), parameter.clone(), vec![], vec![]);
                    // Assign to global
                    self.push(self.main_fn, I32Const(f as i32));
                    self.new_capture(self.main_fn, 0);
                    self.new_struct(self.main_fn, ftype);
                    self.push(self.main_fn, GlobalSet(global_id));
                    // Create constructor func
                    self.push(f, I32Const(variant as i32));
                    self.get_symbol(f, &parameter);
                    self.new_struct(f, out_type);
                }
                _ => {}
            }
        }
    }

    pub fn new_global(&mut self, mangle: Path, type_: Type) -> u32 {
        let id = self.global_section.len() as u32;
        let type_ = self.get_asm_type(type_.clone()).id.unwrap();
        self.global_section.push(type_);
        self.export_section.push(Export {
            name: mangle.to_string(),
            kind: ExportKind::Global,
            index: id,
        });
        self.global_map.insert(mangle, id);
        id
    }

    pub fn finish(self) -> Vec<u8> {
        let FunctionKind::Native(main_func) = self.elements_section[self.main_fn as usize] else {
            panic!()
        };

        let mut name_section = NameSection::new();
        let mut type_names_map = self
            .type_map
            .clone()
            .into_iter()
            .map(|(type_, id)| (id, format!("{}", type_)))
            .chain(
                self.raw_type_map
                    .clone()
                    .into_iter()
                    .map(|(type_, id)| (id, format!("(raw) {}", type_))),
            )
            .collect::<Vec<_>>();
        type_names_map.sort_by(|(id1, _), (id2, _)| id1.cmp(id2));
        name_section.types(&type_names_map.into_iter().fold(
            NameMap::new(),
            |mut names, (id, type_)| {
                names.append(id, &type_);
                names
            },
        ));
        name_section.locals(
            &self
                .code_section
                .clone()
                .into_iter()
                .enumerate()
                .map(|(id, code)| (id, code.encode_name_map()))
                .fold(IndirectNameMap::new(), |mut indirect_map, (id, map)| {
                    indirect_map.append(id as u32, &map);
                    indirect_map
                }),
        );

        let no_imports = self.import_section.len() as u32;
        let no_funcs = self.function_section.len() as u32;

        let elements_section = ElementSection::new()
            .segment(ElementSegment {
                mode: ElementMode::Active {
                    table: None,
                    offset: &ConstExpr::i32_const(0),
                },
                elements: Elements::Functions(std::borrow::Cow::from(
                    &self
                        .elements_section
                        .clone()
                        .into_iter()
                        .map(|e| match e {
                            FunctionKind::Native(f) => f + no_imports,
                            FunctionKind::Import(f) => f,
                        })
                        .collect::<Vec<_>>(),
                )),
            })
            .clone();

        Module::new()
            .section(&name_section)
            // Type section
            .section(&self.make_type_section())
            // Import section
            .section(
                &*self
                    .import_section
                    .clone()
                    .into_iter()
                    .fold(&mut ImportSection::new(), |section, import| {
                        section.import(&import.major, &import.minor, import.entity)
                    })
                    .import(
                        "sys",
                        "memory",
                        EntityType::Memory(MemoryType {
                            minimum: 1,
                            maximum: None,
                            memory64: false,
                            shared: false,
                            page_size_log2: None,
                        }),
                    ),
            )
            // Function section
            .section(
                &*self
                    .function_section
                    .into_iter()
                    .fold(&mut FunctionSection::new(), |f, t| f.function(t)),
            )
            // Table section
            .section(TableSection::new().table(TableType {
                element_type: RefType::FUNCREF,
                table64: false,
                minimum: (no_funcs + no_imports) as u64,
                maximum: Some((no_funcs + no_imports) as u64),
                shared: false,
            }))
            // Global section
            .section(&*self.global_section.into_iter().fold(
                &mut GlobalSection::new(),
                |section, type_| {
                    section.global(
                        GlobalType {
                            val_type: ValType::Ref(RefType {
                                nullable: true,
                                heap_type: HeapType::Concrete(type_),
                            }),
                            mutable: true,
                            shared: false,
                        },
                        &ConstExpr::ref_null(HeapType::Concrete(type_)),
                    )
                },
            ))
            // Export section
            .section(
                &*self
                    .export_section
                    .into_iter()
                    .fold(&mut ExportSection::new(), |section, ex| {
                        section.export(&ex.name, ex.kind, ex.index)
                    }),
            )
            // Start section
            .section(&StartSection {
                function_index: (main_func + no_imports),
            })
            // Elements
            .section(&elements_section)
            // Code section
            .section(
                &*self
                    .code_section
                    .into_iter()
                    .fold(&mut CodeSection::new(), |s, c| s.function(&c.encode())),
            )
            // Finalize
            .clone()
            .finish()
    }

    pub fn make_type_section(&self) -> TypeSection {
        let mut ts = TypeSection::new();
        for t in &self.type_section {
            match t {
                RegisteredType::Function(func_type) => ts.ty().func_type(func_type),
                RegisteredType::Array(storage_type) => ts.ty().array(storage_type, true),
                RegisteredType::Struct(storage_types) => {
                    ts.ty().struct_(storage_types.iter().map(|t| FieldType {
                        element_type: *t,
                        mutable: false,
                    }))
                }
            }
        }
        ts
    }
}
