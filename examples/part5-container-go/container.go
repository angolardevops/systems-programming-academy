// Package container is a dependency-injection container: register services by
// name with a factory, then resolve them — the framework wires the graph,
// calling each factory and feeding it whatever it asks the container to
// resolve.
//
// This is the inversion of control from Part 2's Repository & DI lesson, made
// into a reusable framework. Three things a real container must get right and
// this one does, all tested:
//
//   - Lifetimes: a transient service is rebuilt on every resolve; a singleton
//     is built once and cached.
//   - Cycle detection: if A needs B and B needs A, naive resolution recurses
//     forever; we track the resolution stack and return a clear error.
//   - Missing dependencies fail loudly, naming what was not found.
//
// Factories build string values so the assembled graph is directly assertable.
package container

import (
	"fmt"
	"strings"
)

// Factory builds a service, using the Container to resolve its own
// dependencies. Returns the built value or an error.
type Factory func(c *Container) (string, error)

type registration struct {
	factory   Factory
	singleton bool
}

// Container holds registrations, the singleton cache, and the in-progress
// resolution stack used for cycle detection. Use New.
type Container struct {
	registrations map[string]registration
	cache         map[string]string
	resolving     []string
}

// New returns an empty container.
func New() *Container {
	return &Container{
		registrations: map[string]registration{},
		cache:         map[string]string{},
	}
}

// Register adds a transient service: its factory runs on every resolve,
// producing a fresh value each time.
func (c *Container) Register(name string, f Factory) {
	c.registrations[name] = registration{factory: f, singleton: false}
}

// RegisterSingleton adds a singleton service: its factory runs at most once;
// the result is cached and returned on every later resolve.
func (c *Container) RegisterSingleton(name string, f Factory) {
	c.registrations[name] = registration{factory: f, singleton: true}
}

// Resolve returns the cached singleton if present, otherwise runs the factory
// (which may resolve further dependencies), caching the result if it is a
// singleton. Errors if the name is not registered or resolving would form a
// cycle (A -> B -> A).
func (c *Container) Resolve(name string) (string, error) {
	if value, ok := c.cache[name]; ok {
		return value, nil
	}

	for _, n := range c.resolving {
		if n == name {
			chain := append(append([]string{}, c.resolving...), name)
			return "", fmt.Errorf("dependency cycle: %s", strings.Join(chain, " -> "))
		}
	}

	reg, ok := c.registrations[name]
	if !ok {
		return "", fmt.Errorf("service not registered: %s", name)
	}

	c.resolving = append(c.resolving, name)
	value, err := reg.factory(c)
	c.resolving = c.resolving[:len(c.resolving)-1]
	if err != nil {
		return "", err
	}

	if reg.singleton {
		c.cache[name] = value
	}
	return value, nil
}
