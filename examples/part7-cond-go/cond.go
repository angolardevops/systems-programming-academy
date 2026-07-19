// Package main — cond: the language gains a decision, and with it, recursion.
//
// Lesson 3 gave functions and closures. This lesson adds comparisons
// (< <= > >= == !=) and a lazy `if cond then a else b` expression — only the
// taken branch is evaluated, so recursion can terminate. Booleans are integers
// (1 = true, 0 = false), keeping results byte-identical. Functions plus
// conditionals make the language Turing-complete. Library and command share one
// package so `go run .` and `go test` both work.
package main

import (
	"fmt"
	"strconv"
	"strings"
	"unicode"
)

// ---------------------------------------------------------------- values & env

type Value struct {
	Kind byte // 'i' int, 'f' func
	Int  int64
	Fn   *Function
}

type Function struct {
	Params []string
	Body   *Expr
	Env    *Scope
}

type Scope struct {
	vars   map[string]Value
	parent *Scope
}

func RootEnv() *Scope { return &Scope{vars: map[string]Value{}} }

func childEnv(parent *Scope) *Scope { return &Scope{vars: map[string]Value{}, parent: parent} }

func lookup(env *Scope, name string) (Value, bool) {
	for s := env; s != nil; s = s.parent {
		if v, ok := s.vars[name]; ok {
			return v, true
		}
	}
	return Value{}, false
}

// ---------------------------------------------------------------- lexer

// Token: Kind 'n' number, 'i' identifier, '=' assign, ',' comma, 'C' comparison
// (operator in Op), else the operator/parenthesis character.
type Token struct {
	Kind byte
	Num  int64
	Name string
	Op   string
}

// Tokenize turns one statement's source into tokens, adding comparison operators
// (single- and two-char) to lesson 3's lexer.
func Tokenize(src string) ([]Token, error) {
	r := []rune(src)
	var tokens []Token
	i := 0
	peek := func(k int) rune {
		if i+k < len(r) {
			return r[i+k]
		}
		return 0
	}
	for i < len(r) {
		c := r[i]
		switch {
		case c == ' ' || c == '\t' || c == '\r' || c == '\n':
			i++
		case c == ',' || c == '+' || c == '-' || c == '*' || c == '/' || c == '(' || c == ')':
			tokens = append(tokens, Token{Kind: byte(c)})
			i++
		case c == '=' && peek(1) == '=':
			tokens = append(tokens, Token{Kind: 'C', Op: "=="})
			i += 2
		case c == '=':
			tokens = append(tokens, Token{Kind: '='})
			i++
		case c == '!' && peek(1) == '=':
			tokens = append(tokens, Token{Kind: 'C', Op: "!="})
			i += 2
		case c == '<' && peek(1) == '=':
			tokens = append(tokens, Token{Kind: 'C', Op: "<="})
			i += 2
		case c == '<':
			tokens = append(tokens, Token{Kind: 'C', Op: "<"})
			i++
		case c == '>' && peek(1) == '=':
			tokens = append(tokens, Token{Kind: 'C', Op: ">="})
			i += 2
		case c == '>':
			tokens = append(tokens, Token{Kind: 'C', Op: ">"})
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

// Expr: Kind 'n' number, 'v' variable, 'c' call, 'u' unary neg, 'C' comparison
// (Op, L, R), 'I' if (Cond, Then, Els), else a binary operator (L, R).
type Expr struct {
	Kind            byte
	Num             int64
	Name            string
	Op              string
	Args            []*Expr
	L, R            *Expr
	Cond, Then, Els *Expr
}

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

func (p *parser) isKw(kw string) bool {
	t, ok := p.peek()
	return ok && t.Kind == 'i' && t.Name == kw
}

func (p *parser) expectKw(kw string) error {
	if p.isKw(kw) {
		p.pos++
		return nil
	}
	return fmt.Errorf("expected '%s'", kw)
}

// expr := 'if' expr 'then' expr 'else' expr | comparison
func (p *parser) expr() (*Expr, error) {
	if p.isKw("if") {
		p.pos++
		cond, err := p.expr()
		if err != nil {
			return nil, err
		}
		if err := p.expectKw("then"); err != nil {
			return nil, err
		}
		then, err := p.expr()
		if err != nil {
			return nil, err
		}
		if err := p.expectKw("else"); err != nil {
			return nil, err
		}
		els, err := p.expr()
		if err != nil {
			return nil, err
		}
		return &Expr{Kind: 'I', Cond: cond, Then: then, Els: els}, nil
	}
	return p.comparison()
}

// comparison := add (cmpop add)?
func (p *parser) comparison() (*Expr, error) {
	left, err := p.add()
	if err != nil {
		return nil, err
	}
	if t, ok := p.peek(); ok && t.Kind == 'C' {
		p.pos++
		right, err := p.add()
		if err != nil {
			return nil, err
		}
		return &Expr{Kind: 'C', Op: t.Op, L: left, R: right}, nil
	}
	return left, nil
}

func (p *parser) add() (*Expr, error) {
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
			p.pos++
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
	i := 2
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
	i += 2
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
	case 'C':
		return evalCmp(e, env)
	case 'I':
		c, err := Eval(e.Cond, env)
		if err != nil {
			return Value{}, err
		}
		n, err := asInt(c)
		if err != nil {
			return Value{}, err
		}
		if n != 0 { // lazy: only the taken branch
			return Eval(e.Then, env)
		}
		return Eval(e.Els, env)
	case 'c':
		return evalCall(e, env)
	default:
		return evalBin(e, env)
	}
}

func evalBin(e *Expr, env *Scope) (Value, error) {
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

func evalCmp(e *Expr, env *Scope) (Value, error) {
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
	var t bool
	switch e.Op {
	case "<":
		t = a < b
	case "<=":
		t = a <= b
	case ">":
		t = a > b
	case ">=":
		t = a >= b
	case "==":
		t = a == b
	case "!=":
		t = a != b
	}
	if t {
		return Value{Kind: 'i', Int: 1}, nil // booleans are integers
	}
	return Value{Kind: 'i', Int: 0}, nil
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
	argv := make([]Value, len(e.Args))
	for i, a := range e.Args {
		av, err := Eval(a, env)
		if err != nil {
			return Value{}, err
		}
		argv[i] = av
	}
	call := childEnv(fn.Env)
	for i, p := range fn.Params {
		call.vars[p] = argv[i]
	}
	return Eval(fn.Body, call)
}

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
