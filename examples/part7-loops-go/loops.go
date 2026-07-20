// Package main — loops: the interpreter becomes imperative: statements, print,
// and while.
//
// Until now every line was one expression that echoed its value. This lesson
// makes the language *do* things: a program is a sequence of statements separated
// by `;`, grouped with `{ }` blocks; `print` emits output; and `while cond do
// body` loops. The printed output is what the three implementations verify
// byte-for-byte. Library and command share one package so `go run .` and
// `go test` both work.
package main

import (
	"fmt"
	"strconv"
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

// Token: Kind 'n' number, 'i' identifier, '=' assign, ',' comma, ';' semicolon,
// '{'/'}' braces, 'C' comparison (Op), else the operator/parenthesis character.
type Token struct {
	Kind byte
	Num  int64
	Name string
	Op   string
}

// Tokenize the whole program. Newlines are whitespace; statements are separated
// by ';' and grouped by '{ }'.
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
		case c == ';' || c == '{' || c == '}' || c == ',' || c == '+' || c == '-' || c == '*' || c == '/' || c == '(' || c == ')':
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

// ---------------------------------------------------------------- ast

// Expr: Kind 'n' number, 'v' variable, 'c' call, 'u' unary neg, 'C' comparison
// (Op), 'A'/'O'/'N' and/or/not, 'I' if (Cond,Then,Els), else a binary operator.
type Expr struct {
	Kind            byte
	Num             int64
	Name            string
	Op              string
	Args            []*Expr
	L, R            *Expr
	Cond, Then, Els *Expr
}

// Stmt: Kind 'f' fndef, 'a' assign, 'p' print, 'w' while, 'b' block, 'e' expr.
type Stmt struct {
	Kind   byte
	Name   string
	Params []string
	Expr   *Expr // assign/fndef body/print/expr; while condition
	Body   *Stmt // while body
	Stmts  []Stmt
}

// ---------------------------------------------------------------- parser

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

func (p *parser) at(k int) (Token, bool) {
	if p.pos+k < len(p.tokens) {
		return p.tokens[p.pos+k], true
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

func (p *parser) kindIs(k byte) bool {
	t, ok := p.peek()
	return ok && t.Kind == k
}

// program := stmt (';' stmt)* ';'?
func (p *parser) program() ([]Stmt, error) {
	var stmts []Stmt
	for {
		for p.kindIs(';') {
			p.pos++
		}
		if _, ok := p.peek(); !ok {
			return stmts, nil
		}
		s, err := p.stmt()
		if err != nil {
			return nil, err
		}
		stmts = append(stmts, s)
		if t, ok := p.peek(); ok && t.Kind != ';' {
			return nil, fmt.Errorf("expected ';' between statements")
		}
	}
}

func (p *parser) stmt() (Stmt, error) {
	if p.isKw("print") {
		p.pos++
		e, err := p.expr()
		if err != nil {
			return Stmt{}, err
		}
		return Stmt{Kind: 'p', Expr: e}, nil
	}
	if p.isKw("while") {
		p.pos++
		cond, err := p.expr()
		if err != nil {
			return Stmt{}, err
		}
		if err := p.expectKw("do"); err != nil {
			return Stmt{}, err
		}
		body, err := p.stmt()
		if err != nil {
			return Stmt{}, err
		}
		return Stmt{Kind: 'w', Expr: cond, Body: &body}, nil
	}
	if p.kindIs('{') {
		p.pos++
		var stmts []Stmt
		for {
			for p.kindIs(';') {
				p.pos++
			}
			if p.kindIs('}') {
				p.pos++
				break
			}
			if _, ok := p.peek(); !ok {
				return Stmt{}, fmt.Errorf("unclosed '{'")
			}
			s, err := p.stmt()
			if err != nil {
				return Stmt{}, err
			}
			stmts = append(stmts, s)
			if t, ok := p.peek(); ok && t.Kind != ';' && t.Kind != '}' {
				return Stmt{}, fmt.Errorf("expected ';' or '}}'")
			}
		}
		return Stmt{Kind: 'b', Stmts: stmts}, nil
	}
	if p.isFnDefHere() {
		return p.fnDef()
	}
	if t, ok := p.peek(); ok && t.Kind == 'i' {
		if nt, ok := p.at(1); ok && nt.Kind == '=' {
			p.pos += 2 // name and '='
			e, err := p.expr()
			if err != nil {
				return Stmt{}, err
			}
			return Stmt{Kind: 'a', Name: t.Name, Expr: e}, nil
		}
	}
	e, err := p.expr()
	if err != nil {
		return Stmt{}, err
	}
	return Stmt{Kind: 'e', Expr: e}, nil
}

func (p *parser) isFnDefHere() bool {
	t, ok := p.peek()
	if !ok || t.Kind != 'i' {
		return false
	}
	if nt, ok := p.at(1); !ok || nt.Kind != '(' {
		return false
	}
	depth := 0
	for k := 1; ; k++ {
		tk, ok := p.at(k)
		if !ok {
			return false
		}
		switch tk.Kind {
		case '(':
			depth++
		case ')':
			depth--
			if depth == 0 {
				at, ok := p.at(k + 1)
				return ok && at.Kind == '='
			}
		}
	}
}

func (p *parser) fnDef() (Stmt, error) {
	name, _ := p.next() // identifier
	p.pos++             // '('
	var params []string
	if !p.kindIs(')') {
		for {
			t, ok := p.next()
			if !ok || t.Kind != 'i' {
				return Stmt{}, fmt.Errorf("expected a parameter name")
			}
			params = append(params, t.Name)
			sep, _ := p.next()
			if sep.Kind == ',' {
				continue
			}
			if sep.Kind == ')' {
				break
			}
			return Stmt{}, fmt.Errorf("expected ',' or ')' in parameter list")
		}
	} else {
		p.pos++ // ')'
	}
	if eq, ok := p.next(); !ok || eq.Kind != '=' {
		return Stmt{}, fmt.Errorf("expected '='")
	}
	body, err := p.expr()
	if err != nil {
		return Stmt{}, err
	}
	return Stmt{Kind: 'f', Name: name.Name, Params: params, Expr: body}, nil
}

// ---- expressions (same as the boolean-logic lesson) ----

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
	return p.orExpr()
}

func (p *parser) orExpr() (*Expr, error) {
	left, err := p.andExpr()
	if err != nil {
		return nil, err
	}
	for p.isKw("or") {
		p.pos++
		right, err := p.andExpr()
		if err != nil {
			return nil, err
		}
		left = &Expr{Kind: 'O', L: left, R: right}
	}
	return left, nil
}

func (p *parser) andExpr() (*Expr, error) {
	left, err := p.notExpr()
	if err != nil {
		return nil, err
	}
	for p.isKw("and") {
		p.pos++
		right, err := p.notExpr()
		if err != nil {
			return nil, err
		}
		left = &Expr{Kind: 'A', L: left, R: right}
	}
	return left, nil
}

func (p *parser) notExpr() (*Expr, error) {
	if p.isKw("not") {
		p.pos++
		inner, err := p.notExpr()
		if err != nil {
			return nil, err
		}
		return &Expr{Kind: 'N', L: inner}, nil
	}
	return p.comparison()
}

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

// Parse a whole program into a list of top-level statements.
func Parse(tokens []Token) ([]Stmt, error) {
	p := &parser{tokens: tokens}
	return p.program()
}

// ---------------------------------------------------------------- evaluator

func asInt(v Value) (int64, error) {
	if v.Kind == 'i' {
		return v.Int, nil
	}
	return 0, fmt.Errorf("cannot do arithmetic on a function")
}

func truthy(n int64) int64 {
	if n != 0 {
		return 1
	}
	return 0
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
	case 'A', 'O', 'N':
		return evalBool(e, env)
	case 'I':
		c, err := Eval(e.Cond, env)
		if err != nil {
			return Value{}, err
		}
		n, err := asInt(c)
		if err != nil {
			return Value{}, err
		}
		if n != 0 {
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

func evalBool(e *Expr, env *Scope) (Value, error) {
	if e.Kind == 'N' {
		v, err := Eval(e.L, env)
		if err != nil {
			return Value{}, err
		}
		n, err := asInt(v)
		if err != nil {
			return Value{}, err
		}
		return Value{Kind: 'i', Int: 1 - truthy(n)}, nil
	}
	lv, err := Eval(e.L, env)
	if err != nil {
		return Value{}, err
	}
	left, err := asInt(lv)
	if err != nil {
		return Value{}, err
	}
	if e.Kind == 'A' && left == 0 {
		return Value{Kind: 'i', Int: 0}, nil
	}
	if e.Kind == 'O' && left != 0 {
		return Value{Kind: 'i', Int: 1}, nil
	}
	rv, err := Eval(e.R, env)
	if err != nil {
		return Value{}, err
	}
	right, err := asInt(rv)
	if err != nil {
		return Value{}, err
	}
	return Value{Kind: 'i', Int: truthy(right)}, nil
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
		return Value{Kind: 'i', Int: 1}, nil
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

func show(v Value) string {
	if v.Kind == 'i' {
		return strconv.FormatInt(v.Int, 10)
	}
	return "<fn>"
}

// Exec runs one statement, appending any print output to *out.
func Exec(stmt Stmt, env *Scope, out *[]string) error {
	switch stmt.Kind {
	case 'f':
		env.vars[stmt.Name] = Value{Kind: 'f', Fn: &Function{Params: stmt.Params, Body: stmt.Expr, Env: env}}
	case 'a':
		v, err := Eval(stmt.Expr, env)
		if err != nil {
			return err
		}
		env.vars[stmt.Name] = v
	case 'p':
		v, err := Eval(stmt.Expr, env)
		if err != nil {
			return err
		}
		*out = append(*out, show(v))
	case 'w':
		for {
			c, err := Eval(stmt.Expr, env)
			if err != nil {
				return err
			}
			n, err := asInt(c)
			if err != nil {
				return err
			}
			if n == 0 {
				break
			}
			if err := Exec(*stmt.Body, env, out); err != nil {
				return err
			}
		}
	case 'b':
		for _, s := range stmt.Stmts {
			if err := Exec(s, env, out); err != nil {
				return err
			}
		}
	default: // 'e'
		if _, err := Eval(stmt.Expr, env); err != nil {
			return err
		}
	}
	return nil
}

// RunProgram runs a whole program, returning everything it printed (one string
// per print). A lexing, parsing, or run-time error yields a single "error: ..."
// line.
func RunProgram(src string) []string {
	env := RootEnv()
	var out []string
	tokens, err := Tokenize(src)
	if err == nil {
		var stmts []Stmt
		stmts, err = Parse(tokens)
		if err == nil {
			for _, s := range stmts {
				if err = Exec(s, env, &out); err != nil {
					break
				}
			}
		}
	}
	if err != nil {
		out = append(out, fmt.Sprintf("error: %s", err))
	}
	return out
}
