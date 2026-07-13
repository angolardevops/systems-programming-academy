// Package hexagonal is the composition root: the only place that knows every
// layer, wiring adapters into use cases and mapping errors to user messages.
package hexagonal

import (
	"errors"
	"fmt"

	"academy.example/hexagonal/adapters"
	"academy.example/hexagonal/app"
	"academy.example/hexagonal/domain"
)

// App is what a CLI/HTTP layer would hold.
type App struct{ service *app.Service }

func NewApp() *App {
	return &App{service: app.NewService(adapters.NewInMemoryRepo())}
}

func (a *App) Add(title string) string {
	id, err := a.service.Add(title)
	switch {
	case err == nil:
		return fmt.Sprintf("Added task #%d.", id)
	case errors.Is(err, domain.ErrEmptyTitle):
		return "A task needs a title."
	default:
		return fmt.Sprintf("Unexpected error: %v", err)
	}
}

func (a *App) Complete(id int) string {
	err := a.service.Complete(id)
	switch {
	case err == nil:
		return fmt.Sprintf("Task #%d done.", id)
	case errors.Is(err, app.ErrNotFound):
		return fmt.Sprintf("No task #%d.", id)
	case errors.Is(err, domain.ErrAlreadyDone):
		return fmt.Sprintf("Task #%d was already done.", id)
	default:
		return fmt.Sprintf("Unexpected error: %v", err)
	}
}
