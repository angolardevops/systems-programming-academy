// Package main — calc: a tiny interpreter for integer arithmetic, in three
// stages.
//
// Every interpreter is the same pipeline:
//
//	source text  --lexer-->  tokens  --parser-->  a syntax tree  --eval-->  a value
//
// The language is integer arithmetic with + - * /, parentheses, and unary minus.
// `/` is integer division truncated toward zero. Integers keep every result
// exact and identical across languages — the lexer, parser, and evaluator are the
// lesson. Library and command share one package so `go run .` and `go test` work.
package main

import (
	"fmt"
	"strconv"
)

// ---------------------------------------------------------------- lexer

// Token is the smallest meaningful piece of source. Kind is 'n' for a number
// (its value in Num), otherwise the operator/parenthesis character itself.
type Token struct {
	Kind byte
	Num  int64
}

// Tokenize turns source text into a flat list of tokens. Whitespace is skipped;
// any character that isn't a digit, operator, or parenthesis is an error.
func Tokenize(src string) ([]Token, error) {
	var tokens []Token
	r := []rune(src)
	i := 0
	for i < len(r) {
		c := r[i]
		switch {
		case c == ' ' || c == '\t' || c == '\n' || c == '\r':
			i++
		case c == '+' || c == '-' || c == '*' || c == '/' || c == '(' || c == ')':
			tokens = append(tokens, Token{Kind: byte(c)})
			i++
		case c >= '0' && c <= '9':
			start := i
			for i < len(r) && r[i] >= '0' && r[i] <= '9' {
				i++
			}
			n, err := strconv.ParseInt(string(r[start:i]), 10, 64)
			if err != nil {
				return nil, fmt.Errorf("number too large: %s", string(r[start:i]))
			}
			tokens = append(tokens, Token{Kind: 'n', Num: n})
		default:
			return nil, fmt.Errorf("unexpected character '%c'", c)
		}
	}
	return tokens, nil
}

// ---------------------------------------------------------------- parser

// Expr is a node in the abstract syntax tree. Kind is 'n' for a number, 'u' for
// unary negation (L set), otherwise a binary operator (L and R set).
type Expr struct {
	Kind byte
	Num  int64
	L, R *Expr
}

type parser struct {
	tokens []Token
	pos    int
}

func (p *parser) peek() (Token, bool) {
	if p.pos < len(p.tokens) {
		return p.tokens[p.pos], true
	}
	return Token{}, false
}

func (p *parser) next() (Token, bool) {
	t, ok := p.peek()
	if ok {
		p.pos++
	}
	return t, ok
}

// expr := term (('+' | '-') term)*
func (p *parser) expr() (*Expr, error) {
	left, err := p.term()
	if err != nil {
		return nil, err
	}
	for {
		t, ok := p.peek()
		if !ok || (t.Kind != '+' && t.Kind != '-') {
			return left, nil
		}
		p.pos++
		right, err := p.term()
		if err != nil {
			return nil, err
		}
		left = &Expr{Kind: t.Kind, L: left, R: right}
	}
}

// term := factor (('*' | '/') factor)*
func (p *parser) term() (*Expr, error) {
	left, err := p.factor()
	if err != nil {
		return nil, err
	}
	for {
		t, ok := p.peek()
		if !ok || (t.Kind != '*' && t.Kind != '/') {
			return left, nil
		}
		p.pos++
		right, err := p.factor()
		if err != nil {
			return nil, err
		}
		left = &Expr{Kind: t.Kind, L: left, R: right}
	}
}

// factor := Num | '(' expr ')' | '-' factor
func (p *parser) factor() (*Expr, error) {
	t, ok := p.next()
	if !ok {
		return nil, fmt.Errorf("unexpected end of input")
	}
	switch t.Kind {
	case 'n':
		return &Expr{Kind: 'n', Num: t.Num}, nil
	case '-':
		inner, err := p.factor()
		if err != nil {
			return nil, err
		}
		return &Expr{Kind: 'u', L: inner}, nil
	case '(':
		inner, err := p.expr()
		if err != nil {
			return nil, err
		}
		if rp, ok := p.next(); !ok || rp.Kind != ')' {
			return nil, fmt.Errorf("expected ')'")
		}
		return inner, nil
	default:
		return nil, fmt.Errorf("unexpected token '%c'", t.Kind)
	}
}

// Parse turns a token list into a syntax tree, enforcing precedence (* and /
// bind tighter than + and -) and rejecting trailing garbage.
func Parse(tokens []Token) (*Expr, error) {
	p := &parser{tokens: tokens}
	e, err := p.expr()
	if err != nil {
		return nil, err
	}
	if p.pos != len(p.tokens) {
		return nil, fmt.Errorf("unexpected trailing input: '%c'", p.tokens[p.pos].Kind)
	}
	return e, nil
}

// ---------------------------------------------------------------- evaluator

// ToSexp renders a syntax tree as a fully-parenthesised S-expression, so
// precedence is visible: `1 + 2 * 3` becomes `(+ 1 (* 2 3))`.
func ToSexp(e *Expr) string {
	switch e.Kind {
	case 'n':
		return strconv.FormatInt(e.Num, 10)
	case 'u':
		return fmt.Sprintf("(neg %s)", ToSexp(e.L))
	default:
		return fmt.Sprintf("(%c %s %s)", e.Kind, ToSexp(e.L), ToSexp(e.R))
	}
}

// Eval walks the tree and computes its value. Division is integer division
// truncated toward zero; dividing by zero is an error.
func Eval(e *Expr) (int64, error) {
	switch e.Kind {
	case 'n':
		return e.Num, nil
	case 'u':
		v, err := Eval(e.L)
		return -v, err
	default:
		a, err := Eval(e.L)
		if err != nil {
			return 0, err
		}
		b, err := Eval(e.R)
		if err != nil {
			return 0, err
		}
		switch e.Kind {
		case '+':
			return a + b, nil
		case '-':
			return a - b, nil
		case '*':
			return a * b, nil
		case '/':
			if b == 0 {
				return 0, fmt.Errorf("division by zero")
			}
			return a / b, nil // Go's integer / truncates toward zero
		}
		return 0, fmt.Errorf("unreachable")
	}
}

// Run is the whole pipeline: source text to (s-expression, value), or the first
// error.
func Run(src string) (string, int64, error) {
	tokens, err := Tokenize(src)
	if err != nil {
		return "", 0, err
	}
	ast, err := Parse(tokens)
	if err != nil {
		return "", 0, err
	}
	v, err := Eval(ast)
	if err != nil {
		return "", 0, err
	}
	return ToSexp(ast), v, nil
}
