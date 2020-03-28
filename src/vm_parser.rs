use lip::*;
use im::hashset::HashSet;
use std::hash::{Hash};
use itertools::Itertools;
use lazy_static::lazy_static;

#[derive(Hash, Clone, Eq, PartialEq, Debug)]
struct VMLocation {
  row: usize,
  col: usize,
}

#[derive(Hash, Clone, Eq, PartialEq, Debug)]
struct VMLocatedString {
  from: VMLocation,
  to: VMLocation,
  value: String
}

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
  Label(String),
  Goto(String),
  IfGoto(String),
  Function {
    name: String,
    local_vars: usize,
  },
  Return,
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

#[derive(Clone, Debug)]
pub struct State {
  defined_labels: HashSet<VMLocatedString>,
  used_labels: HashSet<VMLocatedString>,
}

lazy_static! {
  static ref RESERVED_WORDS: std::collections::HashSet<String> = std::collections::HashSet::new();
}

// // Executes pop and push commands using the virtual memory segments.
// push constant 10
// pop local 0
// add
pub fn parse<'a>(source: &'a str) -> Result<Vec<Instruction>, String> {
  let initial_state = State {
    defined_labels: HashSet::new(),
    used_labels: HashSet::new(),
  };
  let output = one_or_more(
    left(
      one_of!(
        push_instruction(),
        pop_instruction(),
        arith_instruction(),
        label_declaration(),
        goto_instruction(),
        if_goto_instruction(),
        function_declaration(),
        return_statement(),
        comment_or_spaces()
      ),
      newline_with_comment("//")
    )
  ).end().parse(source, Location { row: 1, col: 1 }, initial_state)
  .map(| instructions |
    instructions.into_iter().filter(|instruction| match instruction { Instruction::Ignored => false, _ => true } ).collect()
  );
  match output {
    ParseResult::Ok { output, state, .. } => {
      let defined_labels = state.defined_labels;
      let defined_label_names = defined_labels.clone().into_iter().map(|located_label| located_label.value).collect::<HashSet<String>>();
      let used_labels = state.used_labels;
      let used_label_names = used_labels.clone().into_iter().map(|located_label| located_label.value).collect::<HashSet<String>>();
      let label_name_difference = defined_label_names.difference(used_label_names);
      let label_difference = used_labels.into_iter().filter(|located_label|
        label_name_difference.contains(&located_label.value)
        ).collect::<HashSet<VMLocatedString>>();
      if label_difference.is_empty() {
        Ok(output)
      } else {
        Err(
          label_difference.iter().sorted_by_key(|located_label| located_label.from.row)
          .map(|located_label|
            display_error(source, 
            format!(
              "I found an undefined label named {}. Try removing it or define it somewhere.",
              located_label.value
            ),
            to_location(located_label.from.clone()), to_location(located_label.to.clone())
            )
          ).collect::<Vec<String>>().join("\n\n")
        )
      }
    },
    ParseResult::Err {
      message: error_message,
      from,
      to,
      ..
    } => Err(display_error(source, error_message, from, to)),
  }
}

fn push_instruction<'a>() -> BoxedParser<'a, Instruction, State> {
  chain!(
    token("push"),
    space1(),
    segment_label(),
    space1(),
    int()
  ).map(|output| match output {
    (_, (_, (segment, (_, offset)))) =>
      Instruction::Push { segment, offset }
  })
}

fn pop_instruction<'a>() -> BoxedParser<'a, Instruction, State> {
  chain!(
    token("pop"),
    space1(),
    segment_label(),
    space1(),
    int()
  ).and_then(|output|
    move |input, location, state|
      match output.clone() {
      (_, (_, (segment, (_, offset)))) =>
        match segment {
          Segment::Constant =>
            ParseResult::Err {
              message: format!("You can't store a popped value into a constant.\nTry pushing a constant onto the stack using `push constant {}` or consider push/pop other memory segments like `local` and `argument`.", offset),
              from: Location {
                col: 1,
                ..location
              },
              to: location,
              state,
            },
          Segment::Pointer =>
            if offset > 1 {
              ParseResult::Err {
                message: format!("I found that {} is outside the allowed range of pointers.\nYou can only push/pop pointer 0 or 1. Pointer 0 refers to `this` and pointer 1 refers to `that`.", offset),
                from: Location {
                  col: 1,
                  ..location
                },
                to: location,
                state,
              }
            } else {
              ParseResult::Ok {
                input,
                output: Instruction::Pop { segment, offset },
                location,
                state,
              }
            }
          _ =>
            ParseResult::Ok {
              input,
              output: Instruction::Pop { segment, offset },
              location,
              state,
            }
        }
      }
  )
}

fn arith_instruction<'a>() -> BoxedParser<'a, Instruction, State> {
  one_of!(
    token("add").map(|_| ArithInstruction::Add),
    token("sub").map(|_| ArithInstruction::Sub),
    token("neg").map(|_| ArithInstruction::Neg),
    token("eq").map(|_| ArithInstruction::Eq),
    token("gt").map(|_| ArithInstruction::Gt),
    token("lt").map(|_| ArithInstruction::Lt),
    token("and").map(|_| ArithInstruction::And),
    token("or").map(|_| ArithInstruction::Or),
    token("not").map(|_| ArithInstruction::Not)
  ).map(|output| Instruction::Arithmetic(output))
}

// label LOOP_START
fn label_declaration<'a>() -> BoxedParser<'a, Instruction, State> {
  chain!(
    token("label"),
    space1(),
    located(label())
  ).update(|input, output, location, state| match output {
    (_, (_, label)) =>
      if state.defined_labels.iter().map(|located_label| located_label.value.clone())
        .collect::<HashSet<String>>().contains(&label.value) {
        ParseResult::Err {
          message: format!("I found a duplicated label name `{}`. Try renaming it.", &label.value),
          from: Location {
            col: location.col - label.value.len(),
            ..location
          },
          to: location,
          state,
        }
      } else {
        ParseResult::Ok {
          input,
          output: Instruction::Label(label.value.clone()),
          location,
          state: State {
            defined_labels: state.defined_labels.update(to_vmlocated_string(label)),
            ..state
          }
        }
      }
  })
}

fn label<'a>() -> impl Parser<'a, String, State> {
  variable(
    &(|c: &char| c.is_alphabetic()),
    &(|c: &char| c.is_alphanumeric()),
    &(|c: &char| *c == '_' || *c == '.' || *c == '$'),
    &RESERVED_WORDS,
    "a label like `LOOP_ONE` or `ponggame.run$if_end1`"
  )
}

fn goto_instruction<'a>() -> BoxedParser<'a, Instruction, State> {
  chain!(
    token("goto"),
    space1(),
    located(label())
  ).update(|input, output, location, state| match output {
    (_, (_, located_label)) =>
      ParseResult::Ok {
        input,
        output: Instruction::Goto(located_label.value.clone()),
        location,
        state: State {
          used_labels: state.used_labels.update(to_vmlocated_string(located_label)),
          ..state
        }
      }
  })
}

fn if_goto_instruction<'a>() -> BoxedParser<'a, Instruction, State> {
  chain!(
    token("if-goto"),
    space1(),
    located(label())
  ).update(|input, output, location, state| match output {
    (_, (_, located_label)) =>
      ParseResult::Ok {
        input,
        output: Instruction::IfGoto(located_label.value.clone()),
        location,
        state: State {
          used_labels: state.used_labels.update(to_vmlocated_string(located_label)),
          ..state
        }
      }
  })
}

fn function_declaration<'a>() -> BoxedParser<'a, Instruction, State> {
  chain!(
    token("function"),
    space1(),
    located(label()),
    space1(),
    int()
  ).update(|input, output, location, state| match output {
    (_, (_, (label, (_, local_vars)))) =>
      if state.defined_labels.iter().map(|located_label| located_label.value.clone())
        .collect::<HashSet<String>>().contains(&label.value) {
        ParseResult::Err {
          message: format!("I found a duplicated label name `{}`. Try renaming it.", &label.value),
          from: Location {
            col: location.col - label.value.len(),
            ..location
          },
          to: location,
          state,
        }
      } else {
        ParseResult::Ok {
          input,
          output: Instruction::Function {
            name: label.value.clone(),
            local_vars,
          },
          location,
          state: State {
            defined_labels: state.defined_labels.update(to_vmlocated_string(label)),
            ..state
          }
        }
      }
  })
}

fn return_statement<'a>() -> BoxedParser<'a, Instruction, State> {
  token("return").map(|_| Instruction::Return)
}

fn to_vmlocated_string(located_str: Located<String>) -> VMLocatedString {
  VMLocatedString {
    from: to_vmlocation(located_str.from),
    to: to_vmlocation(located_str.to),
    value: located_str.value,
  }
}

fn to_vmlocation(location: Location) -> VMLocation {
  VMLocation {
    row: location.row,
    col: location.col,
  }
}

fn to_location(location: VMLocation) -> Location {
  Location {
    row: location.row,
    col: location.col,
  }
}

fn comment_or_spaces<'a>() -> BoxedParser<'a, Instruction, State> {
  token("").map(|_| Instruction::Ignored)
}

fn segment_label<'a>() -> BoxedParser<'a, Segment, State> {
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