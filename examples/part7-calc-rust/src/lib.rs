//! calc — a tiny interpreter for integer arithmetic, in three stages.
//!
//! Every interpreter, from this one to a full language, is the same pipeline:
//!
//!   source text  --lexer-->  tokens  --parser-->  a syntax tree  --eval-->  a value
//!
//! Here the language is integer arithmetic with `+ - * /`, parentheses, and unary
//! minus. `/` is integer division truncated toward zero. Keeping it to integers
//! means every result is exact and identical across languages — the lexer, the
//! parser, and the evaluator are the lesson, not floating-point formatting.

// ---------------------------------------------------------------- lexer

/// A lexical token — the smallest meaningful piece of the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Num(i64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

/// Turn source text into a flat list of tokens. Whitespace is skipped; any
/// character that isn't a digit, operator, or parenthesis is an error.
pub fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\n' | '\r' => i += 1,
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
            _ => return Err(format!("unexpected character '{c}'")),
        }
    }
    Ok(tokens)
}

// ---------------------------------------------------------------- parser

/// A node in the abstract syntax tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Num(i64),
    Neg(Box<Expr>),
    Bin(char, Box<Expr>, Box<Expr>),
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

    // expr := term (('+' | '-') term)*
    fn expr(&mut self) -> Result<Expr, String> {
        let mut left = self.term()?;
        while let Some(op) = match self.peek() {
            Some(Token::Plus) => Some('+'),
            Some(Token::Minus) => Some('-'),
            _ => None,
        } {
            self.pos += 1;
            let right = self.term()?;
            left = Expr::Bin(op, Box::new(left), Box::new(right));
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
            let right = self.factor()?;
            left = Expr::Bin(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    // factor := Num | '(' expr ')' | '-' factor
    fn factor(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Token::Num(n)) => Ok(Expr::Num(n)),
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

/// Parse a token list into a syntax tree, enforcing operator precedence
/// (`*` and `/` bind tighter than `+` and `-`) and rejecting trailing garbage.
pub fn parse(tokens: Vec<Token>) -> Result<Expr, String> {
    let mut p = Parser { tokens, pos: 0 };
    let e = p.expr()?;
    if p.pos != p.tokens.len() {
        return Err(format!("unexpected trailing input: {:?}", p.tokens[p.pos]));
    }
    Ok(e)
}

// ---------------------------------------------------------------- evaluator

/// Render a syntax tree as a fully-parenthesised S-expression — the exact shape
/// the parser built, so precedence is visible. `1 + 2 * 3` becomes
/// `(+ 1 (* 2 3))`.
pub fn to_sexp(e: &Expr) -> String {
    match e {
        Expr::Num(n) => n.to_string(),
        Expr::Neg(inner) => format!("(neg {})", to_sexp(inner)),
        Expr::Bin(op, l, r) => format!("({op} {} {})", to_sexp(l), to_sexp(r)),
    }
}

/// Walk the tree and compute its value. Division is integer division truncated
/// toward zero; dividing by zero is an error.
pub fn eval(e: &Expr) -> Result<i64, String> {
    match e {
        Expr::Num(n) => Ok(*n),
        Expr::Neg(inner) => Ok(-eval(inner)?),
        Expr::Bin(op, l, r) => {
            let a = eval(l)?;
            let b = eval(r)?;
            match op {
                '+' => Ok(a + b),
                '-' => Ok(a - b),
                '*' => Ok(a * b),
                '/' => {
                    if b == 0 {
                        Err("division by zero".to_string())
                    } else {
                        Ok(a / b) // Rust's i64 `/` truncates toward zero
                    }
                }
                _ => unreachable!("parser only produces + - * /"),
            }
        }
    }
}

/// The whole pipeline: source text to `(s-expression, value)`, or the first
/// error encountered.
pub fn run(src: &str) -> Result<(String, i64), String> {
    let ast = parse(tokenize(src)?)?;
    Ok((to_sexp(&ast), eval(&ast)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_numbers_and_operators() {
        assert_eq!(
            tokenize("12 + 3").unwrap(),
            vec![Token::Num(12), Token::Plus, Token::Num(3)]
        );
    }

    #[test]
    fn precedence_binds_star_tighter_than_plus() {
        let (sexp, value) = run("1 + 2 * 3").unwrap();
        assert_eq!(sexp, "(+ 1 (* 2 3))");
        assert_eq!(value, 7);
    }

    #[test]
    fn parentheses_override_precedence() {
        let (sexp, value) = run("(1 + 2) * 3").unwrap();
        assert_eq!(sexp, "(* (+ 1 2) 3)");
        assert_eq!(value, 9);
    }

    #[test]
    fn unary_minus_and_truncating_division() {
        let (sexp, value) = run("-7 / 2").unwrap();
        assert_eq!(sexp, "(/ (neg 7) 2)");
        assert_eq!(value, -3); // truncates toward zero, not floor(-3.5) = -4
    }

    #[test]
    fn evaluates_a_longer_expression() {
        let (sexp, value) = run("2 * (3 + 4) - 10 / 3").unwrap();
        assert_eq!(sexp, "(- (* 2 (+ 3 4)) (/ 10 3))");
        assert_eq!(value, 11); // 10/3 = 3, 2*7 = 14, 14 - 3 = 11
    }

    #[test]
    fn reports_errors_without_panicking() {
        assert!(run("1 / 0").unwrap_err().contains("division by zero"));
        assert!(run("1 +").unwrap_err().contains("unexpected end of input"));
        assert!(run("1 @ 2").unwrap_err().contains("unexpected character"));
        assert!(run("(1 + 2").unwrap_err().contains("expected ')'"));
        assert!(run("1 2").unwrap_err().contains("trailing"));
    }
}
