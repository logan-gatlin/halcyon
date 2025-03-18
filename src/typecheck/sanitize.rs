use std::collections::HashSet;

use crate::{Span, hlir::builtins::*, hlir::*, lint::*, mlir::*};

use super::TypeChecker;
use HlIrKind::*;

impl TypeChecker {
  pub(super) fn sanitize_main(&self) -> Result<HashSet<IrPtr>> {
    let Some(main) = self.module.main.clone() else {
      return Err(lint_nospan(NameLint::InvalidMain));
    };
    let mut visited = HashSet::new();
    let mut to_visit = vec![];
    let mut current = self.module.functions.get(&main).cloned().unwrap();
    'outer: loop {
      visited.insert(current);
      self.sanitize_node(current, &mut to_visit)?;
      loop {
        if let Some(new_node) = to_visit.pop() {
          if !visited.contains(&new_node) {
            current = new_node;
            break;
          }
        } else {
          break 'outer;
        }
      }
    }
    Ok(visited)
  }

  fn sanitize_node(
    &self,
    node: IrPtr,
    to_visit: &mut Vec<IrPtr>,
  ) -> Result<()> {
    let err = Err(lint_nospan(TypeLint::Sanitization));
    let span = self.module.nodes[node].span;
    if let Immediate(ConstValue::Function(mangle)) =
      &self.module.nodes[node].kind
    {
      // Only push non-builtin functions
      if let Some(mangle) = self.module.functions.get(mangle) {
        to_visit.push(mangle.clone());
      }
      if let Some(bt) = Builtin::from_mangle(mangle)
        && !bt.sanitary()
      {
        return err.span(span);
      }
    }
    let mut sanitize = move |ptr| self.sanitize_node(ptr, to_visit);
    if self.module.type_of(node).ambiguous() {
      return err.span(span);
    }
    match &self.module.nodes[node].kind {
      Declaration {
        is_constant, value, ..
      } => {
        let type_ = self.module.type_of(*value);
        if (type_.ambiguous() || type_ == Type::Type) && !is_constant {
          return err.span(self.module.value_span(*value));
        }
        if !is_constant {
          sanitize(*value)?;
        }
      },
      Immediate(_) => {},
      Block(items) => {
        for item in items {
          sanitize(*item)?;
        }
      },
      Identifier(_) => {},
      StructDef { types, .. } => {
        for type_ in types {
          sanitize(*type_)?;
        }
      },
      StructLiteral {
        struct_t,
        field_values,
        ..
      } => {
        if let Some((struct_t, _)) = struct_t {
          sanitize(*struct_t)?;
        }
        for value in field_values {
          sanitize(*value)?;
        }
      },
      Field { of, .. } => {
        sanitize(*of)?;
      },
      Binary { left, right, .. } => {
        sanitize(*left)?;
        sanitize(*right)?;
      },
      Unary { child, .. } => {
        sanitize(*child)?;
      },
      FunctionDef { body, .. } => {
        sanitize(*body)?;
      },
      FunctionCall {
        callee, arguments, ..
      } => {
        sanitize(*callee)?;
        for arg in arguments {
          sanitize(*arg)?;
        }
      },
      If {
        predicate,
        then,
        else_,
      } => {
        sanitize(*predicate)?;
        sanitize(*then)?;
        if let Some(else_) = else_ {
          sanitize(*else_)?;
        }
      },
      Loop {
        parameter_values,
        body,
        ..
      } => {
        for val in parameter_values {
          sanitize(*val)?;
        }
        sanitize(*body)?;
      },
      Break(node) => {
        if let Some(node) = node {
          sanitize(*node)?;
        }
      },
    };
    Ok(())
  }
}
