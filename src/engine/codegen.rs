use super::{Instruction, parser::AST};
use crate::helper::safe_add;
use std::{
    error::Error,
    fmt::{self, Display},
};

/// Type representing a code generation error.
#[derive(Debug)]
pub enum CodeGenError {
    PCOverFlow,
    FailStar,
    FailOr,
    FailQuestion,
}

impl Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CodeGenError: {:?}", self)
    }
}

impl Error for CodeGenError {}

/// Code generator.
#[derive(Default, Debug)]
struct Generator {
    pc: usize,
    insts: Vec<Instruction>,
}

/// Increment program counter.
fn inc_pc(&mut self) -> Result<(), CodeGenError> {
    safe_add(&mut self.pc, &1, || CodeGenError::PCOverFlow)
}

// Function to pattern AST and perform code generation.
fn gen_expr(&mut self, ast: &AST) -> Result<(), CodeGenError> {
    match ast {
        AST::Char(c) => self.gen_char(*c)?,
        AST::Or(e1, e2) => self.gen_or(e1, e2)?,
        AST::Plus(e) => self.gen_plus(e)?,
        AST::Star(e) => self.gen_star(e)?,
        AST::Question(e) => self.gen_question(e)?,
        AST::Seq(v) => self.gen_seq(v)?,
    }

    Ok(())
}
