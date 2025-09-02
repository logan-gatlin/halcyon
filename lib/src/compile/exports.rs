use wasm_encoder::{ConstExpr, GlobalSection, GlobalType};

use super::*;

#[derive(Debug, Clone, Default)]
pub struct ExportEncoder {
    global_section: Vec<u32>,
    global_map: HashMap<Path, u32>,
}

impl ExportEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_global(&mut self, path: Path, type_id: u32) {
        let id = self.global_section.len() as u32;
        self.global_section.push(type_id);
        self.global_map.insert(path, id);
    }

    pub fn get_global_id(&self, path: &Path) -> u32 {
        self.global_map
            .get(path)
            .unwrap_or_else(|| panic!("Global value not found: {path}"))
            .clone()
    }

    pub fn finish(self) -> GlobalSection {
        self.global_section
            .into_iter()
            .fold(GlobalSection::new(), |mut gs, g| {
                gs.global(
                    GlobalType {
                        val_type: ValType::Ref(RefType {
                            nullable: true,
                            heap_type: HeapType::Concrete(g),
                        }),
                        mutable: true,
                        shared: false,
                    },
                    &ConstExpr::ref_null(HeapType::Concrete(g)),
                );
                gs
            })
    }
}
