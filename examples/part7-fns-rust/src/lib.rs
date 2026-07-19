//! fns — the small language grows functions and closures.
//!
//! Lesson 2 gave the language *memory* (an environment). This lesson gives it
//! *abstraction*: named functions you define and call. Two ideas make functions
//! real, and both are about environments:
//!
//!   * A **call** creates a fresh scope holding the arguments, so a function's
//!     parameters don't leak out and calls don't clobber each other.
//!   * A **closure**: a function captures the environment where it was *defined*,
//!     and its call scope chains to that — so it sees the variables in scope where
//!     it was written, not where it was called. That is *lexical scoping*.
//!
//! Environments therefore form a parent chain: a lookup that misses locally walks
//! up to the enclosing scope. Values are now either integers or functions.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// ---------------------------------------------------------------- values & env

/// A runtime value: an integer, or a function (first-class — it can be stored,
/// passed, and returned).
#[derive(Clone)]
pub enum Value {
    Int(i64),
    Func(Rc<Function>),
}

/// A function captures its parameter names, its body expression, and — crucially
/// — the environment in which it was defined (its closure).
pub struct Function {
    params: Vec<String>,
    body: Expr,
    env: Env,
}

/// One lexical scope: its own bindings plus an optional parent to fall through
/// to. The chain of parents is what makes scoping lexical.
pub struct Scope {
    vars: HashMap<String, Value>,
    parent: Option<Env>,
}

/// A shared, mutable environment handle.
pub type Env = Rc<RefCell<Scope>>;

/// A fresh root environment.
pub fn root_env() -> Env {
    Rc::new(RefCell::new(Scope {
        vars: HashMap::new(),
        parent: None,
    }))
}

fn child_env(parent: &Env) -> Env {
    Rc::new(RefCell::new(Scope {
        vars: HashMap::new(),
        parent: Some(parent.clone()),
    }))
}

/// Look a name up, walking from the innermost scope outward — the essence of
/// lexical scoping.
fn lookup(env: &Env, name: &str) -> Option<Value> {
    let scope = env.borrow();
    if let Some(v) = scope.vars.get(name) {
        return Some(v.clone());
    }
    match &scope.parent {
        Some(p) => lookup(p, name),
        None => None,
    }
}

// ---------------------------------------------------------------- lexer

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Num(i64),
    Ident(String),
    Assign,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

/// Tokenize one statement, adding the comma (for argument lists) to lesson 2's
/// lexer.
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
            ',' => {
                tokens.push(Token::Comma);
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

/// An expression node. New since lesson 2: `Call`, a function application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Num(i64),
    Var(String),
    Call(String, Vec<Expr>),
    Neg(Box<Expr>),
    Bin(char, Box<Expr>, Box<Expr>),
}

/// A statement: a function definition, an assignment, or a bare expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    FnDef(String, Vec<String>, Expr),
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

    // factor := Num | Ident '(' args ')' | Ident | '(' expr ')' | '-' factor
    fn factor(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Token::Num(n)) => Ok(Expr::Num(n)),
            Some(Token::Ident(name)) => {
                if self.peek() == Some(&Token::LParen) {
                    self.pos += 1; // consume '('
                    let args = self.args()?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Var(name))
                }
            }
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

    // args := (expr (',' expr)*)? ')'  — the '(' is already consumed
    fn args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if self.peek() == Some(&Token::RParen) {
            self.pos += 1;
            return Ok(args);
        }
        loop {
            args.push(self.expr()?);
            match self.next() {
                Some(Token::Comma) => continue,
                Some(Token::RParen) => return Ok(args),
                _ => return Err("expected ',' or ')' in argument list".to_string()),
            }
        }
    }
}

/// True when the tokens are a function definition: `name ( ... ) =`.
fn is_fn_def(tokens: &[Token]) -> bool {
    if !matches!(tokens.first(), Some(Token::Ident(_))) || tokens.get(1) != Some(&Token::LParen) {
        return false;
    }
    let mut depth = 0;
    for (i, t) in tokens.iter().enumerate().skip(1) {
        match t {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth == 0 {
                    return tokens.get(i + 1) == Some(&Token::Assign);
                }
            }
            _ => {}
        }
    }
    false
}

/// Parse one statement: a function definition (`name(params) = body`), an
/// assignment (`name = expr`), or a bare expression.
pub fn parse_stmt(tokens: Vec<Token>) -> Result<Stmt, String> {
    if is_fn_def(&tokens) {
        return parse_fn_def(tokens);
    }
    let is_assign =
        matches!(tokens.first(), Some(Token::Ident(_))) && tokens.get(1) == Some(&Token::Assign);
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

fn parse_fn_def(tokens: Vec<Token>) -> Result<Stmt, String> {
    let name = match &tokens[0] {
        Token::Ident(n) => n.clone(),
        _ => unreachable!(),
    };
    // Parameters: identifiers between the parentheses.
    let mut params = Vec::new();
    let mut i = 2; // skip name and '('
    if tokens.get(i) != Some(&Token::RParen) {
        loop {
            match tokens.get(i) {
                Some(Token::Ident(p)) => params.push(p.clone()),
                _ => return Err("expected a parameter name".to_string()),
            }
            i += 1;
            match tokens.get(i) {
                Some(Token::Comma) => i += 1,
                Some(Token::RParen) => break,
                _ => return Err("expected ',' or ')' in parameter list".to_string()),
            }
        }
    }
    i += 1; // skip ')'
            // tokens[i] is '=' (guaranteed by is_fn_def); the body is everything after.
    let body_tokens = tokens[i + 1..].to_vec();
    let mut p = Parser {
        tokens: body_tokens,
        pos: 0,
    };
    let body = p.expr()?;
    if p.pos != p.tokens.len() {
        return Err("unexpected trailing input in function body".to_string());
    }
    Ok(Stmt::FnDef(name, params, body))
}

// ---------------------------------------------------------------- evaluator

fn as_int(v: Value) -> Result<i64, String> {
    match v {
        Value::Int(n) => Ok(n),
        Value::Func(_) => Err("cannot do arithmetic on a function".to_string()),
    }
}

/// Evaluate an expression to a value in the given environment.
pub fn eval(e: &Expr, env: &Env) -> Result<Value, String> {
    match e {
        Expr::Num(n) => Ok(Value::Int(*n)),
        Expr::Var(name) => lookup(env, name).ok_or_else(|| format!("undefined variable '{name}'")),
        Expr::Neg(inner) => Ok(Value::Int(-as_int(eval(inner, env)?)?)),
        Expr::Bin(op, l, r) => {
            let a = as_int(eval(l, env)?)?;
            let b = as_int(eval(r, env)?)?;
            let v = match op {
                '+' => a + b,
                '-' => a - b,
                '*' => a * b,
                '/' => {
                    if b == 0 {
                        return Err("division by zero".to_string());
                    }
                    a / b
                }
                _ => unreachable!(),
            };
            Ok(Value::Int(v))
        }
        Expr::Call(name, args) => {
            let func = match lookup(env, name) {
                Some(Value::Func(f)) => f,
                Some(Value::Int(_)) => return Err(format!("'{name}' is not a function")),
                None => return Err(format!("undefined function '{name}'")),
            };
            if args.len() != func.params.len() {
                return Err(format!(
                    "'{name}' expects {} argument(s), got {}",
                    func.params.len(),
                    args.len()
                ));
            }
            // Evaluate arguments in the CALLER's environment...
            let mut argv = Vec::with_capacity(args.len());
            for a in args {
                argv.push(eval(a, env)?);
            }
            // ...but run the body in a new scope chained to the function's
            // DEFINING environment (its closure) — this is lexical scoping.
            let call_scope = child_env(&func.env);
            for (p, v) in func.params.iter().zip(argv) {
                call_scope.borrow_mut().vars.insert(p.clone(), v);
            }
            eval(&func.body, &call_scope)
        }
    }
}

/// Run one statement, returning its value. A definition builds a closure; an
/// assignment stores a value; a bare expression is evaluated.
pub fn exec(stmt: &Stmt, env: &Env) -> Result<Value, String> {
    match stmt {
        Stmt::FnDef(name, params, body) => {
            let f = Value::Func(Rc::new(Function {
                params: params.clone(),
                body: body.clone(),
                env: env.clone(), // capture the defining environment
            }));
            env.borrow_mut().vars.insert(name.clone(), f.clone());
            Ok(f)
        }
        Stmt::Assign(name, e) => {
            let v = eval(e, env)?;
            env.borrow_mut().vars.insert(name.clone(), v.clone());
            Ok(v)
        }
        Stmt::Expr(e) => eval(e, env),
    }
}

fn show(v: &Value) -> String {
    match v {
        Value::Int(n) => n.to_string(),
        Value::Func(_) => "<fn>".to_string(),
    }
}

/// Run a whole program: one statement per non-empty line, sharing one root
/// environment. Returns one `"line  =>  value"` (or error) string per statement.
pub fn run_program(src: &str) -> Vec<String> {
    let env = root_env();
    let mut out = Vec::new();
    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let result = tokenize(trimmed)
            .and_then(parse_stmt)
            .and_then(|s| exec(&s, &env));
        match result {
            Ok(v) => out.push(format!("{trimmed}  =>  {}", show(&v))),
            Err(e) => out.push(format!("{trimmed}  =>  error: {e}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_a_function_definition() {
        assert_eq!(
            tokenize("f(x, y) = x + y").unwrap(),
            vec![
                Token::Ident("f".into()),
                Token::LParen,
                Token::Ident("x".into()),
                Token::Comma,
                Token::Ident("y".into()),
                Token::RParen,
                Token::Assign,
                Token::Ident("x".into()),
                Token::Plus,
                Token::Ident("y".into()),
            ]
        );
    }

    #[test]
    fn defines_and_calls_a_function() {
        let out = run_program("double(x) = x * 2\ndouble(21)");
        assert_eq!(
            out,
            vec!["double(x) = x * 2  =>  <fn>", "double(21)  =>  42"]
        );
    }

    #[test]
    fn handles_multiple_arguments_and_nested_calls() {
        let out = run_program("add(a, b) = a + b\nadd(add(1, 2), 3)");
        assert_eq!(*out.last().unwrap(), "add(add(1, 2), 3)  =>  6");
    }

    #[test]
    fn closures_use_lexical_not_dynamic_scope() {
        // f captures x = 10 from where it is DEFINED. g has its own parameter
        // also named x; calling f from inside g must still see the defining x
        // (10), not g's argument (999). That is lexical scoping.
        let out = run_program("x = 10\nf(n) = n + x\ng(x) = f(0)\ng(999)");
        assert_eq!(*out.last().unwrap(), "g(999)  =>  10");
    }

    #[test]
    fn closure_sees_later_updates_to_captured_variable() {
        // The captured environment is shared, so reassigning base is visible.
        let out = run_program("base = 100\nshift(n) = n + base\nshift(5)\nbase = 200\nshift(5)");
        assert_eq!(out[2], "shift(5)  =>  105");
        assert_eq!(out[4], "shift(5)  =>  205");
    }

    #[test]
    fn reports_arity_and_kind_errors() {
        let out = run_program("double(x) = x * 2\ndouble(1, 2)\nnope(3)\n5(3)");
        assert!(out[1].contains("expects 1 argument(s), got 2"));
        assert!(out[2].contains("undefined function 'nope'"));
        assert!(out[3].contains("not a function") || out[3].contains("trailing"));
    }
}
