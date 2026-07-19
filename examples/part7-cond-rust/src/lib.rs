//! cond — the language gains a decision, and with it, recursion.
//!
//! Lesson 3 gave the language functions and closures, but every program still ran
//! straight to the end. This lesson adds the one construct that makes a language
//! *compute anything*: a conditional. With comparisons (`< <= > >= == !=`) and a
//! lazy `if cond then a else b` — where only the taken branch is evaluated — a
//! function can finally stop recursing:
//!
//! ```text
//!   fact(n) = if n <= 1 then 1 else n * fact(n - 1)
//!   fact(5)   # => 120
//! ```
//!
//! Booleans are integers here (1 = true, 0 = false), keeping results exact and
//! byte-identical. With functions plus conditionals the language is now
//! Turing-complete.

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
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Cmp(&'static str), // one of  <  <=  >  >=  ==  !=
}

/// Tokenize one statement, adding comparison operators (single- and two-char) to
/// lesson 3's lexer.
pub fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = src.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match c {
            ' ' | '\t' | '\r' | '\n' => i += 1,
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

// ---------------------------------------------------------------- parser

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Num(i64),
    Var(String),
    Call(String, Vec<Expr>),
    Neg(Box<Expr>),
    Bin(char, Box<Expr>, Box<Expr>),
    Cmp(&'static str, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
}

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

    // expr := 'if' expr 'then' expr 'else' expr | comparison
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
        self.comparison()
    }

    // comparison := add (cmpop add)?   — a single, non-associative comparison
    fn comparison(&mut self) -> Result<Expr, String> {
        let left = self.add()?;
        if let Some(Token::Cmp(op)) = self.peek() {
            let op = *op;
            self.pos += 1;
            let right = self.add()?;
            return Ok(Expr::Cmp(op, Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    // add := term (('+' | '-') term)*
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

    // term := factor (('*' | '/') factor)*
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
        p.pos += 1;
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
    let mut params = Vec::new();
    let mut i = 2;
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
    i += 1;
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
            Ok(Value::Int(if t { 1 } else { 0 })) // booleans are integers
        }
        Expr::If(cond, then, els) => {
            // Lazy: evaluate the condition, then ONLY the taken branch.
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

pub fn exec(stmt: &Stmt, env: &Env) -> Result<Value, String> {
    match stmt {
        Stmt::FnDef(name, params, body) => {
            let f = Value::Func(Rc::new(Function {
                params: params.clone(),
                body: body.clone(),
                env: env.clone(),
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

    fn run1(src: &str) -> String {
        let out = run_program(src);
        out.last().unwrap().clone()
    }

    #[test]
    fn tokenizes_comparison_operators() {
        assert_eq!(
            tokenize("a <= b == c != d").unwrap(),
            vec![
                Token::Ident("a".into()),
                Token::Cmp("<="),
                Token::Ident("b".into()),
                Token::Cmp("=="),
                Token::Ident("c".into()),
                Token::Cmp("!="),
                Token::Ident("d".into()),
            ]
        );
    }

    #[test]
    fn comparisons_yield_one_or_zero() {
        assert_eq!(run1("3 < 5"), "3 < 5  =>  1");
        assert_eq!(run1("3 > 5"), "3 > 5  =>  0");
        assert_eq!(run1("4 == 4"), "4 == 4  =>  1");
        assert_eq!(run1("4 != 4"), "4 != 4  =>  0");
        assert_eq!(run1("5 >= 5"), "5 >= 5  =>  1");
    }

    #[test]
    fn if_selects_the_taken_branch() {
        assert_eq!(run1("if 1 then 10 else 20"), "if 1 then 10 else 20  =>  10");
        assert_eq!(run1("if 0 then 10 else 20"), "if 0 then 10 else 20  =>  20");
        assert_eq!(
            run1("if 3 < 5 then 100 else 200"),
            "if 3 < 5 then 100 else 200  =>  100"
        );
    }

    #[test]
    fn recursion_now_terminates() {
        let out = run_program("fact(n) = if n <= 1 then 1 else n * fact(n - 1)\nfact(5)");
        assert_eq!(out.last().unwrap(), "fact(5)  =>  120");
    }

    #[test]
    fn recursive_fibonacci() {
        let out = run_program("fib(n) = if n < 2 then n else fib(n - 1) + fib(n - 2)\nfib(10)");
        assert_eq!(out.last().unwrap(), "fib(10)  =>  55");
    }

    #[test]
    fn only_the_taken_branch_is_evaluated() {
        // The else branch would divide by zero, but it is never taken.
        let out = run_program("safe(n) = if n == 0 then 0 else 100 / n\nsafe(0)\nsafe(4)");
        assert_eq!(out[1], "safe(0)  =>  0"); // no division-by-zero error
        assert_eq!(out[2], "safe(4)  =>  25");
    }
}
