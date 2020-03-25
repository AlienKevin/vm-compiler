use crate::vm_parser::*;

pub fn emit(program_name: &str, instructions: Vec<Instruction>) -> String {
  instructions
    .iter()
    .filter(|instruction| match instruction { Instruction::Ignored => false, _ => true } )
    .enumerate()
    .map(|(instruction_index, instruction)| match instruction {
      Instruction::Arithmetic(arith_instruction) =>
        match arith_instruction {
          ArithInstruction::Add =>
            emit_binary_arithmetic("M=M+D"),
          ArithInstruction::Sub =>
            emit_binary_arithmetic("M=M-D"),
          ArithInstruction::Eq =>
            emit_comparison(instruction_index, "EQ"),
          ArithInstruction::Gt =>
            emit_comparison(instruction_index, "GT"),
          ArithInstruction::Lt =>
            emit_comparison(instruction_index, "LT"),
          ArithInstruction::And =>
            emit_binary_arithmetic("M=D&M"),
          ArithInstruction::Or =>
            emit_binary_arithmetic("M=D|M"),
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

fn emit_comparison(instruction_index: usize, operation_str: &str) -> String {
  let comp_success_label = &format!("{}_{}", operation_str, instruction_index);
  let comp_failure_label = &format!("NOT_{}_{}", operation_str, instruction_index);
  let jump_instruction = &format!("J{}", operation_str);
  emit_binary_arithmetic(&format!(
"D=M-D
@{}
D;{}
@SP
A=M
M=0
@{}
0;JMP
({})
@SP
A=M
M=-1
({})", comp_success_label, jump_instruction, comp_failure_label, comp_success_label, comp_failure_label),
  )
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
{}", file_name, offset, emit_push_d_to_stack())
}

fn emit_pop_static_segment(file_name: &str, offset: &usize) -> String {
  format!(
"{}
@{}.{}
M=D", emit_pop_stack_to_d(), file_name, offset)
}

fn emit_push_temp_segment(offset: &usize) -> String {
format!("@5
D=A
@{}
D=D+A
A=D
D=M
{}", offset, emit_push_d_to_stack())
}

fn emit_pop_temp_segment(offset: &usize) -> String {
format!(
"@5
D=A
@{}
D=D+A
@SP
M=M-1
A=M
D=D+M
@SP
A=M
A=D-M
M=D-A", offset)
}

fn emit_push_pointer_segment(offset: &usize) -> String {
  format!(
"@{}
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
