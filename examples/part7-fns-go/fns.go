// Package main — fns: the small language grows functions and closures.
//
// Lesson 2 gave the language memory (an environment). This lesson gives it
// abstraction: named functions you define and call. A call creates a fresh scope
// for the arguments, and a function captures the environment where it was defined
// (a closure); its call scope chains to that captured one, so it sees the
// variables in scope where it was written — lexical scoping. Environments form a
// parent chain. Values are now integers or functions. Library and command share
// one package so `go run .` and `go test` both work.
package main

import (
	"fmt"
	"strconv"
	"strings"
	"unicode"
)

// ---------------------------------------------------------------- values & env

// Value is a runtime value: an integer (Kind 'i') or a function (Kind 'f').
type Value struct {
	Kind byte
	Int  int64
	Fn   *Function
}

// Function captures its parameters, body, and the environment where it was
// defined (its closure).
type Function struct {
	Params []string
	Body   *Expr
	Env    *Scope
}

// Scope is one lexical scope: its own bindings plus an optional parent to fall
// through to. The chain of parents is what makes scoping lexical.
type Scope struct {
	vars   map[string]Value
	parent *Scope
}

// RootEnv returns a fresh root environment.
func RootEnv() *Scope { return &Scope{vars: map[string]Value{}} }

func childEnv(parent *Scope) *Scope {
	return &Scope{vars: map[string]Value{}, parent: parent}
}

// lookup walks from the innermost scope outward — the essence of lexical scoping.
func lookup(env *Scope, name string) (Value, bool) {
	for s := env; s != nil; s = s.parent {
		if v, ok := s.vars[name]; ok {
			return v, true
		}
	}
	return Value{}, false
}

// ---------------------------------------------------------------- lexer

// Token: Kind 'n' number (Num), 'i' identifier (Name), '=' assign, ',' comma,
// else the operator/parenthesis character.
type Token struct {
	Kind byte
	Num  int64
	Name string
}

// Tokenize turns one statement's source into tokens, adding the comma (for
// argument lists) to lesson 2's lexer.
func Tokenize(src string) ([]Token, error) {
	r := []rune(src)
	var tokens []Token
	i := 0
	for i < len(r) {
		c := r[i]
		switch {
		case c == ' ' || c == '\t' || c == '\r' || c == '\n':
			i++
		case c == '=' || c == ',' || c == '+' || c == '-' || c == '*' || c == '/' || c == '(' || c == ')':
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

// Expr: Kind 'n' number (Num), 'v' variable (Name), 'c' call (Name, Args), 'u'
// unary neg (L), else a binary operator (L, R).
type Expr struct {
	Kind byte
	Num  int64
	Name string
	Args []*Expr
	L, R *Expr
}

// Stmt: Kind 'f' function definition (Name, Params, Expr), 'a' assignment (Name =
// Expr), or 'e' bare expression.
type Stmt struct {
	Kind   byte
	Name   string
	Params []string
	Expr   *Expr
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

// factor := Num | Ident '(' args ')' | Ident | '(' expr ')' | '-' factor
func (p *parser) factor() (*Expr, error) {
	t, ok := p.next()
	if !ok {
		return nil, fmt.Errorf("unexpected end of input")
	}
	switch t.Kind {
	case 'n':
		return &Expr{Kind: 'n', Num: t.Num}, nil
	case 'i':
		if nt, ok := p.peek(); ok && nt.Kind == '(' {
			p.pos++ // consume '('
			args, err := p.args()
			if err != nil {
				return nil, err
			}
			return &Expr{Kind: 'c', Name: t.Name, Args: args}, nil
		}
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

// args := (expr (',' expr)*)? ')'  — the '(' is already consumed
func (p *parser) args() ([]*Expr, error) {
	var args []*Expr
	if t, ok := p.peek(); ok && t.Kind == ')' {
		p.pos++
		return args, nil
	}
	for {
		a, err := p.expr()
		if err != nil {
			return nil, err
		}
		args = append(args, a)
		t, ok := p.next()
		if !ok {
			return nil, fmt.Errorf("expected ',' or ')' in argument list")
		}
		if t.Kind == ',' {
			continue
		}
		if t.Kind == ')' {
			return args, nil
		}
		return nil, fmt.Errorf("expected ',' or ')' in argument list")
	}
}

func isFnDef(tokens []Token) bool {
	if len(tokens) < 2 || tokens[0].Kind != 'i' || tokens[1].Kind != '(' {
		return false
	}
	depth := 0
	for i := 1; i < len(tokens); i++ {
		switch tokens[i].Kind {
		case '(':
			depth++
		case ')':
			depth--
			if depth == 0 {
				return i+1 < len(tokens) && tokens[i+1].Kind == '='
			}
		}
	}
	return false
}

// ParseStmt parses one statement: a function definition (`name(params) = body`),
// an assignment (`name = expr`), or a bare expression.
func ParseStmt(tokens []Token) (Stmt, error) {
	if isFnDef(tokens) {
		return parseFnDef(tokens)
	}
	isAssign := len(tokens) >= 2 && tokens[0].Kind == 'i' && tokens[1].Kind == '='
	p := &parser{tokens: tokens}
	if isAssign {
		name := tokens[0].Name
		p.pos = 2
		e, err := p.expr()
		if err != nil {
			return Stmt{}, err
		}
		if p.pos != len(p.tokens) {
			return Stmt{}, fmt.Errorf("unexpected trailing input: '%c'", p.tokens[p.pos].Kind)
		}
		return Stmt{Kind: 'a', Name: name, Expr: e}, nil
	}
	e, err := p.expr()
	if err != nil {
		return Stmt{}, err
	}
	if p.pos != len(p.tokens) {
		return Stmt{}, fmt.Errorf("unexpected trailing input: '%c'", p.tokens[p.pos].Kind)
	}
	return Stmt{Kind: 'e', Expr: e}, nil
}

func parseFnDef(tokens []Token) (Stmt, error) {
	name := tokens[0].Name
	var params []string
	i := 2 // skip name and '('
	if tokens[i].Kind != ')' {
		for {
			if tokens[i].Kind != 'i' {
				return Stmt{}, fmt.Errorf("expected a parameter name")
			}
			params = append(params, tokens[i].Name)
			i++
			if tokens[i].Kind == ',' {
				i++
				continue
			}
			if tokens[i].Kind == ')' {
				break
			}
			return Stmt{}, fmt.Errorf("expected ',' or ')' in parameter list")
		}
	}
	i += 2 // skip ')' and '='
	p := &parser{tokens: tokens[i:]}
	body, err := p.expr()
	if err != nil {
		return Stmt{}, err
	}
	if p.pos != len(p.tokens) {
		return Stmt{}, fmt.Errorf("unexpected trailing input in function body")
	}
	return Stmt{Kind: 'f', Name: name, Params: params, Expr: body}, nil
}

// ---------------------------------------------------------------- evaluator

func asInt(v Value) (int64, error) {
	if v.Kind == 'i' {
		return v.Int, nil
	}
	return 0, fmt.Errorf("cannot do arithmetic on a function")
}

// Eval evaluates an expression to a value in the given environment.
func Eval(e *Expr, env *Scope) (Value, error) {
	switch e.Kind {
	case 'n':
		return Value{Kind: 'i', Int: e.Num}, nil
	case 'v':
		v, ok := lookup(env, e.Name)
		if !ok {
			return Value{}, fmt.Errorf("undefined variable '%s'", e.Name)
		}
		return v, nil
	case 'u':
		v, err := Eval(e.L, env)
		if err != nil {
			return Value{}, err
		}
		n, err := asInt(v)
		return Value{Kind: 'i', Int: -n}, err
	case 'c':
		return evalCall(e, env)
	default:
		av, err := Eval(e.L, env)
		if err != nil {
			return Value{}, err
		}
		bv, err := Eval(e.R, env)
		if err != nil {
			return Value{}, err
		}
		a, err := asInt(av)
		if err != nil {
			return Value{}, err
		}
		b, err := asInt(bv)
		if err != nil {
			return Value{}, err
		}
		switch e.Kind {
		case '+':
			return Value{Kind: 'i', Int: a + b}, nil
		case '-':
			return Value{Kind: 'i', Int: a - b}, nil
		case '*':
			return Value{Kind: 'i', Int: a * b}, nil
		case '/':
			if b == 0 {
				return Value{}, fmt.Errorf("division by zero")
			}
			return Value{Kind: 'i', Int: a / b}, nil
		}
		return Value{}, fmt.Errorf("unreachable")
	}
}

func evalCall(e *Expr, env *Scope) (Value, error) {
	v, ok := lookup(env, e.Name)
	if !ok {
		return Value{}, fmt.Errorf("undefined function '%s'", e.Name)
	}
	if v.Kind != 'f' {
		return Value{}, fmt.Errorf("'%s' is not a function", e.Name)
	}
	fn := v.Fn
	if len(e.Args) != len(fn.Params) {
		return Value{}, fmt.Errorf("'%s' expects %d argument(s), got %d", e.Name, len(fn.Params), len(e.Args))
	}
	// Evaluate arguments in the CALLER's environment...
	argv := make([]Value, len(e.Args))
	for i, a := range e.Args {
		av, err := Eval(a, env)
		if err != nil {
			return Value{}, err
		}
		argv[i] = av
	}
	// ...but run the body in a new scope chained to the function's DEFINING
	// environment (its closure) — this is lexical scoping.
	call := childEnv(fn.Env)
	for i, p := range fn.Params {
		call.vars[p] = argv[i]
	}
	return Eval(fn.Body, call)
}

// Exec runs one statement, returning its value. A definition builds a closure; an
// assignment stores a value; a bare expression is evaluated.
func Exec(stmt Stmt, env *Scope) (Value, error) {
	switch stmt.Kind {
	case 'f':
		f := Value{Kind: 'f', Fn: &Function{Params: stmt.Params, Body: stmt.Expr, Env: env}}
		env.vars[stmt.Name] = f
		return f, nil
	case 'a':
		v, err := Eval(stmt.Expr, env)
		if err != nil {
			return Value{}, err
		}
		env.vars[stmt.Name] = v
		return v, nil
	default:
		return Eval(stmt.Expr, env)
	}
}

func show(v Value) string {
	if v.Kind == 'i' {
		return strconv.FormatInt(v.Int, 10)
	}
	return "<fn>"
}

// RunProgram runs a whole program: one statement per non-empty line, sharing one
// root environment. Returns one "line  =>  value" (or error) string per statement.
func RunProgram(src string) []string {
	env := RootEnv()
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
			out = append(out, fmt.Sprintf("%s  =>  %s", trimmed, show(v)))
		}
	}
	return out
}

func runOne(src string, env *Scope) (Value, error) {
	tokens, err := Tokenize(src)
	if err != nil {
		return Value{}, err
	}
	stmt, err := ParseStmt(tokens)
	if err != nil {
		return Value{}, err
	}
	return Exec(stmt, env)
}
