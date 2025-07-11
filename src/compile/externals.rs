use super::*;

pub fn get_exports(module: &HlIrModule) -> ExportSection {
  module
    .nodes
    .iter()
    .flat_map(|n| match &n.kind {
      HlIrKind::FunctionDef {
        export_name, id, ..
      } => {
        if let Some(export) = &export_name {
          Some((export, *id))
        } else {
          None
        }
      }
      _ => None,
    })
    .fold(&mut ExportSection::new(), |s, e| {
      s.export(e.0, ExportKind::Func, e.1)
    })
    .clone()
}
