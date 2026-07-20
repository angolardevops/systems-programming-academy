//! boollang — the language gains short-circuit boolean logic.
//!
//! Lesson 4 made the language Turing-complete with conditionals. This lesson adds
//! the boolean operators that make conditions *expressive*: `and`, `or`, and
//! `not`. Their defining feature is **short-circuit evaluation** — the same
//! laziness as `if`: `and` skips its right side when the left is false, `or` skips
//! it when the left is true. That lets a guard like
//!
//! ```text
//!   guard(x) = if x != 0 and 100 / x > 1 then 100 / x else -1
//! ```
//!
//! call `guard(0)` safely — `100 / x` is never reached. Booleans are integers
//! (1 = true, 0 = false, any nonzero is truthy). The operators are keywords the
//! parser recognizes by name, sitting below comparison in precedence:
//! `or` < `and` < `not` < comparison.

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
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
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
        self.or_expr()
    }

    // or_expr := and_expr ('or' and_expr)*    — loosest boolean level
    fn or_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.and_expr()?;
        while self.is_kw("or") {
            self.pos += 1;
            left = Expr::Or(Box::new(left), Box::new(self.and_expr()?));
        }
        Ok(left)
    }

    // and_expr := not_expr ('and' not_expr)*
    fn and_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.not_expr()?;
        while self.is_kw("and") {
            self.pos += 1;
            left = Expr::And(Box::new(left), Box::new(self.not_expr()?));
        }
        Ok(left)
    }

    // not_expr := 'not' not_expr | comparison   — 'not' binds tighter than and/or
    fn not_expr(&mut self) -> Result<Expr, String> {
        if self.is_kw("not") {
            self.pos += 1;
            return Ok(Expr::Not(Box::new(self.not_expr()?)));
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
        Expr::And(l, r) => {
            // Short-circuit: if the left is falsy, never evaluate the right.
            if as_int(eval(l, env)?)? == 0 {
                Ok(Value::Int(0))
            } else {
                Ok(Value::Int(if as_int(eval(r, env)?)? != 0 { 1 } else { 0 }))
            }
        }
        Expr::Or(l, r) => {
            // Short-circuit: if the left is truthy, never evaluate the right.
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
    fn and_or_truth_tables() {
        assert_eq!(run1("1 and 1"), "1 and 1  =>  1");
        assert_eq!(run1("1 and 0"), "1 and 0  =>  0");
        assert_eq!(run1("0 and 1"), "0 and 1  =>  0");
        assert_eq!(run1("0 or 0"), "0 or 0  =>  0");
        assert_eq!(run1("1 or 0"), "1 or 0  =>  1");
    }

    #[test]
    fn not_negates_truthiness() {
        assert_eq!(run1("not 0"), "not 0  =>  1");
        assert_eq!(run1("not 5"), "not 5  =>  0"); // any nonzero is truthy
        assert_eq!(run1("not not 3"), "not not 3  =>  1");
    }

    #[test]
    fn precedence_not_tighter_than_and_tighter_than_or() {
        // not binds tightest, then and, then or:
        // 1 or 0 and 0  ==  1 or (0 and 0)  ==  1
        assert_eq!(run1("1 or 0 and 0"), "1 or 0 and 0  =>  1");
        // not 0 and 1  ==  (not 0) and 1  ==  1
        assert_eq!(run1("not 0 and 1"), "not 0 and 1  =>  1");
        // comparison binds tighter than the booleans:
        // 2 > 1 and 3 > 5  ==  (2 > 1) and (3 > 5)  ==  0
        assert_eq!(run1("2 > 1 and 3 > 5"), "2 > 1 and 3 > 5  =>  0");
    }

    #[test]
    fn and_short_circuits_avoiding_the_error() {
        // 100 / x is never evaluated when x == 0, because `and`'s left is false.
        let out = run_program(
            "guard(x) = if x != 0 and 100 / x > 1 then 100 / x else -1\nguard(0)\nguard(50)",
        );
        assert_eq!(out[1], "guard(0)  =>  -1"); // no division-by-zero
        assert_eq!(out[2], "guard(50)  =>  2");
    }

    #[test]
    fn or_short_circuits_avoiding_the_error() {
        // 10 / a is never evaluated when a == 0, because `or`'s left is true.
        let out = run_program("check(a) = if a == 0 or 10 / a > 0 then 1 else 0\ncheck(0)");
        assert_eq!(out.last().unwrap(), "check(0)  =>  1"); // no division-by-zero
    }

    #[test]
    fn booleans_compose_into_real_predicates() {
        let out = run_program(
            "in_range(x, lo, hi) = if x >= lo and x <= hi then 1 else 0\nin_range(5, 1, 10)\nin_range(15, 1, 10)",
        );
        assert_eq!(out[1], "in_range(5, 1, 10)  =>  1");
        assert_eq!(out[2], "in_range(15, 1, 10)  =>  0");
    }
}
