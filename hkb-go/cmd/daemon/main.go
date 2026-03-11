package main

import (
	"fmt"
	"log/slog"
	"os"
	"os/signal"
	"sync"
	"syscall"
	"time"

	"github.com/kaloyan-yosifov/hkb-go/internal/config"
	"github.com/kaloyan-yosifov/hkb-go/internal/dateutil"
	"github.com/kaloyan-yosifov/hkb-go/internal/ipc"
	"github.com/kaloyan-yosifov/hkb-go/internal/logger"
	"github.com/kaloyan-yosifov/hkb-go/internal/notify"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage/sqlite"
)

func main() {
	if err := config.EnsureDataDir(); err != nil {
		fmt.Fprintf(os.Stderr, "fatal: %v\n", err)
		os.Exit(1)
	}

	logger.Init(logger.File, logger.Stdout)
	log := logger.Logger()

	repo, err := sqlite.New(config.DBPath())
	if err != nil {
		log.Error("failed to open database", "error", err)
		os.Exit(1)
	}
	defer repo.Close()
	log.Info("database opened", "path", config.DBPath())

	server, err := ipc.Bind()
	if err != nil {
		log.Error("failed to bind IPC server", "error", err)
		os.Exit(1)
	}
	defer server.Close()
	log.Info("IPC server listening", "addr", server.Addr())

	notify.InitAudio(log)

	stopCh := make(chan struct{})
	var wg sync.WaitGroup

	wg.Add(1)
	go func() {
		defer wg.Done()
		reminderScheduler(log, repo, stopCh)
	}()

	wg.Add(1)
	go func() {
		defer wg.Done()
		cleanupScheduler(log, repo, stopCh)
	}()

	wg.Add(1)
	go func() {
		defer wg.Done()
		acceptLoop(log, server, stopCh)
	}()

	fmt.Println("HKB Daemon running. Press Ctrl+C to stop.")

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
	<-sig

	log.Info("shutting down")
	close(stopCh)
	server.Close()
	wg.Wait()
	log.Info("daemon stopped")
}

func acceptLoop(log *slog.Logger, server *ipc.Server, stop <-chan struct{}) {
	for {
		conn, err := server.Accept()
		if err != nil {
			select {
			case <-stop:
				return
			default:
				log.Warn("accept error", "error", err)
				continue
			}
		}

		log.Info("client connected")
		go handleConnection(log, ipc.FromConn(conn), stop)
	}
}

func handleConnection(log *slog.Logger, client *ipc.Client, stop <-chan struct{}) {
	defer client.Close()

	flushTicker := time.NewTicker(500 * time.Millisecond)
	defer flushTicker.Stop()

	eventCh := make(chan ipc.Event)
	errCh := make(chan error)

	go func() {
		for {
			ev, err := client.ReadEvent()
			if err != nil {
				errCh <- err
				return
			}
			eventCh <- ev
		}
	}()

	for {
		select {
		case <-stop:
			return
		case ev := <-eventCh:
			log.Info("received event", "type", ev.Type)
		case err := <-errCh:
			log.Info("client disconnected", "reason", err)
			return
		case <-flushTicker.C:
			if err := client.Flush(); err != nil {
				log.Warn("flush error", "error", err)
				return
			}
		}
	}
}

type notificationInterval struct {
	start time.Duration
	end   time.Duration
	label string
}

var intervals = []notificationInterval{
	{0, 1 * time.Minute, "1 minute"},
	{0, 5 * time.Minute, "5 minutes"},
	{6 * time.Minute, 15 * time.Minute, "15 minutes"},
	{16 * time.Minute, 30 * time.Minute, "30 minutes"},
}

func reminderScheduler(log *slog.Logger, repo storage.Repository, stop <-chan struct{}) {
	reminded := make(map[string]map[int64]bool)
	for _, iv := range intervals {
		reminded[iv.label] = make(map[int64]bool)
	}

	ticker := time.NewTicker(10 * time.Second)
	defer ticker.Stop()

	checkReminders(log, repo, reminded)

	for {
		select {
		case <-stop:
			return
		case <-ticker.C:
			checkReminders(log, repo, reminded)
		}
	}
}

func checkReminders(log *slog.Logger, repo storage.Repository, reminded map[string]map[int64]bool) {
	now := dateutil.Now()
	notified := false

	for _, iv := range intervals {
		start := now.AddDuration(dateutil.Minute(uint32(iv.start.Minutes())))
		end := now.AddDuration(dateutil.Minute(uint32(iv.end.Minutes())))

		var excludeIDs []int64
		for id := range reminded[iv.label] {
			excludeIDs = append(excludeIDs, id)
		}

		opts := []storage.QueryOption{storage.RemindAtBetween(start, end)}
		if len(excludeIDs) > 0 {
			opts = append(opts, storage.WithoutIDs(excludeIDs...))
		}

		reminders, err := repo.FetchReminders(opts...)
		if err != nil {
			log.Warn("fetch reminders failed", "interval", iv.label, "error", err)
			continue
		}

		for _, r := range reminders {
			summary := fmt.Sprintf("You have a reminder in: %s", iv.label)
			if err := notify.Notify(summary, r.Note, 3*time.Second); err != nil {
				log.Warn("notification failed", "error", err)
			} else {
				log.Info("notification sent", "reminder_id", r.ID, "interval", iv.label)
			}
			reminded[iv.label][r.ID] = true
			notified = true
		}
	}

	if notified {
		notify.Play("notification.wav")
	}
}

func cleanupScheduler(log *slog.Logger, repo storage.Repository, stop <-chan struct{}) {
	ticker := time.NewTicker(5 * time.Minute)
	defer ticker.Stop()

	for {
		select {
		case <-stop:
			return
		case <-ticker.C:
			cutoff := dateutil.Now().SubDuration(dateutil.Day(1))
			if err := repo.DeleteReminders(storage.RemindAtLe(cutoff)); err != nil {
				log.Warn("cleanup failed", "error", err)
			} else {
				log.Debug("cleanup completed", "cutoff", cutoff.String())
			}
		}
	}
}
