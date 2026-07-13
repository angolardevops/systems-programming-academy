package hexagonal

import "testing"

// E2E through the composition root: user-visible messages only.
func TestFullFlowThroughTheApp(t *testing.T) {
	app := NewApp()
	steps := []struct{ got, want string }{
		{app.Add("write lesson"), "Added task #1."},
		{app.Complete(1), "Task #1 done."},
		{app.Complete(1), "Task #1 was already done."},
		{app.Complete(9), "No task #9."},
		{app.Add("  "), "A task needs a title."},
	}
	for i, s := range steps {
		if s.got != s.want {
			t.Errorf("step %d: got %q, want %q", i, s.got, s.want)
		}
	}
}
