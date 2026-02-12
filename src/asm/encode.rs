use std::collections::{
    BTreeSet,
    HashMap,
};

use super::*;
use wasm_encoder::{
    self,
    ArrayType,
    BlockType,
    ConstExpr,
    EntityType,
    ExportKind,
    FieldType,
    FuncType,
    GlobalType,
    HeapType,
    ImportSection,
    Instruction as winstr,
    NameMap,
    NameSection,
    ProducersField,
    RefType,
    StorageType,
    ValType,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ConcreteType {
    Function(FuncType),
    Array(ArrayType),
    StructType(Box<[FieldType]>),
}

#[derive(Debug, Clone, Default)]
struct TypeSection {
    type_section: Vec<ConcreteType>,
    cache: HashMap<ConcreteType, u32>,
}

impl TypeSection {
    fn new() -> Self {
        Self::default()
    }

    fn get_or_insert(
        &mut self,
        ct: ConcreteType,
    ) -> u32 {
        if let Some(index) = self.cache.get(&ct) {
            *index
        } else {
            self.type_section.push(ct.clone());
            let index = (self.type_section.len() - 1) as u32;
            self.cache.insert(ct, index);
            index
        }
    }

    fn new_struct(
        &mut self,
        fields: &[Type],
    ) -> u32 {
        let fields = fields
            .iter()
            .map(|f| {
                FieldType {
                    element_type: StorageType::Val(self.valtype_of(f)),
                    mutable: true,
                }
            })
            .collect();
        self.get_or_insert(ConcreteType::StructType(fields))
    }

    fn new_array(
        &mut self,
        inner: &Type,
    ) -> u32 {
        let ct = ConcreteType::Array(ArrayType(FieldType {
            element_type: self.storagetype_of(inner),
            mutable: true,
        }));
        self.get_or_insert(ct)
    }

    fn new_function(
        &mut self,
        parameters: &[Type],
        returns: &[Type],
    ) -> u32 {
        let ct = ConcreteType::Function(FuncType::new(
            parameters
                .iter()
                .map(|p| self.valtype_of(p))
                .collect::<Box<_>>(),
            returns
                .iter()
                .map(|p| self.valtype_of(p))
                .collect::<Box<_>>(),
        ));
        self.get_or_insert(ct)
    }

    fn new_function_raw(
        &mut self,
        parameters: &[ValType],
        returns: &[ValType],
    ) -> u32 {
        let ct = ConcreteType::Function(FuncType::new(
            parameters.iter().copied().collect::<Box<_>>(),
            returns.iter().copied().collect::<Box<_>>(),
        ));
        self.get_or_insert(ct)
    }

    fn storagetype_of(
        &mut self,
        type_: &Type,
    ) -> StorageType {
        match type_ {
            Type::I8 => StorageType::I8,
            Type::I16 => StorageType::I16,
            _ => StorageType::Val(self.valtype_of(type_)),
        }
    }

    fn valtype_of(
        &mut self,
        type_: &Type,
    ) -> ValType {
        match type_ {
            Type::Any => ValType::Ref(RefType::ANYREF),
            // Small storage-only types are upcast
            Type::I8 | Type::I16 => ValType::I32,
            Type::I32 => ValType::I32,
            Type::I64 => ValType::I64,
            Type::F32 => ValType::F32,
            Type::F64 => ValType::F64,
            Type::Struct(items) => {
                ValType::Ref(RefType {
                    nullable: true,
                    heap_type: HeapType::Concrete(self.new_struct(items)),
                })
            }
            Type::Array(inner) => {
                ValType::Ref(RefType {
                    nullable: true,
                    heap_type: HeapType::Concrete(self.new_array(inner)),
                })
            }
            Type::Function {
                parameters,
                results,
            } => {
                ValType::Ref(RefType {
                    nullable: true,
                    heap_type: HeapType::Concrete(self.new_function(parameters, results)),
                })
            }
        }
    }
}

impl wasm_encoder::Encode for TypeSection {
    fn encode(
        &self,
        sink: &mut Vec<u8>,
    ) {
        let mut ts = wasm_encoder::TypeSection::new();
        for t in &self.type_section {
            match t {
                ConcreteType::Function(func_type) => ts.ty().func_type(func_type),
                ConcreteType::Array(ArrayType(FieldType {
                    element_type,
                    mutable,
                })) => ts.ty().array(element_type, *mutable),
                ConcreteType::StructType(field_types) => ts.ty().struct_(field_types.clone()),
            }
        }
        ts.encode(sink);
    }
}

impl wasm_encoder::Section for TypeSection {
    fn id(&self) -> u8 {
        1
    }
}

fn default_value(valtype: &ValType) -> ConstExpr {
    match valtype {
        ValType::I32 => ConstExpr::i32_const(0),
        ValType::I64 => ConstExpr::i64_const(0),
        ValType::F32 => ConstExpr::f32_const(0.0.into()),
        ValType::F64 => ConstExpr::f64_const(0.0.into()),
        ValType::V128 => ConstExpr::v128_const(0),
        ValType::Ref(RefType { heap_type, .. }) => ConstExpr::ref_null(*heap_type),
    }
}

// WebAssembly section order (enforced):
// 0  Custom (can appear anywhere)
// 1  Type
// 2  Import
// 3  Function
// 4  Table
// 5  Memory
// 6  Global
// 7  Export
// 8  Start
// 9  Element
// 10 Code
// 11 Data
// 12 DataCount
pub fn encode(asm_module: Module) -> Vec<u8> {
    let mut name_section = NameSection::new();
    let mut global_names = NameMap::new();
    let mut type_section = TypeSection::new();
    let mut import_section = ImportSection::new();
    let mut function_section = wasm_encoder::FunctionSection::new();
    let mut table_section = wasm_encoder::TableSection::new();
    let mut memory_section = wasm_encoder::MemorySection::new();
    let mut global_section = wasm_encoder::GlobalSection::new();
    let mut export_section = wasm_encoder::ExportSection::new();
    let num_func_imports = asm_module.function_imports.len() as u32;
    let start_section = wasm_encoder::StartSection {
        function_index: asm_module.start + num_func_imports,
    };
    let mut element_section = wasm_encoder::ElementSection::new();
    let mut code_section = wasm_encoder::CodeSection::new();
    let mut producer_section = wasm_encoder::ProducersSection::new();

    producer_section.field(
        "language",
        ProducersField::new().value("Halcyon", crate::COMPILER_VERSION_STRING),
    );

    let mut global_namespace = HashMap::new();
    let mut referenced_funcs: BTreeSet<u32> = BTreeSet::new();

    name_section.module(&asm_module.name);

    // Encode function imports (these occupy function indices 0..N)
    for fi in &asm_module.function_imports {
        let type_idx = type_section.new_function_raw(&fi.params, &fi.results);
        import_section.import(&fi.module, &fi.name, EntityType::Function(type_idx));
    }

    let mut global_id = 0;
    for (name, type_) in asm_module.imports.iter() {
        import_section.import(
            &name.major,
            &name.minor,
            EntityType::Global(GlobalType {
                val_type: type_section.valtype_of(type_),
                mutable: true,
                shared: false,
            }),
        );
        global_namespace.insert(name, global_id);
        global_names.append(global_id, &format!("{name}"));
        global_id += 1;
    }

    for (name, type_) in asm_module.globals.iter() {
        let val_type = type_section.valtype_of(type_);
        global_section.global(
            GlobalType {
                val_type,
                mutable: true,
                shared: false,
            },
            &default_value(&val_type),
        );
        export_section.export(&name.minor, ExportKind::Global, global_id);
        global_namespace.insert(name, global_id);
        global_names.append(global_id, &name.minor);
        global_id += 1;
    }

    // Add linear memory if required (e.g. for WASI)
    if asm_module.has_memory {
        memory_section.memory(wasm_encoder::MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
        export_section.export("memory", ExportKind::Memory, 0);
    }

    for f in &asm_module.functions {
        let type_id = type_section.new_function(
            f.parameters
                .values()
                .cloned()
                .collect::<Vec<_>>()
                .as_slice(),
            &f.returns,
        );
        function_section.function(type_id);

        let local_namespace = f
            .parameters
            .iter()
            .chain(f.variables.iter())
            .enumerate()
            .map(|(id, (name, _))| (name.clone(), id))
            .collect::<HashMap<_, _>>();

        let mut function_body = wasm_encoder::Function::new_with_locals_types(
            f.variables.iter().map(|(_, t)| type_section.valtype_of(t)),
        );
        // Instruction lowering
        code_section.function(
            f.ops
                .iter()
                .map(|o| {
                    use {
                        Instruction as i,
                        NumberOperation as n,
                    };
                    match o {
                        i::Set(path) => {
                            if let Some(&idx) = local_namespace.get(path) {
                                winstr::LocalSet(idx as u32)
                            } else if let Some(&idx) = global_namespace.get(&path) {
                                winstr::GlobalSet(idx)
                            } else {
                                unreachable!("Unknown variable: {}", &path)
                            }
                        }
                        i::Get(path) => {
                            if let Some(&idx) = local_namespace.get(path) {
                                winstr::LocalGet(idx as u32)
                            } else if let Some(&idx) = global_namespace.get(&path) {
                                winstr::GlobalGet(idx)
                            } else {
                                unreachable!("Unknown variable: {path}")
                            }
                        }
                        i::Const(const_value) => {
                            match const_value {
                                ConstValue::Unit => winstr::Nop,
                                ConstValue::Integer(i) => winstr::I64Const(*i),
                                ConstValue::Real(f) => winstr::F64Const((*f).into()),
                                ConstValue::Boolean(b) => winstr::I32Const(if *b { 1 } else { 0 }),
                                ConstValue::String(_) => {
                                    unreachable!("String constants not yet supported")
                                }
                                ConstValue::Glyph(c) => winstr::I32Const(*c as i32),
                            }
                        }
                        i::I32Const(i) => winstr::I32Const(*i),
                        i::Func(id) => {
                            let adjusted = *id as u32 + num_func_imports;
                            referenced_funcs.insert(adjusted);
                            winstr::RefFunc(adjusted)
                        }
                        i::StructNew(items) => winstr::StructNew(type_section.new_struct(items)),
                        i::StructGet(t, field_index) => {
                            winstr::StructGet {
                                struct_type_index: type_section.new_struct(t),
                                field_index: *field_index as u32,
                            }
                        }
                        i::ArrayGet(t) => {
                            let arr_idx = type_section.new_array(t);
                            match t {
                                Type::I8 | Type::I16 => winstr::ArrayGetU(arr_idx),
                                _ => winstr::ArrayGet(arr_idx),
                            }
                        }
                        i::ArrayNewFixed { inner_type, length } => {
                            winstr::ArrayNewFixed {
                                array_type_index: type_section.new_array(inner_type),
                                array_size: *length as u32,
                            }
                        }
                        i::ArrayNewDefault(t) => winstr::ArrayNewDefault(type_section.new_array(t)),
                        i::ArrayLen => winstr::ArrayLen,
                        i::ArrayCopy { dst_type, src_type } => {
                            winstr::ArrayCopy {
                                array_type_index_dst: type_section.new_array(dst_type),
                                array_type_index_src: type_section.new_array(src_type),
                            }
                        }
                        i::CallRef {
                            parameters,
                            returns,
                        } => winstr::CallRef(type_section.new_function(parameters, returns)),
                        i::Unreachable => winstr::Unreachable,
                        i::Drop => winstr::Drop,
                        i::If(result) => {
                            winstr::If(match result {
                                Some(r) => BlockType::Result(type_section.valtype_of(r)),
                                None => BlockType::Empty,
                            })
                        }
                        i::Else => winstr::Else,
                        i::End => winstr::End,
                        i::Loop => winstr::Loop(BlockType::Empty),
                        i::Block(result) => {
                            winstr::Block(match result {
                                Some(r) => BlockType::Result(type_section.valtype_of(r)),
                                None => BlockType::Empty,
                            })
                        }
                        i::Break(target) => winstr::Br(*target as u32),
                        i::BreakIf(target) => winstr::BrIf(*target as u32),
                        i::I32Op(op) => {
                            match op {
                                n::Eq => winstr::I32Eq,
                                n::Ne => winstr::I32Ne,
                                n::Gt => winstr::I32GtS,
                                n::Lt => winstr::I32LtS,
                                n::Ge => winstr::I32GeS,
                                n::Le => winstr::I32LeS,
                                n::Add => winstr::I32Add,
                                n::Sub => winstr::I32Sub,
                                n::Mul => winstr::I32Mul,
                                n::Div => winstr::I32DivS,
                                n::And => winstr::I32And,
                                n::Or => winstr::I32Or,
                                n::Xor => winstr::I32Xor,
                            }
                        }
                        i::I64Op(op) => {
                            match op {
                                n::Eq => winstr::I64Eq,
                                n::Ne => winstr::I64Ne,
                                n::Gt => winstr::I64GtS,
                                n::Lt => winstr::I64LtS,
                                n::Ge => winstr::I64GeS,
                                n::Le => winstr::I64LeS,
                                n::Add => winstr::I64Add,
                                n::Sub => winstr::I64Sub,
                                n::Mul => winstr::I64Mul,
                                n::Div => winstr::I64DivS,
                                n::And => winstr::I64And,
                                n::Or => winstr::I64Or,
                                n::Xor => winstr::I64Xor,
                            }
                        }
                        i::F32Op(op) => {
                            match op {
                                n::Eq => winstr::F32Eq,
                                n::Ne => winstr::F32Ne,
                                n::Gt => winstr::F32Gt,
                                n::Lt => winstr::F32Lt,
                                n::Ge => winstr::F32Ge,
                                n::Le => winstr::F32Le,
                                n::Add => winstr::F32Add,
                                n::Sub => winstr::F32Sub,
                                n::Mul => winstr::F32Mul,
                                n::Div => winstr::F32Div,
                                n::And | n::Or | n::Xor => {
                                    unreachable!("Bitwise operations not supported on F32")
                                }
                            }
                        }
                        i::F64Op(op) => {
                            match op {
                                n::Eq => winstr::F64Eq,
                                n::Ne => winstr::F64Ne,
                                n::Gt => winstr::F64Gt,
                                n::Lt => winstr::F64Lt,
                                n::Ge => winstr::F64Ge,
                                n::Le => winstr::F64Le,
                                n::Add => winstr::F64Add,
                                n::Sub => winstr::F64Sub,
                                n::Mul => winstr::F64Mul,
                                n::Div => winstr::F64Div,
                                n::And | n::Or | n::Xor => {
                                    unreachable!("Bitwise operations not supported on F64")
                                }
                            }
                        }
                        i::RefCastFunc {
                            parameters,
                            returns,
                        } => {
                            let func_type_idx = type_section.new_function(parameters, returns);
                            winstr::RefCastNullable(HeapType::Concrete(func_type_idx))
                        }
                        i::RefCastStruct(fields) => {
                            let struct_type_idx = type_section.new_struct(fields);
                            winstr::RefCastNullable(HeapType::Concrete(struct_type_idx))
                        }
                        i::RefCastArray(inner) => {
                            let array_type_idx = type_section.new_array(inner);
                            winstr::RefCastNullable(HeapType::Concrete(array_type_idx))
                        }
                        i::I32Store8 => {
                            winstr::I32Store8(wasm_encoder::MemArg {
                                offset: 0,
                                align: 0,
                                memory_index: 0,
                            })
                        }
                        i::I32Store => {
                            winstr::I32Store(wasm_encoder::MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            })
                        }
                        i::Call(idx) => winstr::Call(*idx as u32),
                    }
                })
                .fold::<&mut _, _>(&mut function_body, |body, i| body.instruction(&i))
                .instruction(&winstr::End),
        );
    }

    // Declare all referenced functions in the element section
    let referenced_funcs: Vec<u32> = referenced_funcs.into_iter().collect();
    table_section.table(wasm_encoder::TableType {
        element_type: RefType::FUNCREF,
        table64: false,
        minimum: referenced_funcs.len() as u64,
        maximum: Some(referenced_funcs.len() as u64),
        shared: false,
    });
    element_section.declared(wasm_encoder::Elements::Functions(referenced_funcs.into()));

    name_section.globals(&global_names);

    let mut module = wasm_encoder::Module::new();
    module
        .section(&name_section)
        .section(&type_section)
        .section(&import_section)
        .section(&function_section)
        .section(&table_section);
    if asm_module.has_memory {
        module.section(&memory_section);
    }
    module
        .section(&global_section)
        .section(&export_section)
        .section(&start_section)
        .section(&element_section)
        .section(&code_section)
        .section(&asm_module.sig);
    module.finish()
}
