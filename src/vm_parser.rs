extern crate lip;

use lip::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
  Arithmetic(ArithInstruction),
  Push {
    segment: Segment,
    offset: usize,
  },
  Pop {
    segment: Segment,
    offset: usize,
  },
  Ignored,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithInstruction {
  Add,
  Sub,
  Neg,
  Eq,
  Gt,
  Lt,
  And,
  Or,
  Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
  Local,
  Argument,
  This,
  That,
  Constant,
  Static,
  Temp,
  Pointer,
}

// // Executes pop and push commands using the virtual memory segments.
// push constant 10
// pop local 0
// add
pub fn parse<'a>(source: &'a str) -> Result<Vec<Instruction>, String> {
  let output = one_or_more_till_end(
    one_of!(
      push_instruction(),
      pop_instruction(),
      arith_instruction(),
      comment_or_spaces()
    )
  ).parse(source, Location { row: 1, col: 1 }, ())
  .map(| instructions |
    instructions.into_iter().filter(|instruction| match instruction { Instruction::Ignored => false, _ => true } ).collect()
  );
  match output {
    ParseResult::ParseOk { output, .. } =>
      Ok(output),
    ParseResult::ParseErr {
      message: error_message,
      from,
      to,
      ..
    } => Err(display_error(source, error_message, from, to)),
  }
}

fn push_instruction<'a>() -> BoxedParser<'a, Instruction, ()> {
  chain!(
    token("push"),
    space1(),
    segment_label(),
    space1(),
    whole_decimal(),
    newline_with_comment("//")
  ).map(|output| match output {
    (_, (_, (segment, (_, (offset, _))))) =>
      Instruction::Push { segment, offset }
  })
}

fn pop_instruction<'a>() -> BoxedParser<'a, Instruction, ()> {
  chain!(
    token("pop"),
    space1(),
    segment_label(),
    space1(),
    whole_decimal(),
    newline_with_comment("//")
  ).map(|output| match output {
    (_, (_, (segment, (_, (offset, _))))) =>
      Instruction::Pop { segment, offset }
  })
}

fn arith_instruction<'a>() -> BoxedParser<'a, Instruction, ()> {
  left(
    one_of!(
      token("add").map(|_| ArithInstruction::Add),
      token("sub").map(|_| ArithInstruction::Sub),
      token("neg").map(|_| ArithInstruction::Neg),
      token("eq").map(|_| ArithInstruction::Eq),
      token("gt").map(|_| ArithInstruction::Gt),
      token("lt").map(|_| ArithInstruction::Lt),
      token("and").map(|_| ArithInstruction::Add),
      token("or").map(|_| ArithInstruction::Or),
      token("not").map(|_| ArithInstruction::Not)
    ).map(|output| Instruction::Arithmetic(output)),
    newline_with_comment("//")
  )
}

fn comment_or_spaces<'a>() -> BoxedParser<'a, Instruction, ()> {
  either(line_comment("//"), newline_char()).map(|_| Instruction::Ignored)
}

fn segment_label<'a>() -> BoxedParser<'a, Segment, ()> {
  one_of!(
    token("local").map(|_| Segment::Local),
    token("argument").map(|_| Segment::Argument),
    token("this").map(|_| Segment::This),
    token("that").map(|_| Segment::That),
    token("constant").map(|_| Segment::Constant),
    token("static").map(|_| Segment::Static),
    token("temp").map(|_| Segment::Temp),
    token("pointer").map(|_| Segment::Pointer)
  )
}