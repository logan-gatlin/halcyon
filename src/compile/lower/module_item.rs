use super::*;

impl Encode<ModuleItem> for FunctionEncoder<'_> {
    fn encode(&mut self, item: ModuleItem) -> &mut Self {
        match item {
            ModuleItem::Let(mut pattern, node) => {
                pattern.visit(|(path, type_)| {
                    self.module_encoder.new_global(path, type_);
                });
                self.encode(node.type_.clone())
                    .encode(node)
                    .encode((pattern, ScopeKind::Global))
            }
            ModuleItem::Constructor(
                path,
                Constructor {
                    variant_id,
                    kind: ConstructorKind::Unitary(t),
                },
            ) => {
                self.encode(t.clone());
                let type_id = self.module_encoder.type_id(&t);
                self.module_encoder.new_global(&path, &t);
                self.encode(I32Const(variant_id as i32))
                    .encode(ConstValue::Unit)
                    .encode(StructNew(type_id))
                    .set_symbol(&path)
            }
            ModuleItem::Constructor(
                path,
                Constructor {
                    variant_id,
                    kind: ConstructorKind::Function(in_type, out_type),
                },
            ) => {
                self.encode([
                    in_type.clone(),
                    out_type.clone(),
                    Type::func(in_type.clone(), out_type.clone()),
                ]);
                let function_type = Type::func(in_type.clone(), out_type.clone());
                let parameter_name = Path::from("_");
                let struct_type_id = self.module_encoder.reduced_type_id(&ReducedType::Sum);
                let capture_type_id = self.module_encoder.reduced_type_id(&ReducedType::capture());
                let function_id = self
                    .module_encoder
                    .new_global(&path, &function_type)
                    .function(parameter_name.clone(), &in_type)
                    .encode(I32Const(variant_id as i32))
                    .get_symbol(&parameter_name)
                    .encode(StructNew(struct_type_id))
                    .finish();
                self.encode([
                    RefFunc(function_id),
                    ArrayNewFixed {
                        array_type_index: capture_type_id,
                        array_size: 0,
                    },
                    StructNew(self.module_encoder.type_id(&function_type)),
                ])
                .set_symbol(&path)
            }
            ModuleItem::Import {
                path,
                type_,
                major,
                minor,
            } => {
                self.module_encoder.new_global(&path, &type_.into());
                self.new_import_function(major, minor, type_)
                    .set_symbol(&path)
            }
            ModuleItem::Type(_) => self,
        }
    }
}
