// Package main — vars: the expression interpreter grown into a tiny language
// with variables and a persistent environment.
//
// Lesson 1 evaluated one expression. Real languages remember things: you name a
// value and use it later. That is a variable, and the thing holding the names is
// an environment — a name->value map threaded through evaluation. Add that and
// the calculator becomes a language. Still integer arithmetic, so results stay
// byte-identical across the three languages. Library and command share one
// package so `go run .` and `go test` both work.
package main

import (
	"fmt"
	"strconv"
	"strings"
	"unicode"
)

// Env is the running memory of the program: variable names to their values.
type Env map[string]int64

// ---------------------------------------------------------------- lexer

// Token: Kind 'n' number (Num), 'i' identifier (Name), '=' assign, else the
// operator/parenthesis character.
type Token struct {
	Kind byte
	Num  int64
	Name string
}

// Tokenize turns one statement's source into tokens, adding identifiers and the
// '=' assignment token to the lexer from lesson 1.
func Tokenize(src string) ([]Token, error) {
	r := []rune(src)
	var tokens []Token
	i := 0
	for i < len(r) {
		c := r[i]
		switch {
		case c == ' ' || c == '\t' || c == '\r' || c == '\n':
			i++
		case c == '=' || c == '+' || c == '-' || c == '*' || c == '/' || c == '(' || c == ')':
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
		case unicode.IsLetter(c) || c == '_':
			start := i
			for i < len(r) && (unicode.IsLetter(r[i]) || unicode.IsDigit(r[i]) || r[i] == '_') {
				i++
			}
			tokens = append(tokens, Token{Kind: 'i', Name: string(r[start:i])})
		default:
			return nil, fmt.Errorf("unexpected character '%c'", c)
		}
	}
	return tokens, nil
}

// ---------------------------------------------------------------- parser

// Expr: Kind 'n' number (Num), 'v' variable (Name), 'u' unary neg (L), else a
// binary operator (L, R).
type Expr struct {
	Kind byte
	Num  int64
	Name string
	L, R *Expr
}

// Stmt: Kind 'a' assignment (Name = Expr) or 'e' bare expression.
type Stmt struct {
	Kind byte
	Name string
	Expr *Expr
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

// factor := Num | Ident | '(' expr ')' | '-' factor
func (p *parser) factor() (*Expr, error) {
	t, ok := p.next()
	if !ok {
		return nil, fmt.Errorf("unexpected end of input")
	}
	switch t.Kind {
	case 'n':
		return &Expr{Kind: 'n', Num: t.Num}, nil
	case 'i':
		return &Expr{Kind: 'v', Name: t.Name}, nil
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

// ParseStmt parses one statement: `name = expr` is an assignment (identifier
// followed by '='); anything else is a bare expression.
func ParseStmt(tokens []Token) (Stmt, error) {
	isAssign := len(tokens) >= 2 && tokens[0].Kind == 'i' && tokens[1].Kind == '='
	p := &parser{tokens: tokens}
	var stmt Stmt
	if isAssign {
		name := tokens[0].Name
		p.pos = 2 // skip identifier and '='
		e, err := p.expr()
		if err != nil {
			return Stmt{}, err
		}
		stmt = Stmt{Kind: 'a', Name: name, Expr: e}
	} else {
		e, err := p.expr()
		if err != nil {
			return Stmt{}, err
		}
		stmt = Stmt{Kind: 'e', Expr: e}
	}
	if p.pos != len(p.tokens) {
		return Stmt{}, fmt.Errorf("unexpected trailing input: '%c'", p.tokens[p.pos].Kind)
	}
	return stmt, nil
}

// ---------------------------------------------------------------- evaluator

// Eval evaluates an expression against the environment; an unbound variable is
// an error.
func Eval(e *Expr, env Env) (int64, error) {
	switch e.Kind {
	case 'n':
		return e.Num, nil
	case 'v':
		v, ok := env[e.Name]
		if !ok {
			return 0, fmt.Errorf("undefined variable '%s'", e.Name)
		}
		return v, nil
	case 'u':
		v, err := Eval(e.L, env)
		return -v, err
	default:
		a, err := Eval(e.L, env)
		if err != nil {
			return 0, err
		}
		b, err := Eval(e.R, env)
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
			return a / b, nil
		}
		return 0, fmt.Errorf("unreachable")
	}
}

// Exec runs one statement, returning its value. An assignment stores the value
// and evaluates to it; a bare expression just evaluates.
func Exec(stmt Stmt, env Env) (int64, error) {
	if stmt.Kind == 'a' {
		v, err := Eval(stmt.Expr, env)
		if err != nil {
			return 0, err
		}
		env[stmt.Name] = v
		return v, nil
	}
	return Eval(stmt.Expr, env)
}

// RunProgram runs a whole program: one statement per non-empty line, sharing a
// single environment so state persists. Returns one "line  =>  value" (or error)
// string per statement.
func RunProgram(src string) []string {
	env := Env{}
	var out []string
	for _, line := range strings.Split(src, "\n") {
		trimmed := strings.TrimSpace(line)
		if trimmed == "" {
			continue
		}
		v, err := runOne(trimmed, env)
		if err != nil {
			out = append(out, fmt.Sprintf("%s  =>  error: %s", trimmed, err))
		} else {
			out = append(out, fmt.Sprintf("%s  =>  %d", trimmed, v))
		}
	}
	return out
}

func runOne(src string, env Env) (int64, error) {
	tokens, err := Tokenize(src)
	if err != nil {
		return 0, err
	}
	stmt, err := ParseStmt(tokens)
	if err != nil {
		return 0, err
	}
	return Exec(stmt, env)
}
