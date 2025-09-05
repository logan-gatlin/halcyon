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

    pub fn parameter_type(&self) -> Type {
        match self.parameters {
            0 => Type::Unit,
            1 => Type::Integer,
            n => Type::Product(vec![Type::Integer; n]),
        }
    }

    pub fn return_type(&self) -> Type {
        match self.returns {
            0 => Type::Unit,
            1 => Type::Integer,
            n => Type::Product(vec![Type::Integer; n]),
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
                Type::Product(v) if v.len() != 1 && v.iter().all(|t| t == &Type::Integer) => {
                    Ok(v.len())
                }
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
        let id = self.functions();
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

impl FunctionEncoder<'_> {
    pub fn new_import_function(
        &mut self,
        major: impl Into<String>,
        minor: impl Into<String>,
        ftype: ForeignFunctionType,
    ) -> &mut Self {
        self.encode(Type::Integer);
        self.module_encoder.type_encoder.encode(ftype);
        let import_function_id = self.module_encoder.import_encoder.push(
            major.into(),
            minor.into(),
            EntityType::Function(self.module_encoder.foreign_function_type(ftype)),
        );
        let element_id = self.module_encoder.element_section.len() as u32;
        self.module_encoder
            .element_section
            .push(FunctionKind::Import(import_function_id));
        let parameter = Path::from("a");
        let mut enc = self.module_encoder.function(
            parameter.clone(),
            &ftype.parameter_type(),
            &ftype.return_type(),
        );
        let integer_type = enc.module_encoder.type_id(&Type::Integer);
        match ftype.parameters {
            0 => {}
            1 => {
                enc.get_symbol(&parameter).encode(StructGet {
                    struct_type_index: integer_type,
                    field_index: 0,
                });
            }
            n => {
                let tuple_type = enc
                    .module_encoder
                    .type_id(&Type::Product(vec![Type::Integer; ftype.parameters]));
                for i in 0..(n as u32) {
                    enc.get_symbol(&parameter).encode([
                        StructGet {
                            struct_type_index: tuple_type,
                            field_index: i,
                        },
                        StructGet {
                            struct_type_index: integer_type,
                            field_index: 0,
                        },
                    ]);
                }
            }
        };
        enc.encode(Call(element_id));
        match ftype.returns {
            0 => {
                enc.encode(ConstValue::Unit);
            }
            1 => {
                enc.encode(StructNew(integer_type));
            }
            n => {
                let mut locals = vec![];
                for _ in 0..n {
                    let temp = enc.new_temporary(&Type::Integer);
                    locals.push(temp);
                    enc.encode([StructNew(integer_type), LocalSet(temp)]);
                }
                for l in locals {
                    enc.encode(LocalGet(l));
                }
                let tuple_type = enc
                    .module_encoder
                    .type_id(&Type::Product(vec![Type::Integer; ftype.returns]));
                enc.encode(StructNew(tuple_type));
            }
        }
        let fid = enc.finish();
        let capture_type_id = self.module_encoder.reduced_type_id(&ReducedType::capture());
        let function_type = self.module_encoder.type_id(&ftype.into());
        self.encode([
            RefFunc(fid),
            ArrayNewFixed {
                array_type_index: capture_type_id,
                array_size: 0,
            },
            StructNew(function_type),
        ]);
        self
    }
}
