pub mod encoding;
mod exports;
pub mod function;
mod imports;
mod lower;
mod types;

pub use encoding::*;
use exports::*;
pub use function::*;
pub use imports::*;
pub use types::*;
use wasm_encoder::{
    CodeSection, ConstExpr, ElementMode, ElementSection, ElementSegment, Elements, FunctionSection,
    Module, TableSection, TypeSection,
};

use std::collections::HashMap;

#[allow(unused_imports)]
pub use wasm_encoder::{
    // No glob import, conflict with Encode trait
    BlockType,
    FuncType,
    Function,
    HeapType,
    Instruction::*,
    RefType,
    StorageType,
    ValType,
};

type Instruction = wasm_encoder::Instruction<'static>;

use crate::{Visit, compile::exports::ExportEncoder, ir::*, semantic::Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionKind {
    Import(u32),
    Native(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableKind {
    Global(u32),
    Local(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Global,
    Local,
}

impl VariableKind {
    pub fn set(self) -> Instruction {
        match self {
            VariableKind::Global(id) => GlobalSet(id),
            VariableKind::Local(id) => LocalSet(id),
        }
    }

    pub fn get(self) -> Instruction {
        match self {
            VariableKind::Global(id) => GlobalGet(id),
            VariableKind::Local(id) => LocalGet(id),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ModuleEncoder {
    type_encoder: TypeEncoder,
    code_section: Vec<EncodedFunction>,
    export_encoder: ExportEncoder,
    import_encoder: ImportEncoder,
    element_section: Vec<FunctionKind>,
    function_section: Vec<u32>,
    init_functions: Vec<u32>,
}

impl ModuleEncoder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn new_global(&mut self, path: &Path, type_: &Type) -> &mut Self {
        self.export_encoder
            .new_global(path.clone(), self.type_id(type_));
        self
    }

    pub fn get_global_id(&self, path: &Path) -> u32 {
        self.export_encoder.get_global_id(path)
    }

    pub fn function(
        &'_ mut self,
        parameter_name: Path,
        parameter_type: &Type,
        return_type: &Type,
    ) -> FunctionEncoder<'_> {
        FunctionEncoder::new(self, parameter_name, parameter_type, return_type)
    }

    pub fn main_function(&'_ mut self) -> FunctionEncoder<'_> {
        FunctionEncoder::new_main(self)
    }

    pub fn finish(self) -> Vec<u8> {
        let function_count = self.function_section.len() as u64;
        let imported_function_count = self.import_encoder.functions();
        let mut module = Module::new();
        module
            // Type section
            .section(&self.type_encoder.finish())
            // Import section
            .section(&self.import_encoder.finish())
            // Function section
            .section(&self.function_section.into_iter().fold(
                FunctionSection::new(),
                |mut fs, f| {
                    fs.function(f);
                    fs
                },
            ))
            // Table section
            .section(TableSection::new().table(wasm_encoder::TableType {
                element_type: RefType::FUNCREF,
                table64: false,
                minimum: function_count,
                maximum: Some(function_count),
                shared: false,
            }))
            // Global section
            .section(&self.export_encoder.finish())
            // Export section
            // Start section
            // Elements
            .section(
                &ElementSection::new()
                    .segment(ElementSegment {
                        mode: ElementMode::Active {
                            table: None,
                            offset: &ConstExpr::i32_const(0),
                        },
                        // This mapping is no longer necessary, consider removing?
                        elements: Elements::Functions(std::borrow::Cow::from(
                            &self
                                .element_section
                                .clone()
                                .into_iter()
                                .map(|e| match e {
                                    FunctionKind::Import(id) => id,
                                    FunctionKind::Native(id) => id + imported_function_count,
                                })
                                .collect::<Vec<_>>(),
                        )),
                    })
                    .clone(),
            )
            // Code section
            .section(
                &self
                    .code_section
                    .into_iter()
                    // This is annoying to have to do. The alternatives are switching to
                    // indirect calling which is probably less performant, or using another
                    // intermediary IR level, which is a big maintenance burden.
                    .fold(CodeSection::new(), |mut cs, c| {
                        cs.function(
                            &c.fix_function_ids(&self.element_section, imported_function_count)
                                .into(),
                        );
                        cs
                    }),
            );
        module.finish()
    }
}

impl Encode<EncodedFunction> for ModuleEncoder {
    fn encode(&mut self, obj: EncodedFunction) -> &mut Self {
        let id = self.code_section.len();
        self.function_section.push(obj.type_id);
        self.code_section.push(obj);
        self.element_section.push(FunctionKind::Native(id as u32));
        self
    }
}

impl Encode<IrModule> for ModuleEncoder {
    fn encode(&mut self, mut module: IrModule) -> &mut Self {
        module.visit(|t: &mut Type| {
            self.type_encoder.encode(t.clone());
        });
        let init = FunctionEncoder::new_main(self)
            .encode(module.items.as_slice())
            .finish_mainfn();
        self.init_functions.push(init);
        self
    }
}
