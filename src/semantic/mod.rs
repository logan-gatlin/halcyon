pub mod analyzer;
pub mod primitives;

pub use analyzer::*;
pub use primitives::*;

use std::collections::HashMap;

use crate::{Parameters, Span, err::*, semantic::Primitive};

pub type UID = String;

#[derive(Debug, Clone)]
pub enum Type {
  Ambiguous,
  Prim(Primitive),
  Nothing,
  Struct(UID),
  Function(UID),
}

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::Ambiguous => write!(f, "ambiguous"),
      Type::Prim(primitive) => write!(f, "{primitive}"),
      Type::Nothing => write!(f, "nothing"),
      Type::Struct(s) => write!(f, "struct {s}"),
      Type::Function(func) => write!(f, "func {func}"),
    }
  }
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
  Variable {
    type_: UID,
    mutable: bool,
    global: bool,
    children: Vec<UID>,
  },
  Function {
    params: Parameters,
    returns: UID,
  },
  Struct {
    params: Parameters,
    size: usize,
    align: usize,
  },
  TypeDef {
    actual: Type,
  },
}

#[derive(Debug, Clone)]
pub struct Symbol {
  pub name: String,
  pub uid: UID,
  pub span: Option<Span>,
  pub kind: SymbolKind,
}

impl Symbol {
  pub fn is_variable(&self) -> Result<()> {
    let name = &self.name;
    match &self.kind {
      SymbolKind::Variable { .. } => Ok(()),
      SymbolKind::Function { .. } => {
        error().reason(format!("'{name}' refers to a function, not a value"))
      },
      _ => error().reason(format!("'{name}' refers to a type, not a value")),
    }
  }

  pub fn is_type(&self) -> Result<Type> {
    let name = &self.name;
    match &self.kind {
      SymbolKind::Struct { .. } => Ok(Type::Struct(self.uid.clone())),
      SymbolKind::TypeDef { actual } => Ok(actual.clone()),
      _ => error().reason(format!("'{name}' refers to a value, not a type")),
    }
  }
}

#[derive(Debug, Clone)]
pub enum Event {
  Declared { name: String, uid: UID },
  Moved { name: String, uid: UID, span: Span },
  Func { returns: Vec<UID>, uid: UID },
  Block { returns: UID },
}

pub struct SymbolTable {
  syms: HashMap<UID, Symbol>,
  scope: Vec<Event>,
  nesting: usize,
  mangle_num: usize,
}

impl SymbolTable {
  pub fn new() -> Self {
    Self {
      syms: HashMap::new(),
      scope: vec![],
      nesting: 0,
      mangle_num: 0,
    }
  }

  fn generate_uid(&mut self, name: &str) -> UID {
    let uid = format!("${}${name}", self.mangle_num);
    self.mangle_num += 1;
    uid
  }

  pub fn get(&self, uid: &UID) -> &Symbol {
    self.syms.get(uid).unwrap()
  }

  pub fn get_mut(&mut self, uid: &UID) -> &mut Symbol {
    self.syms.get_mut(uid).unwrap()
  }

  // Find the definition of a symbol in local and global scope
  pub fn find(&self, name: &str) -> Result<&Symbol> {
    let mut nesting = self.nesting;
    for e in self.scope.iter().rev() {
      match e {
        Event::Declared { uid, .. }
          if nesting == self.nesting || nesting == 0 =>
        {
          return Ok(self.get(uid));
        },
        Event::Moved { name, span, .. }
          if nesting == self.nesting || nesting == 0 =>
        {
          return error()
            .reason(format!("Symbol '{name}' moved out of scope here"))
            .span(&span);
        },
        Event::Func { .. } => {
          nesting -= 1;
        },
        _ => {},
      }
    }
    error().reason(format!("Cannot find symbol '{name}'"))
  }

  // Get all nested members of a struct variable
  fn get_all_children(&self, uid: &UID) -> Vec<UID> {
    use SymbolKind as s;
    match &self.get(uid).kind {
      s::Variable { children, .. } => {
        let mut new_children = children.clone();
        for uid in children {
          new_children.append(&mut self.get_all_children(uid))
        }
        new_children
      },
      _ => {
        vec![]
      },
    }
  }

  // Move a symbol out of scope
  pub fn move_symbol(&mut self, move_uid: &UID, span: Span) -> Result<()> {
    if let SymbolKind::Variable { global, .. } = self.get(move_uid).kind {
      if global {
        return error().reason("Cannot move global symbol").span(&span);
      }
    } else {
      panic!("Moved non-variable {move_uid}");
    }
    let children = self.get_all_children(move_uid);
    for e in self.scope.iter().rev() {
      match e {
        Event::Declared { uid, .. } => {
          if move_uid == uid {
            break;
          }
        },
        Event::Moved { uid, span, .. } => {
          if children.contains(uid) {
            return error()
              .reason("Symbol was partially moved here")
              .span(&span);
          } else if move_uid == uid {
            return error()
              .reason("Symbol was previously moved here")
              .span(&span);
          }
        },
        _ => {},
      }
    }
    self.scope.push(Event::Moved {
      name: self.get(move_uid).name.clone(),
      uid: move_uid.clone(),
      span,
    });
    Ok(())
  }

  fn in_global_scope(&self) -> bool {
    for e in self.scope.iter().rev() {
      if let Event::Func { .. } = e {
        return false;
      }
    }
    true
  }

  pub fn define_var(
    &mut self,
    name: &str,
    mutable: bool,
    type_: &UID,
    span: Span,
  ) -> UID {
    let uid = self.generate_uid(name);
    self.syms.insert(uid.clone(), Symbol {
      name: name.to_string(),
      span: Some(span),
      uid: uid.clone(),
      kind: SymbolKind::Variable {
        type_: type_.clone(),
        mutable,
        global: self.in_global_scope(),
        children: vec![],
      },
    });
    self.scope.push(Event::Declared {
      name: name.to_string(),
      uid: uid.clone(),
    });
    uid
  }

  pub fn declare_struct(
    &mut self,
    struct_name: &str,
    span: Span,
  ) -> Result<UID> {
    // Check for multiple definition
    for e in self.scope.iter().rev() {
      match e {
        Event::Declared { name, uid } => {
          if name == struct_name {
            let e = error().reason(format!(
              "Structure '{struct_name}' is defined multiple times"
            ));
            if let Some(s) = &self.get(uid).span {
              return e.span(s);
            } else {
              return e;
            }
          }
        },
        Event::Moved { .. } => {},
        _ => break,
      }
    }
    let uid = self.generate_uid(struct_name);
    self.syms.insert(uid.clone(), Symbol {
      name: struct_name.to_string(),
      uid: uid.clone(),
      span: Some(span),
      kind: SymbolKind::Struct {
        params: Parameters::default(),
        size: 0,
        align: 0,
      },
    });
    self.scope.push(Event::Declared {
      name: struct_name.to_string(),
      uid: uid.clone(),
    });
    Ok(uid)
  }

  fn type_params(&mut self, mut params: Parameters) -> Result<Parameters> {
    for i in 0..params.arity {
      params.types[i] = self
        .find(&params.type_names[i])
        .trace(format!(
          "while resolving type of field '{}'",
          params.type_names[i]
        ))?
        .uid
        .clone();
    }
    Ok(params)
  }

  pub fn define_function(
    &mut self,
    func_name: &str,
    params: Parameters,
    returns_str: Option<&str>,
    span: Span,
  ) -> Result<UID> {
    // Check for multiple definition
    for e in self.scope.iter().rev() {
      match e {
        Event::Declared { name, uid } => {
          if name == func_name {
            let e = error().reason(format!(
              "Function '{func_name}' is defined multiple times"
            ));
            if let Some(s) = &self.get(uid).span {
              return e.span(s);
            } else {
              return e;
            }
          }
        },
        Event::Moved { .. } => {},
        _ => break,
      }
    }
    let uid = self.generate_uid(func_name);
    // Check types
    let params = self.type_params(params)?;
    let returns = {
      let sym = self.find(returns_str)?;
      sym.is_type()?;
      sym.uid.clone()
    };
    self.syms.insert(uid.clone(), Symbol {
      name: func_name.to_string(),
      uid: uid.clone(),
      span: Some(span),
      kind: SymbolKind::Function { params, returns },
    });
    self.scope.push(Event::Declared {
      name: func_name.to_string(),
      uid: uid.clone(),
    });
    Ok(uid)
  }

  pub fn define_struct(&mut self, uid: &UID, params: Parameters) -> Result<()> {
    let params = self.type_params(params)?;
    if let SymbolKind::Struct {
      params: old_params, ..
    } = &mut self.get_mut(uid).kind
    {
      *old_params = params;
    } else {
      unreachable!("Defined non-existent struct")
    }
    Ok(())
  }
}
