mod vm_parser;
mod vm_emitter;

pub fn compile(program_name: &str, source: &str) -> Result<String, String>
{
  vm_parser::parse(source).map(
    |instructions| vm_emitter::emit(program_name, instructions)
  )
}

#[cfg(test)]
mod test {
  use crate::vm_parser::*;

  #[test]
  fn test_parser() {
    let source =
"// Executes pop and push commands using the virtual memory segments.
// Execute arithmetic and logic commands too.
push constant 10 // push
pop local 0 // pop
add
// end of vm program
";
    assert_eq!(
      parse(source),
      Ok(vec![
        Instruction::Push { segment: Segment::Constant, offset: 10 },
        Instruction::Pop { segment: Segment::Local, offset: 0 },
        Instruction::Arithmetic(ArithInstruction::Add),
      ])
    );
  }
}