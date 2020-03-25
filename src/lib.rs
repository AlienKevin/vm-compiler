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
    let source1 =
"// Executes pop and push commands using the virtual memory segments.
// Execute arithmetic and logic commands too.
push constant 10 // push
pop local 0 // pop
add
// end of vm program
";
    assert_eq!(
      parse(source1),
      Ok(vec![
        Instruction::Push { segment: Segment::Constant, offset: 10 },
        Instruction::Pop { segment: Segment::Local, offset: 0 },
        Instruction::Arithmetic(ArithInstruction::Add),
      ])
    );
    let source2 =
"label UNUSED
goto NORMAL
label NORMAL
if-goto UNDEFINED
";
    assert_eq!(
      parse(source2),
      Err(
"1| label UNUSED
         ^^^^^^
⚠️ I found an unused label named UNUSED. Try removing it or use it somewhere.

4| if-goto UNDEFINED
           ^^^^^^^^^
⚠️ I found an undefined label named UNDEFINED. Try removing it or define it somewhere.".to_string())
    );
  }
}