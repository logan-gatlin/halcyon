use super::resolve::{
    ResolvedInstruction,
    ResolvedModule,
};
use super::*;

#[derive(Debug, Clone, Copy)]
enum ControlFrame {
    Block,
    Loop,
    If { seen_else: bool },
}

/// Handles verify module.
pub fn verify_module(module: &ResolvedModule) -> Result<(), BackendError> {
    for ((function_path, _), function) in
        module.lowered.functions.iter().zip(module.functions.iter())
    {
        if function.ops.len() != function.op_origins.len() {
            return Err(BackendError::in_function(
                function_path.clone(),
                None,
                None,
                format!(
                    "resolved origin count mismatch: {} ops but {} origins",
                    function.ops.len(),
                    function.op_origins.len()
                ),
            ));
        }

        let mut frames = Vec::new();
        for (op_index, op) in function.ops.iter().enumerate() {
            let origin = function.op_origins.get(op_index).cloned().unwrap_or(None);
            match op {
                ResolvedInstruction::If(_) => frames.push(ControlFrame::If { seen_else: false }),
                ResolvedInstruction::Block(_) => frames.push(ControlFrame::Block),
                ResolvedInstruction::Loop => frames.push(ControlFrame::Loop),
                ResolvedInstruction::Else => {
                    let Some(ControlFrame::If { seen_else }) = frames.last_mut() else {
                        return Err(BackendError::in_function(
                            function_path.clone(),
                            Some(op_index),
                            origin,
                            "`else` outside `if`".to_string(),
                        ));
                    };
                    if *seen_else {
                        return Err(BackendError::in_function(
                            function_path.clone(),
                            Some(op_index),
                            origin,
                            "multiple `else` blocks for one `if`".to_string(),
                        ));
                    }
                    *seen_else = true;
                }
                ResolvedInstruction::End => {
                    if frames.pop().is_none() {
                        return Err(BackendError::in_function(
                            function_path.clone(),
                            Some(op_index),
                            origin,
                            "`end` without matching block".to_string(),
                        ));
                    }
                }
                ResolvedInstruction::Break(depth) | ResolvedInstruction::BreakIf(depth) => {
                    if *depth >= frames.len() {
                        return Err(BackendError::in_function(
                            function_path.clone(),
                            Some(op_index),
                            origin,
                            format!(
                                "invalid branch depth {depth}, current nesting depth is {}",
                                frames.len()
                            ),
                        ));
                    }
                }
                ResolvedInstruction::Const(ImmediateValue::String(_)) => {
                    return Err(BackendError::in_function(
                        function_path.clone(),
                        Some(op_index),
                        origin,
                        "string constants must be lowered to array construction".to_string(),
                    ));
                }
                ResolvedInstruction::F32Op(
                    NumberOperation::And
                    | NumberOperation::Or
                    | NumberOperation::Xor
                    | NumberOperation::Rem,
                )
                | ResolvedInstruction::F64Op(
                    NumberOperation::And
                    | NumberOperation::Or
                    | NumberOperation::Xor
                    | NumberOperation::Rem,
                ) => {
                    return Err(BackendError::in_function(
                        function_path.clone(),
                        Some(op_index),
                        origin,
                        "unsupported floating-point operation".to_string(),
                    ));
                }
                _ => {}
            }
        }

        if !frames.is_empty() {
            return Err(BackendError::in_function(
                function_path.clone(),
                None,
                None,
                "unclosed block/if/loop in function".to_string(),
            ));
        }
    }

    Ok(())
}
