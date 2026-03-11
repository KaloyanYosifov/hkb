package main

import (
	"fmt"
	"log/slog"
	"os"
	"time"

	tea "github.com/charmbracelet/bubbletea"

	"github.com/kaloyan-yosifov/hkb-go/internal/config"
	"github.com/kaloyan-yosifov/hkb-go/internal/ipc"
	"github.com/kaloyan-yosifov/hkb-go/internal/logger"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage/sqlite"
	appTui "github.com/kaloyan-yosifov/hkb-go/internal/tui"
)

func main() {
	if err := config.EnsureDataDir(); err != nil {
		fmt.Fprintf(os.Stderr, "fatal: %v\n", err)
		os.Exit(1)
	}

	logger.Init(logger.File)
	log := logger.Logger()

	repo, err := sqlite.New(config.DBPath())
	if err != nil {
		log.Error("failed to open database", "error", err)
		os.Exit(1)
	}
	defer repo.Close()

	daemonCh := make(chan ipc.Event, 64)
	go daemonConnector(log, daemonCh)

	model := appTui.NewModel(repo, daemonCh)
	p := tea.NewProgram(model, tea.WithAltScreen())

	if _, err := p.Run(); err != nil {
		log.Error("TUI error", "error", err)
		os.Exit(1)
	}
}

func daemonConnector(log *slog.Logger, events <-chan ipc.Event) {
	for {
		client, err := ipc.Connect()
		if err != nil {
			log.Debug("daemon not available, retrying in 3s", "error", err)
			time.Sleep(3 * time.Second)
			continue
		}

		log.Info("connected to daemon")

		flushTicker := time.NewTicker(500 * time.Millisecond)
		connected := true

		for connected {
			select {
			case ev, ok := <-events:
				if !ok {
					client.Close()
					return
				}
				client.QueueEvent(ev)
			case <-flushTicker.C:
				if err := client.Flush(); err != nil {
					log.Warn("daemon connection lost", "error", err)
					connected = false
				}
			}
		}

		flushTicker.Stop()
		client.Close()
		log.Info("reconnecting to daemon in 3s")
		time.Sleep(3 * time.Second)
	}
}
