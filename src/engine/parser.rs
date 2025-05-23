//! Parses regular expression expressions and converts them to AST.
use std::{
    error::Error,
    fmt::{self, Display},
    mem::take,
};

/// Types for expressing AST.
#[derive(Debug)]
pub enum AST {
    Char(char),
    Dot,
    Plus(Box<AST>),
    Star(Box<AST>),
    Question(Box<AST>),
    Or(Box<AST>, Box<AST>),
    Seq(Vec<AST>),
}

impl AST {
    fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        let indent = " ".repeat(depth);
        let branch = if depth == 0 { "  " } else { "└─" };

        match self {
            AST::Char(c) => writeln!(f, "{}└─Char({})", indent, c),
            AST::Dot => writeln!(f, "{}└─Dot", indent),
            AST::Plus(ast) => {
                writeln!(f, "{}{}Plus", indent, branch)?;
                ast.fmt_with_indent(f, depth + 2)
            }
            AST::Star(ast) => {
                writeln!(f, "{}{}Star", indent, branch)?;
                ast.fmt_with_indent(f, depth + 2)
            }
            AST::Question(ast) => {
                writeln!(f, "{}{}Question", indent, branch)?;
                ast.fmt_with_indent(f, depth + 2)
            }
            AST::Or(lhs, rhs) => {
                writeln!(f, "{}{}Or", indent, branch)?;
                lhs.fmt_with_indent(f, depth + 2)?;
                rhs.fmt_with_indent(f, depth)
            }
            AST::Seq(nodes) => {
                writeln!(f, "{}{}Seq", indent, branch)?;
                for node in nodes {
                    node.fmt_with_indent(f, depth + 2)?;
                }
                Ok(())
            }
        }
    }
}

impl Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}

/// Types to represent parse error.
#[derive(Debug)]
pub enum ParseError {
    InvalidEscape(usize, char),
    InvalidRightParen(usize),
    NoPrev(usize),
    NoRightParen,
    Empty,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidEscape(pos, c) => {
                write!(f, "ParseError: invalid escape: pos = {}, char = {}", pos, c)
            }
            ParseError::InvalidRightParen(pos) => {
                write!(f, "ParseError: invalid right parenthesis: pos = {}", pos)
            }
            ParseError::NoPrev(pos) => {
                write!(f, "ParseError: no previous expression: pos = {}", pos)
            }
            ParseError::NoRightParen => {
                write!(f, "ParseError: no right parenthesis")
            }
            ParseError::Empty => write!(f, "ParseError: empty expression"),
        }
    }
}

impl Error for ParseError {}

/// Escaping special characters
fn parse_escape(pos: usize, c: char) -> Result<AST, ParseError> {
    match c {
        '\\' | '(' | ')' | '|' | '.' | '+' | '*' | '?' => Ok(AST::Char(c)),
        _ => {
            let err = ParseError::InvalidEscape(pos, c);
            Err(err)
        }
    }
}

/// Enumerated type for use in the parse_dot_plus_star_question function
enum PSQ {
    Plus,
    Star,
    Question,
}

/// ., +, *, ? to AST.
///
/// In postfix notation, it is an error if there is no pattern before ., +, *, or ?.
///
/// Example: *ab, abc|+, etc. are errors.
fn parse_dot_plus_star_question(
    seq: &mut Vec<AST>,
    ast_type: PSQ,
    pos: usize,
) -> Result<(), ParseError> {
    if let Some(prev) = seq.pop() {
        let ast = match ast_type {
            PSQ::Plus => AST::Plus(Box::new(prev)),
            PSQ::Star => AST::Star(Box::new(prev)),
            PSQ::Question => AST::Question(Box::new(prev)),
        };
        seq.push(ast);
        Ok(())
    } else {
        Err(ParseError::NoPrev(pos))
    }
}

/// Converts multiple expressions combined in Or to AST.
///
/// For example, the abc|def|ghi would be the AST::Or(“abc”, AST::Or(“def”, “ghi”))).
fn fold_or(mut seq_or: Vec<AST>) -> Option<AST> {
    if seq_or.len() > 1 {
        // If there is more than one element of seq_or, join expressions with Or
        let mut ast = seq_or.pop().unwrap();
        seq_or.reverse();
        for s in seq_or {
            ast = AST::Or(Box::new(s), Box::new(ast));
        }
        Some(ast)
    } else {
        // If there is more than one element of seq_or, join expressions with Or.
        seq_or.pop()
    }
}

/// Converts a regular expression to an abstract syntax tree.
pub fn parse(expr: &str) -> Result<AST, ParseError> {
    // Types for representing internal states.
    // Char state: String processing in progress
    // Escape state: Escape sequence is being processed
    enum ParseState {
        Char,
        Escape,
    }

    let mut seq = Vec::new();
    let mut seq_or = Vec::new();
    let mut stack = Vec::new();
    let mut state = ParseState::Char;

    for (i, c) in expr.chars().enumerate() {
        match &state {
            ParseState::Char => match c {
                '+' => parse_dot_plus_star_question(&mut seq, PSQ::Plus, i)?,
                '*' => parse_dot_plus_star_question(&mut seq, PSQ::Star, i)?,
                '?' => parse_dot_plus_star_question(&mut seq, PSQ::Question, i)?,
                '(' => {
                    // Stores the current context on the stack,
                    // Empty the current context.
                    let prev = take(&mut seq);
                    let prev_or = take(&mut seq_or);
                    stack.push((prev, prev_or));
                }
                ')' => {
                    // Pop the current context off the stack.
                    if let Some((mut prev, prev_or)) = stack.pop() {
                        // Do not push if the expression is empty, such as “()”.
                        if !seq.is_empty() {
                            seq_or.push(AST::Seq(seq));
                        }

                        // Generate Or.
                        if let Some(ast) = fold_or(seq_or) {
                            prev.push(ast);
                        }

                        // Make the previous context the current context.
                        seq = prev;
                        seq_or = prev_or;
                    } else {
                        // If there are no opening parentheses but closing parentheses, such as “abc)”, an error is returned.
                        return Err(ParseError::InvalidRightParen(i));
                    }
                }
                '|' => {
                    if seq.is_empty() {
                        // “||”, ‘(|abc)’, etc., and error if expression is empty.
                        return Err(ParseError::NoPrev(i));
                    } else {
                        let prev = take(&mut seq);
                        seq_or.push(AST::Seq(prev));
                    }
                }
                '\\' => state = ParseState::Escape,
                '.' => seq.push(AST::Dot),
                _ => seq.push(AST::Char(c)),
            },
            ParseState::Escape => {
                // Escape sequence processing
                let ast = parse_escape(i, c)?;
                seq.push(ast);
                state = ParseState::Char;
            }
        }
    }

    // Error if closing brackets are missing.
    if !stack.is_empty() {
        return Err(ParseError::NoRightParen);
    }

    // Do not push if expression is empty, such as “()”.
    if !seq.is_empty() {
        seq_or.push(AST::Seq(seq));
    }

    // Generate Or and return it if successful.
    if let Some(ast) = fold_or(seq_or) {
        Ok(ast)
    } else {
        Err(ParseError::Empty)
    }
}
