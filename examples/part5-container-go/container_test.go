package container

import (
	"fmt"
	"strings"
	"testing"
)

func TestResolvesALeafService(t *testing.T) {
	c := New()
	c.Register("config", func(*Container) (string, error) { return "Config(db=memory)", nil })
	got, err := c.Resolve("config")
	if err != nil || got != "Config(db=memory)" {
		t.Fatalf("Resolve = %q, %v", got, err)
	}
}

func TestResolvesADependencyChain(t *testing.T) {
	c := New()
	c.Register("config", func(*Container) (string, error) { return "Config", nil })
	c.Register("repo", func(c *Container) (string, error) {
		dep, err := c.Resolve("config")
		return "Repo(uses " + dep + ")", err
	})
	c.Register("service", func(c *Container) (string, error) {
		dep, err := c.Resolve("repo")
		return "Service(uses " + dep + ")", err
	})
	got, err := c.Resolve("service")
	if err != nil || got != "Service(uses Repo(uses Config))" {
		t.Fatalf("Resolve = %q, %v", got, err)
	}
}

func TestUnknownServiceErrorsWithItsName(t *testing.T) {
	c := New()
	_, err := c.Resolve("nope")
	if err == nil || !strings.Contains(err.Error(), "nope") {
		t.Fatalf("error should name the service, got: %v", err)
	}
}

func TestTransientRebuildsEveryResolve(t *testing.T) {
	count := 0
	c := New()
	c.Register("id", func(*Container) (string, error) {
		count++
		return fmt.Sprintf("instance-%d", count), nil
	})
	for i, want := range []string{"instance-1", "instance-2", "instance-3"} {
		if got, _ := c.Resolve("id"); got != want {
			t.Fatalf("resolve #%d = %q, want %q", i+1, got, want)
		}
	}
}

func TestSingletonBuildsOnceAndCaches(t *testing.T) {
	count := 0
	c := New()
	c.RegisterSingleton("id", func(*Container) (string, error) {
		count++
		return fmt.Sprintf("instance-%d", count), nil
	})
	first, _ := c.Resolve("id")
	second, _ := c.Resolve("id")
	if first != "instance-1" || second != "instance-1" {
		t.Fatalf("got %q then %q, want both instance-1", first, second)
	}
	if count != 1 {
		t.Fatalf("factory ran %d times, want 1", count)
	}
}

func TestDirectCycleIsDetected(t *testing.T) {
	c := New()
	c.Register("a", func(c *Container) (string, error) {
		dep, err := c.Resolve("b")
		return "A(" + dep + ")", err
	})
	c.Register("b", func(c *Container) (string, error) {
		dep, err := c.Resolve("a")
		return "B(" + dep + ")", err
	})
	_, err := c.Resolve("a")
	if err == nil || !strings.Contains(err.Error(), "cycle") {
		t.Fatalf("expected cycle error, got: %v", err)
	}
	if !strings.Contains(err.Error(), "a -> b -> a") {
		t.Fatalf("error should show the chain, got: %v", err)
	}
}

func TestSelfCycleIsDetected(t *testing.T) {
	c := New()
	c.Register("loop", func(c *Container) (string, error) { return c.Resolve("loop") })
	_, err := c.Resolve("loop")
	if err == nil || !strings.Contains(err.Error(), "cycle") {
		t.Fatalf("expected cycle error, got: %v", err)
	}
}

func TestSingletonDependencyIsSharedAcrossConsumers(t *testing.T) {
	count := 0
	c := New()
	c.RegisterSingleton("db", func(*Container) (string, error) {
		count++
		return fmt.Sprintf("DB#%d", count), nil
	})
	c.Register("users", func(c *Container) (string, error) {
		dep, err := c.Resolve("db")
		return "Users(" + dep + ")", err
	})
	c.Register("orders", func(c *Container) (string, error) {
		dep, err := c.Resolve("db")
		return "Orders(" + dep + ")", err
	})
	if got, _ := c.Resolve("users"); got != "Users(DB#1)" {
		t.Fatalf("users = %q", got)
	}
	if got, _ := c.Resolve("orders"); got != "Orders(DB#1)" {
		t.Fatalf("orders = %q", got)
	}
	if count != 1 {
		t.Fatalf("db built %d times, want 1", count)
	}
}
