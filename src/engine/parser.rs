//! Parses regular expression expressions and converts them to AST.
use std::{
    error::Error,
    fmt::{self, Display},
    mem::take,
    sync::RwLock,
};

static SPACE_COUNT: RwLock<usize> = RwLock::new(0);

fn add_space_count() {
    SPACE_COUNT
        .write()
        .map(|mut space_count| *space_count += 2)
        .unwrap_or_else(|e| eprintln!("Lock poisoned: {}", e));
}

/// Types for expressing AST.
#[derive(Debug)]
pub enum AST {
    Char(char),
    Plus(Box<AST>),
    Star(Box<AST>),
    Question(Box<AST>),
    Or(Box<AST>, Box<AST>),
    Seq(Vec<AST>),
}

impl Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // if let Ok(mut space) = SPACE.write() {
        //     *space += 1;
        // } else {
        //     eprintln!("could not write for space")
        // }

        let space = " ".repeat(*SPACE_COUNT.read().unwrap());

        match self {
            AST::Char(c) => write!(f, "Char({})", c),
            AST::Plus(ast) => write!(f, "Plus({})", ast),
            AST::Star(ast) => write!(f, "Star({})", ast),
            AST::Question(ast) => write!(f, "Question({})", ast),
            AST::Or(ast1, ast2) => {
                write!(f, "Or\n{}├─{}\n{}└─{}", space, ast1, space, ast2)
            }
            AST::Seq(asts) => {
                for (index, ast) in asts.iter().enumerate() {
                    if index + 1 == asts.len() {
                        // write!(f, "{}└─", space)?;
                        add_space_count();
                    } else {
                        // write!(f, "{}├─", space)?;
                    }
                    ast.fmt(f)?;
                }
                Ok(())
            }
        }
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
        '\\' | '(' | ')' | '|' | '+' | '*' | '?' => Ok(AST::Char(c)),
        _ => {
            let err = ParseError::InvalidEscape(pos, c);
            Err(err)
        }
    }
}

/// Enumerated type for use in the parse_plus_star_question function
enum PSQ {
    Plus,
    Star,
    Question,
}

/// +, *, ? to AST.
///
/// In postfix notation, it is an error if there is no pattern before +, *, or ?.
///
/// Example: *ab, abc|+, etc. are errors.
fn parse_plus_star_question(
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
                '+' => parse_plus_star_question(&mut seq, PSQ::Plus, i)?,
                '*' => parse_plus_star_question(&mut seq, PSQ::Star, i)?,
                '?' => parse_plus_star_question(&mut seq, PSQ::Question, i)?,
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
