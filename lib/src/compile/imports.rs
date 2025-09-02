use wasm_encoder::{EntityType, ImportSection, MemoryType};

use crate::lint::*;

use super::*;

#[derive(Debug, Clone)]
struct ImportSymbol {
    major: String,
    minor: String,
    entity: EntityType,
}

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, sx::SXRepr)]
pub struct ForeignFunctionType {
    /// Number of i64 parameters
    pub parameters: usize,
    /// Number of i64 returns
    pub returns: usize,
}

impl ForeignFunctionType {
    pub fn new(parameters: usize, returns: usize) -> Self {
        Self {
            parameters,
            returns,
        }
    }
}

impl Into<FuncType> for ForeignFunctionType {
    fn into(self) -> FuncType {
        FuncType::new(
            vec![ValType::I64; self.parameters],
            vec![ValType::I64; self.returns],
        )
    }
}

impl Into<Type> for ForeignFunctionType {
    fn into(self) -> Type {
        Type::func(
            match self.parameters {
                0 => Type::Unit,
                1 => Type::Integer,
                n => Type::Product(vec![Type::Integer; n]),
            },
            match self.returns {
                0 => Type::Unit,
                1 => Type::Integer,
                n => Type::Product(vec![Type::Integer; n]),
            },
        )
    }
}

impl TryFrom<Type> for ForeignFunctionType {
    type Error = Lint;

    fn try_from(value: Type) -> Result<Self> {
        let Type::Function(a, b) = value else {
            return Err(lint_nospan(TypeLint::InvalidImportType));
        };
        fn validate(t: Type) -> Result<usize> {
            match t {
                Type::Unit => Ok(0),
                Type::Integer => Ok(1),
                Type::Product(v) if v.iter().all(|t| t == &Type::Integer) => Ok(v.len()),
                _ => Err(lint_nospan(TypeLint::InvalidImportType)),
            }
        }
        Ok(ForeignFunctionType {
            parameters: validate(*a)?,
            returns: validate(*b)?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct ImportEncoder(Vec<ImportSymbol>);

impl ImportEncoder {
    fn push(&mut self, major: String, minor: String, entity: EntityType) -> u32 {
        let id = self.0.len() as u32;
        self.0.push(ImportSymbol {
            major,
            minor,
            entity,
        });
        id
    }

    pub fn functions(&self) -> u32 {
        self.0.iter().fold(0, |acc, ImportSymbol { entity, .. }| {
            if let EntityType::Function(..) = entity {
                acc + 1
            } else {
                acc
            }
        })
    }

    pub fn finish(self) -> ImportSection {
        let mut s = self
            .0
            .into_iter()
            .fold(ImportSection::new(), |mut section, import| {
                section.import(&import.major, &import.minor, import.entity);
                section
            });
        s.import(
            "sys",
            "memory",
            EntityType::Memory(MemoryType {
                minimum: 1,
                maximum: None,
                memory64: false,
                shared: false,
                page_size_log2: None,
            }),
        );
        s
    }
}

impl ModuleEncoder {
    pub fn new_import_function(
        &mut self,
        major: impl Into<String>,
        minor: impl Into<String>,
        i64_parameters: usize,
        i64_returns: usize,
    ) -> u32 {
        let foreign_type = ForeignFunctionType::new(i64_parameters, i64_returns);
        self.type_encoder.encode(foreign_type);
        let type_id = self.foreign_function_type(foreign_type);
        let import_id = self.import_encoder.functions();
        self.import_encoder
            .push(major.into(), minor.into(), EntityType::Function(type_id));
        let element_id = self.element_section.len() as u32;
        self.element_section.push(FunctionKind::Import(import_id));
        element_id
    }
}
