use std::collections::{
    BTreeSet,
    HashMap,
};

use super::*;
use crate::ir::ImmediateValue;
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
    let mut element_section = wasm_encoder::ElementSection::new();
    let mut code_section = wasm_encoder::CodeSection::new();
    let mut producer_section = wasm_encoder::ProducersSection::new();

    producer_section.field(
        "language",
        ProducersField::new().value("Halcyon", crate::COMPILER_VERSION_STRING),
    );

    let mut global_namespace = HashMap::new();
    let mut func_namespace: HashMap<&Path, u32> = HashMap::new();
    let mut referenced_funcs: BTreeSet<u32> = BTreeSet::new();

    name_section.module(&asm_module.name);

    // Encode function imports (these occupy function indices 0..N)
    for (idx, (path, fi)) in asm_module.function_imports.iter().enumerate() {
        let type_idx = type_section.new_function(&fi.params, &fi.results);
        import_section.import(&fi.module, &fi.name, EntityType::Function(type_idx));
        func_namespace.insert(path, idx as u32);
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

    // Build function namespace for local functions (indices N..)
    for (idx, (path, _)) in asm_module.functions.iter().enumerate() {
        func_namespace.insert(path, idx as u32 + num_func_imports);
    }

    // Resolve start function
    let start_section = wasm_encoder::StartSection {
        function_index: func_namespace[&asm_module.start],
    };

    for (_, f) in &asm_module.functions {
        let parameter_types = f.parameters.values().cloned().collect::<Vec<_>>();
        let type_id = type_section.new_function(&parameter_types, &f.returns);
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
        for op in &f.ops {
            use Instruction as i;
            match op {
                i::Set(path) => {
                    if let Some(&idx) = local_namespace.get(path) {
                        function_body.instruction(&winstr::LocalSet(idx as u32));
                    } else if let Some(&idx) = global_namespace.get(&path) {
                        function_body.instruction(&winstr::GlobalSet(idx));
                    } else {
                        function_body.instruction(&winstr::Unreachable);
                    }
                }
                i::Get(path) => {
                    if let Some(&idx) = local_namespace.get(path) {
                        function_body.instruction(&winstr::LocalGet(idx as u32));
                    } else if let Some(&idx) = global_namespace.get(&path) {
                        function_body.instruction(&winstr::GlobalGet(idx));
                    } else {
                        function_body.instruction(&winstr::Unreachable);
                    }
                }
                i::Const(const_value) => {
                    let instr = match const_value {
                        ImmediateValue::Unit => winstr::Nop,
                        ImmediateValue::Integer(i) => winstr::I64Const(*i),
                        ImmediateValue::Real(f) => winstr::F64Const((*f).into()),
                        ImmediateValue::Boolean(b) => winstr::I32Const(if *b { 1 } else { 0 }),
                        ImmediateValue::String(_) => {
                            unreachable!("String constants not yet supported")
                        }
                        ImmediateValue::Glyph(c) => winstr::I32Const(*c as i32),
                    };
                    function_body.instruction(&instr);
                }
                i::I32Const(i) => {
                    function_body.instruction(&winstr::I32Const(*i));
                }
                i::F32Const(f) => {
                    function_body.instruction(&winstr::F32Const((*f).into()));
                }
                i::Func(path) => {
                    if let Some(&idx) = func_namespace.get(path) {
                        referenced_funcs.insert(idx);
                        function_body.instruction(&winstr::RefFunc(idx));
                    } else {
                        function_body.instruction(&winstr::Unreachable);
                    }
                }
                i::StructNew(items) => {
                    function_body.instruction(&winstr::StructNew(type_section.new_struct(items)));
                }
                i::StructGet(t, field_index) => {
                    let struct_type_index = type_section.new_struct(t);
                    function_body.instruction(&winstr::RefCastNullable(HeapType::Concrete(
                        struct_type_index,
                    )));
                    function_body.instruction(&winstr::StructGet {
                        struct_type_index,
                        field_index: *field_index as u32,
                    });
                }
                i::ArrayGet(t) => {
                    let arr_idx = type_section.new_array(t);
                    let instr = match t {
                        Type::I8 | Type::I16 => winstr::ArrayGetU(arr_idx),
                        _ => winstr::ArrayGet(arr_idx),
                    };
                    function_body.instruction(&instr);
                }
                i::ArrayNewFixed { inner_type, length } => {
                    function_body.instruction(&winstr::ArrayNewFixed {
                        array_type_index: type_section.new_array(inner_type),
                        array_size: *length as u32,
                    });
                }
                i::ArrayNewDefault(t) => {
                    function_body.instruction(&winstr::ArrayNewDefault(type_section.new_array(t)));
                }
                i::ArrayLen => {
                    function_body.instruction(&winstr::ArrayLen);
                }
                i::ArrayCopy { dst_type, src_type } => {
                    function_body.instruction(&winstr::ArrayCopy {
                        array_type_index_dst: type_section.new_array(dst_type),
                        array_type_index_src: type_section.new_array(src_type),
                    });
                }
                i::CallRef {
                    parameters,
                    returns,
                } => {
                    function_body.instruction(&winstr::CallRef(
                        type_section.new_function(parameters, returns),
                    ));
                }
                i::Unreachable => {
                    function_body.instruction(&winstr::Unreachable);
                }
                i::Drop => {
                    function_body.instruction(&winstr::Drop);
                }
                i::If(result) => {
                    function_body.instruction(&winstr::If(match result {
                        Some(r) => BlockType::Result(type_section.valtype_of(r)),
                        None => BlockType::Empty,
                    }));
                }
                i::Else => {
                    function_body.instruction(&winstr::Else);
                }
                i::End => {
                    function_body.instruction(&winstr::End);
                }
                i::Loop => {
                    function_body.instruction(&winstr::Loop(BlockType::Empty));
                }
                i::Block(result) => {
                    function_body.instruction(&winstr::Block(match result {
                        Some(r) => BlockType::Result(type_section.valtype_of(r)),
                        None => BlockType::Empty,
                    }));
                }
                i::Break(target) => {
                    function_body.instruction(&winstr::Br(*target as u32));
                }
                i::BreakIf(target) => {
                    function_body.instruction(&winstr::BrIf(*target as u32));
                }
                i::I32Op(op) => {
                    function_body.instruction(&lower_i32_op(*op));
                }
                i::I64Op(op) => {
                    function_body.instruction(&lower_i64_op(*op));
                }
                i::F32Op(op) => {
                    function_body.instruction(&lower_f32_op(*op));
                }
                i::F64Op(op) => {
                    function_body.instruction(&lower_f64_op(*op));
                }
                i::RefCastFunc {
                    parameters,
                    returns,
                } => {
                    let func_type_idx = type_section.new_function(parameters, returns);
                    function_body
                        .instruction(&winstr::RefCastNullable(HeapType::Concrete(func_type_idx)));
                }
                i::RefCastStruct(fields) => {
                    let struct_type_idx = type_section.new_struct(fields);
                    function_body.instruction(&winstr::RefCastNullable(HeapType::Concrete(
                        struct_type_idx,
                    )));
                }
                i::RefCastArray(inner) => {
                    let array_type_idx = type_section.new_array(inner);
                    function_body
                        .instruction(&winstr::RefCastNullable(HeapType::Concrete(array_type_idx)));
                }
                i::I32Store8 => {
                    function_body.instruction(&winstr::I32Store8(wasm_encoder::MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }));
                }
                i::I32Load => {
                    function_body.instruction(&winstr::I32Load(wasm_encoder::MemArg {
                        offset: 0,
                        align: 2,
                        memory_index: 0,
                    }));
                }
                i::I32Store => {
                    function_body.instruction(&winstr::I32Store(wasm_encoder::MemArg {
                        offset: 0,
                        align: 2,
                        memory_index: 0,
                    }));
                }
                i::I64Load => {
                    function_body.instruction(&winstr::I64Load(wasm_encoder::MemArg {
                        offset: 0,
                        align: 3,
                        memory_index: 0,
                    }));
                }
                i::I64ExtendI32U => {
                    function_body.instruction(&winstr::I64ExtendI32U);
                }
                i::Call(path) => {
                    if let Some(&idx) = func_namespace.get(path) {
                        function_body.instruction(&winstr::Call(idx));
                    } else {
                        function_body.instruction(&winstr::Unreachable);
                    }
                }
            }
        }
        function_body.instruction(&winstr::End);
        code_section.function(&function_body);
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

    let mut func_names = NameMap::new();
    for (path, &idx) in &func_namespace {
        func_names.append(idx, &format!("{path}"));
    }
    name_section.functions(&func_names);
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

fn lower_i32_op(op: NumberOperation) -> winstr<'static> {
    match op {
        NumberOperation::Eq => winstr::I32Eq,
        NumberOperation::Ne => winstr::I32Ne,
        NumberOperation::Gt => winstr::I32GtS,
        NumberOperation::Lt => winstr::I32LtS,
        NumberOperation::Ge => winstr::I32GeS,
        NumberOperation::Le => winstr::I32LeS,
        NumberOperation::Add => winstr::I32Add,
        NumberOperation::Sub => winstr::I32Sub,
        NumberOperation::Mul => winstr::I32Mul,
        NumberOperation::Div => winstr::I32DivS,
        NumberOperation::Rem => winstr::I32RemS,
        NumberOperation::And => winstr::I32And,
        NumberOperation::Or => winstr::I32Or,
        NumberOperation::Xor => winstr::I32Xor,
    }
}

fn lower_i64_op(op: NumberOperation) -> winstr<'static> {
    match op {
        NumberOperation::Eq => winstr::I64Eq,
        NumberOperation::Ne => winstr::I64Ne,
        NumberOperation::Gt => winstr::I64GtS,
        NumberOperation::Lt => winstr::I64LtS,
        NumberOperation::Ge => winstr::I64GeS,
        NumberOperation::Le => winstr::I64LeS,
        NumberOperation::Add => winstr::I64Add,
        NumberOperation::Sub => winstr::I64Sub,
        NumberOperation::Mul => winstr::I64Mul,
        NumberOperation::Div => winstr::I64DivS,
        NumberOperation::Rem => winstr::I64RemS,
        NumberOperation::And => winstr::I64And,
        NumberOperation::Or => winstr::I64Or,
        NumberOperation::Xor => winstr::I64Xor,
    }
}

fn lower_f32_op(op: NumberOperation) -> winstr<'static> {
    match op {
        NumberOperation::Eq => winstr::F32Eq,
        NumberOperation::Ne => winstr::F32Ne,
        NumberOperation::Gt => winstr::F32Gt,
        NumberOperation::Lt => winstr::F32Lt,
        NumberOperation::Ge => winstr::F32Ge,
        NumberOperation::Le => winstr::F32Le,
        NumberOperation::Add => winstr::F32Add,
        NumberOperation::Sub => winstr::F32Sub,
        NumberOperation::Mul => winstr::F32Mul,
        NumberOperation::Div => winstr::F32Div,
        NumberOperation::And
        | NumberOperation::Or
        | NumberOperation::Xor
        | NumberOperation::Rem => {
            unreachable!("Bitwise operations not supported on F32")
        }
    }
}

fn lower_f64_op(op: NumberOperation) -> winstr<'static> {
    match op {
        NumberOperation::Eq => winstr::F64Eq,
        NumberOperation::Ne => winstr::F64Ne,
        NumberOperation::Gt => winstr::F64Gt,
        NumberOperation::Lt => winstr::F64Lt,
        NumberOperation::Ge => winstr::F64Ge,
        NumberOperation::Le => winstr::F64Le,
        NumberOperation::Add => winstr::F64Add,
        NumberOperation::Sub => winstr::F64Sub,
        NumberOperation::Mul => winstr::F64Mul,
        NumberOperation::Div => winstr::F64Div,
        NumberOperation::And
        | NumberOperation::Or
        | NumberOperation::Xor
        | NumberOperation::Rem => {
            unreachable!("Bitwise operations not supported on F64")
        }
    }
}
