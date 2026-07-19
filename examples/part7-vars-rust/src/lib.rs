//! vars — the expression interpreter, grown into a tiny language with memory.
//!
//! Lesson 1 evaluated one expression at a time. Real languages remember things:
//! you *name* a value and use it later. That is a variable, and the thing that
//! holds the names is an **environment** — a `name -> value` map threaded through
//! evaluation. Add that one idea and the calculator becomes a language:
//!
//! ```text
//!   x = 5
//!   y = x * 2 + 1     ← reads x, binds y
//!   y - x             ← 6
//! ```
//!
//! Still integer arithmetic, so results stay exact and byte-identical across the
//! three languages. The environment introduced here is the seed of scopes and
//! closures in the lessons to come.

use std::collections::HashMap;

/// The running memory of the program: variable names to their integer values.
pub type Env = HashMap<String, i64>;

// ---------------------------------------------------------------- lexer

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Num(i64),
    Ident(String),
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

/// Turn one statement's source into tokens. Adds identifiers (a letter or `_`
/// followed by letters, digits, or `_`) and the `=` assignment token to the
/// lexer from lesson 1.
pub fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = src.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\r' | '\n' => i += 1,
            '=' => {
                tokens.push(Token::Assign);
                i += 1;
            }
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '-' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            '*' => {
                tokens.push(Token::Star);
                i += 1;
            }
            '/' => {
                tokens.push(Token::Slash);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '0'..='9' => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let n = text
                    .parse::<i64>()
                    .map_err(|_| format!("number too large: {text}"))?;
                tokens.push(Token::Num(n));
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                tokens.push(Token::Ident(chars[start..i].iter().collect()));
            }
            _ => return Err(format!("unexpected character '{c}'")),
        }
    }
    Ok(tokens)
}

// ---------------------------------------------------------------- parser

/// An expression node. New since lesson 1: `Var`, a variable reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Num(i64),
    Var(String),
    Neg(Box<Expr>),
    Bin(char, Box<Expr>, Box<Expr>),
}

/// A statement: either bind a name to an expression, or just an expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Assign(String, Expr),
    Expr(Expr),
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn expr(&mut self) -> Result<Expr, String> {
        let mut left = self.term()?;
        while let Some(op) = match self.peek() {
            Some(Token::Plus) => Some('+'),
            Some(Token::Minus) => Some('-'),
            _ => None,
        } {
            self.pos += 1;
            left = Expr::Bin(op, Box::new(left), Box::new(self.term()?));
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut left = self.factor()?;
        while let Some(op) = match self.peek() {
            Some(Token::Star) => Some('*'),
            Some(Token::Slash) => Some('/'),
            _ => None,
        } {
            self.pos += 1;
            left = Expr::Bin(op, Box::new(left), Box::new(self.factor()?));
        }
        Ok(left)
    }

    // factor := Num | Ident | '(' expr ')' | '-' factor
    fn factor(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Token::Num(n)) => Ok(Expr::Num(n)),
            Some(Token::Ident(name)) => Ok(Expr::Var(name)),
            Some(Token::Minus) => Ok(Expr::Neg(Box::new(self.factor()?))),
            Some(Token::LParen) => {
                let inner = self.expr()?;
                match self.next() {
                    Some(Token::RParen) => Ok(inner),
                    _ => Err("expected ')'".to_string()),
                }
            }
            Some(t) => Err(format!("unexpected token {t:?}")),
            None => Err("unexpected end of input".to_string()),
        }
    }
}

/// Parse one statement. `name = expr` is an assignment (detected by an
/// identifier followed by `=`); anything else is a bare expression.
pub fn parse_stmt(tokens: Vec<Token>) -> Result<Stmt, String> {
    let is_assign = matches!(tokens.first(), Some(Token::Ident(_)))
        && matches!(tokens.get(1), Some(Token::Assign));
    let mut p = Parser { tokens, pos: 0 };
    let stmt = if is_assign {
        let name = match p.next() {
            Some(Token::Ident(n)) => n,
            _ => unreachable!(),
        };
        p.pos += 1; // consume '='
        Stmt::Assign(name, p.expr()?)
    } else {
        Stmt::Expr(p.expr()?)
    };
    if p.pos != p.tokens.len() {
        return Err(format!("unexpected trailing input: {:?}", p.tokens[p.pos]));
    }
    Ok(stmt)
}

// ---------------------------------------------------------------- evaluator

/// Evaluate an expression against the current environment. A variable reference
/// looks its name up; an unbound name is an error.
pub fn eval(e: &Expr, env: &Env) -> Result<i64, String> {
    match e {
        Expr::Num(n) => Ok(*n),
        Expr::Var(name) => env
            .get(name)
            .copied()
            .ok_or_else(|| format!("undefined variable '{name}'")),
        Expr::Neg(inner) => Ok(-eval(inner, env)?),
        Expr::Bin(op, l, r) => {
            let a = eval(l, env)?;
            let b = eval(r, env)?;
            match op {
                '+' => Ok(a + b),
                '-' => Ok(a - b),
                '*' => Ok(a * b),
                '/' => {
                    if b == 0 {
                        Err("division by zero".to_string())
                    } else {
                        Ok(a / b)
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

/// Run one statement against the environment, returning the value. An assignment
/// stores the value (and evaluates to it); a bare expression just evaluates.
pub fn exec(stmt: &Stmt, env: &mut Env) -> Result<i64, String> {
    match stmt {
        Stmt::Assign(name, e) => {
            let v = eval(e, env)?;
            env.insert(name.clone(), v);
            Ok(v)
        }
        Stmt::Expr(e) => eval(e, env),
    }
}

/// Run a whole program: one statement per non-empty line, sharing a single
/// environment so state persists. Returns one `"line  =>  value"` (or error)
/// string per statement.
pub fn run_program(src: &str) -> Vec<String> {
    let mut env = Env::new();
    let mut out = Vec::new();
    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let result = tokenize(trimmed)
            .and_then(parse_stmt)
            .and_then(|s| exec(&s, &mut env));
        match result {
            Ok(v) => out.push(format!("{trimmed}  =>  {v}")),
            Err(e) => out.push(format!("{trimmed}  =>  error: {e}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_identifiers_and_assignment() {
        assert_eq!(
            tokenize("x = 5").unwrap(),
            vec![Token::Ident("x".into()), Token::Assign, Token::Num(5)]
        );
    }

    #[test]
    fn assignment_binds_and_reference_reads() {
        let mut env = Env::new();
        let v = exec(&parse_stmt(tokenize("x = 40").unwrap()).unwrap(), &mut env).unwrap();
        assert_eq!(v, 40);
        let r = exec(&parse_stmt(tokenize("x + 2").unwrap()).unwrap(), &mut env).unwrap();
        assert_eq!(r, 42);
    }

    #[test]
    fn state_persists_across_statements() {
        let out = run_program("x = 5\ny = x * 2 + 1\ny - x");
        assert_eq!(
            out,
            vec!["x = 5  =>  5", "y = x * 2 + 1  =>  11", "y - x  =>  6"]
        );
    }

    #[test]
    fn undefined_variable_is_an_error() {
        let out = run_program("z + 1");
        assert_eq!(out, vec!["z + 1  =>  error: undefined variable 'z'"]);
    }

    #[test]
    fn reassignment_updates_using_the_old_value() {
        let out = run_program("x = 1\nx = x + 10\nx");
        assert_eq!(out, vec!["x = 1  =>  1", "x = x + 10  =>  11", "x  =>  11"]);
    }

    #[test]
    fn arithmetic_errors_still_reported() {
        let out = run_program("10 / 0\nfoo bar");
        assert!(out[0].contains("division by zero"));
        assert!(out[1].contains("trailing"));
    }
}
