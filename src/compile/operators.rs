use super::*;

type Instruction = wasm_encoder::Instruction<'static>;

impl ModuleEncoder {
  pub fn generate_binary_operator(&mut self, op: BinaryOp) -> u32 {
    use BinaryOp::*;
    match op {
      Plus => self.binary_arithmetic(Type::Integer, I64Add),
      Minus => self.binary_arithmetic(Type::Integer, I64Sub),
      Star => self.binary_arithmetic(Type::Integer, I64Mul),
      Slash => self.binary_arithmetic(Type::Integer, I64DivS),
      Percent => self.binary_arithmetic(Type::Integer, I64RemS),
      PlusDot => self.binary_arithmetic(Type::Real, F64Add),
      MinusDot => self.binary_arithmetic(Type::Real, F64Sub),
      StarDot => self.binary_arithmetic(Type::Real, F64Mul),
      SlashDot => self.binary_arithmetic(Type::Real, F64Div),
      And => self.binary_arithmetic(Type::Boolean, I32And),
      Or => self.binary_arithmetic(Type::Boolean, I32Or),
      Xor => self.binary_arithmetic(Type::Boolean, I32Xor),
      DoubleEqual => {
        self.comparisons(DoubleEqual, I64Eq, F64Eq, I32Eq, I32Eq, I32Const(1))
      },
      BangEqual => {
        self.comparisons(BangEqual, I64Ne, F64Ne, I32Ne, I32Ne, I32Const(0))
      },
      LessEqual => {
        self.comparisons(LessEqual, I64LeS, F64Le, I32LeU, I32LeU, I32Const(1))
      },
      GreaterEqual => self.comparisons(
        GreaterEqual,
        I64GeS,
        F64Ge,
        I32GeU,
        I32GeU,
        I32Const(1),
      ),
      Less => {
        self.comparisons(Less, I64LtS, F64Lt, I32LtU, I32LtU, I32Const(0))
      },
      Greater => {
        self.comparisons(Greater, I64GtS, F64Gt, I32GtU, I32GtU, I32Const(0))
      },
      _ => panic!(),
    }
  }

  fn binary_arithmetic(&mut self, type_: Type, op: Instruction) -> u32 {
    let (head, tail) = self.new_curried_function(
      vec!["a".into(), "b".into()],
      vec![type_.clone(), type_.clone()],
      type_.clone(),
      vec![],
      vec![],
    );
    ["a", "b"].into_iter().for_each(|local| {
      self.func(tail).get_local(local);
      self.unwrap_primitive(tail, &type_);
    });
    self.push(tail, op);
    self.new_struct(tail, &type_);
    head
  }

  fn comparisons(
    &mut self,
    op: BinaryOp,
    integer_op: Instruction,
    real_op: Instruction,
    boolean_op: Instruction,
    glyph_op: Instruction,
    unit_op: Instruction,
  ) -> u32 {
    let integer_type = self.get_type_id(&Type::Integer, false);
    let real_type = self.get_type_id(&Type::Real, false);
    let boolean_type = self.get_type_id(&Type::Boolean, false);
    let glyph_type = self.get_type_id(&Type::Glyph, false);
    let unit_type = self.get_type_id(&Type::Unit, false);
    let string_type = self.get_type_id(&Type::String, false);

    const TRUE: Instruction = I32Const(1);
    const FALSE: Instruction = I32Const(0);
    let (head, tail) = self.new_curried_function(
      vec!["a".into(), "b".into()],
      vec![Type::TypeVariable(0), Type::TypeVariable(0)],
      Type::Boolean,
      vec![],
      vec![],
    );
    let a = self.func(tail).get_local_id("a");
    let b = self.func(tail).get_local_id("b");
    macro_rules! asm {
        ($($e:expr);*;) => {
          let __temp = [$($e,)*];
          self.func(tail).extend(&__temp);
        };
      }
    let type_ops = [
      (integer_type, integer_op),
      (real_type, real_op),
      (boolean_type, boolean_op),
      (glyph_type, glyph_op),
      (unit_type, unit_op),
    ];
    (0..6).for_each(|_| {
      asm!(Block(BlockType::Result(ValType::Ref(RefType::ANYREF))););
    });
    let br_on = |id, depth| BrOnCast {
      relative_depth: depth,
      from_ref_type: RefType::ANYREF,
      to_ref_type: RefType {
        nullable: false,
        heap_type: HeapType::Concrete(id),
      },
    };
    // Jump on cast
    self.func(tail).get_local("a");
    type_ops
      .clone()
      .into_iter()
      .map(|(t, _)| t)
      .enumerate()
      .for_each(|(depth, t)| {
        asm!(br_on(t, depth as u32););
      });
    asm!(
      br_on(string_type, 5);
      Unreachable;
      End;
    );
    // Basic comparisons
    type_ops.into_iter().for_each(|(type_, op)| {
      // Get inner values if not unit
      if type_ != unit_type {
        asm!(
          RefCastNonNull(HeapType::Concrete(type_));
          StructGet {
            struct_type_index: type_,
            field_index: 0,
          };
          LocalGet(b);
          RefCastNonNull(HeapType::Concrete(type_));
          StructGet {
            struct_type_index: type_,
            field_index: 0,
          };
        );
      }
      asm!(
        // Perform comparison
        op;
        StructNew(boolean_type);
        Return;
        End;
      );
    });
    // String comparison
    asm!(
      // If len(first) > len(second)
      RefCastNonNull(HeapType::Concrete(string_type));
      ArrayLen;
      LocalGet(b);
      RefCastNonNull(HeapType::Concrete(string_type));
      ArrayLen;
      I32GtU;
      If(BlockType::Empty);
      match op {
        BinaryOp::DoubleEqual | BinaryOp::LessEqual | BinaryOp::Less => {
          FALSE
        },
        BinaryOp::Greater | BinaryOp::BangEqual | BinaryOp::GreaterEqual => {
          TRUE
        },
        _ => unreachable!(),
      };
      StructNew(boolean_type);
      Return;
      End;
      // If len(second) > len(first)
      LocalGet(a);
      RefCastNonNull(HeapType::Concrete(string_type));
      ArrayLen;
      LocalGet(b);
      RefCastNonNull(HeapType::Concrete(string_type));
      ArrayLen;
      I32LtU;
      If(BlockType::Empty);
      match op {
        BinaryOp::DoubleEqual | BinaryOp::GreaterEqual | BinaryOp::Greater => {
          FALSE
        },
        BinaryOp::Less | BinaryOp::BangEqual | BinaryOp::LessEqual => {
          TRUE
        },
        _ => unreachable!(),
      };
      StructNew(boolean_type);
      Return;
      End;
    );
    let index = self.func(tail).new_local("index".into(), ValType::I32);
    let length = self.func(tail).new_local("index".into(), ValType::I32);
    asm!(
      I32Const(0);
      LocalSet(index);
      LocalGet(a);
      RefCastNonNull(HeapType::Concrete(string_type));
      ArrayLen;
      LocalSet(length);
      // Lexical comparison
      Loop(BlockType::Empty);
      LocalGet(a);
      RefCastNonNull(HeapType::Concrete(string_type));
      LocalGet(index);
      ArrayGetU(string_type);
      LocalGet(b);
      RefCastNonNull(HeapType::Concrete(string_type));
      LocalGet(index);
      ArrayGetU(string_type);
      match op {
        BinaryOp::DoubleEqual => I32Ne,
        BinaryOp::BangEqual => I32Eq,
        BinaryOp::LessEqual => I32GtU,
        BinaryOp::GreaterEqual => I32LtU,
        BinaryOp::Less => I32GeU,
        BinaryOp::Greater => I32LeU,
        _ => unreachable!(),
      };
      If(BlockType::Empty);
      FALSE;
      StructNew(boolean_type);
      Return;
      End;
      LocalGet(index);
      I32Const(1);
      I32Add;
      LocalTee(index);
      LocalGet(length);
      I32LtU;
      BrIf(0);
      End;
      TRUE;
      StructNew(boolean_type);
      Return;
    );
    head
  }

  pub fn generate_unary_operator(&mut self, op: UnaryOp) -> u32 {
    let f = self.new_function(&op.get_type(), "a".into(), vec![], vec![]);
    match op {
      UnaryOp::Minus => {
        self.push(f, I64Const(0));
        self.func(f).get_local("a");
        self.unwrap_primitive(f, &Type::Integer);
        self.push(f, I64Sub);
        self.new_struct(f, &Type::Integer);
      },
      UnaryOp::MinusDot => {
        self.func(f).get_local("a");
        self.unwrap_primitive(f, &Type::Real);
        self.push(f, F64Neg);
        self.new_struct(f, &Type::Real);
      },
      UnaryOp::Not => {
        self.func(f).get_local("a");
        self.unwrap_primitive(f, &Type::Boolean);
        self.push(f, I32Eqz);
        self.new_struct(f, &Type::Boolean);
      },
    };
    f
  }
}
