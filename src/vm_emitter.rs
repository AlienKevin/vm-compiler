use crate::vm_parser::*;

pub fn emit(program_name: &str, instructions: Vec<Instruction>) -> String {
  instructions
    .iter()
    .filter(|instruction| match instruction { Instruction::Ignored => false, _ => true } )
    .map(|instruction| match instruction {
      Instruction::Arithmetic(arith_instruction) =>
        match arith_instruction {
          ArithInstruction::Add =>
            emit_binary_arithmetic("M=M+D"),
          ArithInstruction::Sub =>
            emit_binary_arithmetic("M=M-D"),
          ArithInstruction::Eq =>
            emit_binary_arithmetic(
"D=M-D
@EQ
D;JEQ
@SP
M=0
@NOT_EQ
0;JMP
(EQ)
@SP
M=1
(NOT_EQ)"),
          ArithInstruction::Gt =>
            emit_binary_arithmetic(
"D=M-D
@GT
D;JGT
@SP
M=0
@NOT_GT
0;JMP
(GT)
@SP
M=1
(NOT_GT)"),
          ArithInstruction::Lt =>
            emit_binary_arithmetic(
"D=M-D
@LT
D;JLT
@SP
M=0
@NOT_LT
0;JMP
(LT)
@SP
M=1
(NOT_LT)"),
          ArithInstruction::And =>
            emit_binary_arithmetic("D=D&M"),
          ArithInstruction::Or =>
            emit_binary_arithmetic("D=D|M"),
          ArithInstruction::Neg =>
            emit_unary_arithmetic("M=-M"),
          ArithInstruction::Not =>
            emit_unary_arithmetic("M=!M"),
        },
      Instruction::Push { segment, offset } =>
        match segment {
          Segment::Local =>
            emit_push_fixed_segment("LCL", offset),
          Segment::Argument =>
            emit_push_fixed_segment("ARG", offset),
          Segment::This =>
            emit_push_fixed_segment("THIS", offset),
          Segment::That =>
            emit_push_fixed_segment("THAT", offset),
          Segment::Constant =>
            emit_push_constant_segment(offset),
          Segment::Static =>
            emit_push_static_segment(program_name, offset),
          Segment::Temp =>
            emit_push_temp_segment(offset),
          Segment::Pointer =>
            emit_push_pointer_segment(offset),
        },
      Instruction::Pop { segment, offset} =>
        match segment {
          Segment::Local =>
            emit_pop_fixed_segment("LCL", offset),
          Segment::Argument =>
            emit_pop_fixed_segment("ARG", offset),
          Segment::This =>
            emit_pop_fixed_segment("THIS", offset),
          Segment::That =>
            emit_pop_fixed_segment("THAT", offset),
          Segment::Constant =>
            panic!("`pop constant {}` is an invalid command.\nYou can't store a popped value into a constant. The parser should filter out this impossible case before emitting.", offset),
          Segment::Static =>
            emit_pop_static_segment(program_name, offset),
          Segment::Temp =>
            emit_pop_temp_segment(offset),
          Segment::Pointer =>
            emit_pop_pointer_segment(offset),
        },
      Instruction::Ignored =>
        panic!("The emitter should not encountered Ignored instructions.\nThere's either a problem in the emitter or Rust."),
    }).collect::<Vec<String>>()
    .join("\n")
}

fn emit_binary_arithmetic(operation_str: &str) -> String {
  format!(
"@SP
M=M-1
A=M
D=M
@SP
M=M-1
A=M
{}
@SP
M=M+1", operation_str)
}

fn emit_unary_arithmetic(operation_str: &str) -> String {
  format!(
"@SP
M=M-1
A=M
{}
@SP
M=M+1", operation_str)
}

fn emit_push_fixed_segment(base_address: &str, offset: &usize) -> String {
  format!(
"@{}
D=M
@{}
D=D+A
A=D
D=M
{}", base_address, offset, emit_push_d_to_stack())
}

fn emit_pop_fixed_segment(base_address: &str, offset: &usize) -> String {
  format!(
"@{}
D=M
@{}
D=D+A
@SP
M=M-1
A=M
D=D+M
@SP
A=M
A=D-M
M=D-A", base_address, offset)
}

fn emit_push_constant_segment(number: &usize) -> String {
format!(
"@{}
D=A
{}", number, emit_push_d_to_stack())
}

fn emit_push_static_segment(file_name: &str, offset: &usize) -> String {
  format!(
"@{}.{}
D=M
A=D
D=M
{}", file_name, offset, emit_push_d_to_stack())
}

fn emit_pop_static_segment(file_name: &str, offset: &usize) -> String {
  format!(
"{}
@{}.{}
M=D", emit_pop_stack_to_d(), file_name, offset)
}

fn emit_push_temp_segment(offset: &usize) -> String {
  emit_push_fixed_segment("5", offset)
}

fn emit_pop_temp_segment(offset: &usize) -> String {
  emit_pop_fixed_segment("5", offset)
}

fn emit_push_pointer_segment(offset: &usize) -> String {
  format!(
"@{}
A=M
D=M
{}", if *offset == 0 { "THIS" } else { "THAT" }, emit_push_d_to_stack())
}

fn emit_pop_pointer_segment(offset: &usize) -> String {
  format!(
"@SP
M=M-1
A=M
D=M
@{}
M=D", if *offset == 0 { "THIS" } else { "THAT" })
}

fn emit_push_d_to_stack() -> &'static str {
"@SP
A=M
M=D
@SP
M=M+1"
}

fn emit_pop_stack_to_d() -> &'static str {
"@SP
M=M-1
A=M
D=M"
}
