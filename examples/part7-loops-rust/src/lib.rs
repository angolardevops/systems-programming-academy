//! loops — the interpreter becomes imperative: statements, `print`, and `while`.
//!
//! Until now every line was one expression that echoed its value. A real language
//! *does* things: it produces output and it repeats. This lesson makes that shift.
//! A program is now a sequence of **statements** separated by `;`, grouped with
//! `{ }` **blocks**; `print` emits output; and `while cond do body` **loops**.
//! Together with everything before (variables, functions, closures, conditionals,
//! boolean logic), the language can now run real programs:
//!
//! ```text
//!   i = 1;
//!   while i <= 5 do { print i; i = i + 1 }
//! ```
//!
//! prints 1 through 5. Loops add no computational *power* (recursion already made
//! the language Turing-complete) — they add convenience and iteration in constant
//! stack space. The program's printed output is what the three implementations
//! verify byte-for-byte.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// ---------------------------------------------------------------- values & env

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Func(Rc<Function>),
}

pub struct Function {
    params: Vec<String>,
    body: Expr,
    env: Env,
}

pub struct Scope {
    vars: HashMap<String, Value>,
    parent: Option<Env>,
}

pub type Env = Rc<RefCell<Scope>>;

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
    Semi,
    LBrace,
    RBrace,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Cmp(&'static str),
}

/// Tokenize the whole program. Newlines are whitespace; statements are separated
/// by `;` and grouped by `{ }`.
pub fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = src.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match c {
            ' ' | '\t' | '\r' | '\n' => i += 1,
            ';' => {
                tokens.push(Token::Semi);
                i += 1;
            }
            '{' => {
                tokens.push(Token::LBrace);
                i += 1;
            }
            '}' => {
                tokens.push(Token::RBrace);
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
            '=' if next == Some('=') => {
                tokens.push(Token::Cmp("=="));
                i += 2;
            }
            '=' => {
                tokens.push(Token::Assign);
                i += 1;
            }
            '!' if next == Some('=') => {
                tokens.push(Token::Cmp("!="));
                i += 2;
            }
            '<' if next == Some('=') => {
                tokens.push(Token::Cmp("<="));
                i += 2;
            }
            '<' => {
                tokens.push(Token::Cmp("<"));
                i += 1;
            }
            '>' if next == Some('=') => {
                tokens.push(Token::Cmp(">="));
                i += 2;
            }
            '>' => {
                tokens.push(Token::Cmp(">"));
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

// ---------------------------------------------------------------- ast

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Num(i64),
    Var(String),
    Call(String, Vec<Expr>),
    Neg(Box<Expr>),
    Bin(char, Box<Expr>, Box<Expr>),
    Cmp(&'static str, Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    FnDef(String, Vec<String>, Expr),
    Assign(String, Expr),
    Print(Expr),
    While(Expr, Box<Stmt>),
    Block(Vec<Stmt>),
    Expr(Expr),
}

// ---------------------------------------------------------------- parser

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    fn at(&self, k: usize) -> Option<&Token> {
        self.tokens.get(self.pos + k)
    }
    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }
    fn is_kw(&self, kw: &str) -> bool {
        matches!(self.peek(), Some(Token::Ident(s)) if s == kw)
    }
    fn expect_kw(&mut self, kw: &str) -> Result<(), String> {
        if self.is_kw(kw) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("expected '{kw}'"))
        }
    }

    // program := stmt (';' stmt)* ';'?
    fn program(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        loop {
            while self.peek() == Some(&Token::Semi) {
                self.pos += 1;
            }
            if self.peek().is_none() {
                break;
            }
            stmts.push(self.stmt()?);
            match self.peek() {
                Some(Token::Semi) | None => {}
                other => return Err(format!("expected ';' between statements, found {other:?}")),
            }
        }
        Ok(stmts)
    }

    fn stmt(&mut self) -> Result<Stmt, String> {
        if self.is_kw("print") {
            self.pos += 1;
            return Ok(Stmt::Print(self.expr()?));
        }
        if self.is_kw("while") {
            self.pos += 1;
            let cond = self.expr()?;
            self.expect_kw("do")?;
            let body = self.stmt()?;
            return Ok(Stmt::While(cond, Box::new(body)));
        }
        if self.peek() == Some(&Token::LBrace) {
            self.pos += 1;
            let mut stmts = Vec::new();
            loop {
                while self.peek() == Some(&Token::Semi) {
                    self.pos += 1;
                }
                if self.peek() == Some(&Token::RBrace) {
                    self.pos += 1;
                    break;
                }
                if self.peek().is_none() {
                    return Err("unclosed '{'".to_string());
                }
                stmts.push(self.stmt()?);
                match self.peek() {
                    Some(Token::Semi) | Some(Token::RBrace) => {}
                    other => return Err(format!("expected ';' or '}}', found {other:?}")),
                }
            }
            return Ok(Stmt::Block(stmts));
        }
        if self.is_fn_def_here() {
            return self.fn_def();
        }
        if matches!(self.peek(), Some(Token::Ident(_))) && self.at(1) == Some(&Token::Assign) {
            let name = match self.next() {
                Some(Token::Ident(n)) => n,
                _ => unreachable!(),
            };
            self.pos += 1; // '='
            return Ok(Stmt::Assign(name, self.expr()?));
        }
        Ok(Stmt::Expr(self.expr()?))
    }

    fn is_fn_def_here(&self) -> bool {
        if !matches!(self.peek(), Some(Token::Ident(_))) || self.at(1) != Some(&Token::LParen) {
            return false;
        }
        let mut depth = 0;
        let mut k = 1;
        while let Some(t) = self.at(k) {
            match t {
                Token::LParen => depth += 1,
                Token::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        return self.at(k + 1) == Some(&Token::Assign);
                    }
                }
                _ => {}
            }
            k += 1;
        }
        false
    }

    fn fn_def(&mut self) -> Result<Stmt, String> {
        let name = match self.next() {
            Some(Token::Ident(n)) => n,
            _ => unreachable!(),
        };
        self.pos += 1; // '('
        let mut params = Vec::new();
        if self.peek() != Some(&Token::RParen) {
            loop {
                match self.next() {
                    Some(Token::Ident(p)) => params.push(p),
                    _ => return Err("expected a parameter name".to_string()),
                }
                match self.next() {
                    Some(Token::Comma) => continue,
                    Some(Token::RParen) => break,
                    _ => return Err("expected ',' or ')' in parameter list".to_string()),
                }
            }
        } else {
            self.pos += 1; // ')'
        }
        match self.next() {
            Some(Token::Assign) => {}
            _ => return Err("expected '='".to_string()),
        }
        Ok(Stmt::FnDef(name, params, self.expr()?))
    }

    // ---- expressions (same as the boolean-logic lesson) ----

    fn expr(&mut self) -> Result<Expr, String> {
        if self.is_kw("if") {
            self.pos += 1;
            let cond = self.expr()?;
            self.expect_kw("then")?;
            let then = self.expr()?;
            self.expect_kw("else")?;
            let els = self.expr()?;
            return Ok(Expr::If(Box::new(cond), Box::new(then), Box::new(els)));
        }
        self.or_expr()
    }

    fn or_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.and_expr()?;
        while self.is_kw("or") {
            self.pos += 1;
            left = Expr::Or(Box::new(left), Box::new(self.and_expr()?));
        }
        Ok(left)
    }

    fn and_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.not_expr()?;
        while self.is_kw("and") {
            self.pos += 1;
            left = Expr::And(Box::new(left), Box::new(self.not_expr()?));
        }
        Ok(left)
    }

    fn not_expr(&mut self) -> Result<Expr, String> {
        if self.is_kw("not") {
            self.pos += 1;
            return Ok(Expr::Not(Box::new(self.not_expr()?)));
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let left = self.add()?;
        if let Some(Token::Cmp(op)) = self.peek() {
            let op = *op;
            self.pos += 1;
            return Ok(Expr::Cmp(op, Box::new(left), Box::new(self.add()?)));
        }
        Ok(left)
    }

    fn add(&mut self) -> Result<Expr, String> {
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

    fn factor(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Token::Num(n)) => Ok(Expr::Num(n)),
            Some(Token::Ident(name)) => {
                if self.peek() == Some(&Token::LParen) {
                    self.pos += 1;
                    Ok(Expr::Call(name, self.args()?))
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

/// Parse a whole program into a list of top-level statements.
pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, String> {
    let mut p = Parser { tokens, pos: 0 };
    p.program()
}

// ---------------------------------------------------------------- evaluator

fn as_int(v: Value) -> Result<i64, String> {
    match v {
        Value::Int(n) => Ok(n),
        Value::Func(_) => Err("cannot do arithmetic on a function".to_string()),
    }
}

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
        Expr::Cmp(op, l, r) => {
            let a = as_int(eval(l, env)?)?;
            let b = as_int(eval(r, env)?)?;
            let t = match *op {
                "<" => a < b,
                "<=" => a <= b,
                ">" => a > b,
                ">=" => a >= b,
                "==" => a == b,
                "!=" => a != b,
                _ => unreachable!(),
            };
            Ok(Value::Int(if t { 1 } else { 0 }))
        }
        Expr::And(l, r) => {
            if as_int(eval(l, env)?)? == 0 {
                Ok(Value::Int(0))
            } else {
                Ok(Value::Int(if as_int(eval(r, env)?)? != 0 { 1 } else { 0 }))
            }
        }
        Expr::Or(l, r) => {
            if as_int(eval(l, env)?)? != 0 {
                Ok(Value::Int(1))
            } else {
                Ok(Value::Int(if as_int(eval(r, env)?)? != 0 { 1 } else { 0 }))
            }
        }
        Expr::Not(inner) => Ok(Value::Int(if as_int(eval(inner, env)?)? == 0 {
            1
        } else {
            0
        })),
        Expr::If(cond, then, els) => {
            if as_int(eval(cond, env)?)? != 0 {
                eval(then, env)
            } else {
                eval(els, env)
            }
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
            let mut argv = Vec::with_capacity(args.len());
            for a in args {
                argv.push(eval(a, env)?);
            }
            let call_scope = child_env(&func.env);
            for (p, v) in func.params.iter().zip(argv) {
                call_scope.borrow_mut().vars.insert(p.clone(), v);
            }
            eval(&func.body, &call_scope)
        }
    }
}

fn show(v: &Value) -> String {
    match v {
        Value::Int(n) => n.to_string(),
        Value::Func(_) => "<fn>".to_string(),
    }
}

/// Execute one statement against the environment, appending any `print` output to
/// `out`.
pub fn exec(stmt: &Stmt, env: &Env, out: &mut Vec<String>) -> Result<(), String> {
    match stmt {
        Stmt::FnDef(name, params, body) => {
            let f = Value::Func(Rc::new(Function {
                params: params.clone(),
                body: body.clone(),
                env: env.clone(),
            }));
            env.borrow_mut().vars.insert(name.clone(), f);
        }
        Stmt::Assign(name, e) => {
            let v = eval(e, env)?;
            env.borrow_mut().vars.insert(name.clone(), v);
        }
        Stmt::Print(e) => {
            let v = eval(e, env)?;
            out.push(show(&v));
        }
        Stmt::While(cond, body) => {
            while as_int(eval(cond, env)?)? != 0 {
                exec(body, env, out)?;
            }
        }
        Stmt::Block(stmts) => {
            for s in stmts {
                exec(s, env, out)?;
            }
        }
        Stmt::Expr(e) => {
            eval(e, env)?; // evaluated for effect; the value is discarded
        }
    }
    Ok(())
}

/// Run a whole program, returning everything it printed (one string per `print`).
/// A lexing, parsing, or run-time error yields a single `"error: ..."` line.
pub fn run_program(src: &str) -> Vec<String> {
    let env = root_env();
    let mut out = Vec::new();
    let result = tokenize(src).and_then(parse).and_then(|stmts| {
        for s in &stmts {
            exec(s, &env, &mut out)?;
        }
        Ok(())
    });
    if let Err(e) = result {
        out.push(format!("error: {e}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_emits_values() {
        assert_eq!(run_program("print 42; print 7"), vec!["42", "7"]);
    }

    #[test]
    fn while_loop_counts() {
        let out = run_program("i = 1; while i <= 5 do { print i; i = i + 1 }");
        assert_eq!(out, vec!["1", "2", "3", "4", "5"]);
    }

    #[test]
    fn loop_computes_factorial_iteratively() {
        let out = run_program(
            "n = 5; acc = 1; i = 1; while i <= n do { acc = acc * i; i = i + 1 }; print acc",
        );
        assert_eq!(out, vec!["120"]);
    }

    #[test]
    fn nested_loops() {
        let out = run_program(
            "i = 1; while i <= 3 do { j = 1; while j <= 3 do { print i * j; j = j + 1 }; i = i + 1 }",
        );
        assert_eq!(out, vec!["1", "2", "3", "2", "4", "6", "3", "6", "9"]);
    }

    #[test]
    fn loops_and_functions_together() {
        let out = run_program("sq(x) = x * x; i = 1; while i <= 4 do { print sq(i); i = i + 1 }");
        assert_eq!(out, vec!["1", "4", "9", "16"]);
    }

    #[test]
    fn fibonacci_sequence_via_loop() {
        let out = run_program(
            "a = 0; b = 1; i = 0; while i < 8 do { print a; t = a + b; a = b; b = t; i = i + 1 }",
        );
        assert_eq!(out, vec!["0", "1", "1", "2", "3", "5", "8", "13"]);
    }
}
