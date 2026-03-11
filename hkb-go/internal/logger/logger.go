package logger

import (
	"io"
	"log/slog"
	"os"
	"path/filepath"
	"strings"
	"sync/atomic"

	"github.com/kaloyan-yosifov/hkb-go/internal/config"
)

type AppenderType int

const (
	File AppenderType = iota
	Stdout
)

var globalLogger atomic.Pointer[slog.Logger]

func Init(appenders ...AppenderType) {
	writers := make([]io.Writer, 0, len(appenders))

	for _, a := range appenders {
		switch a {
		case File:
			w, err := openLogFile()
			if err == nil {
				writers = append(writers, w)
			}
		case Stdout:
			writers = append(writers, os.Stdout)
		}
	}

	if len(writers) == 0 {
		writers = append(writers, os.Stdout)
	}

	var w io.Writer
	if len(writers) == 1 {
		w = writers[0]
	} else {
		w = io.MultiWriter(writers...)
	}

	level := parseLevel(os.Getenv("HKB_LOG_LEVEL"))
	opts := &slog.HandlerOptions{Level: level}
	handler := slog.NewTextHandler(w, opts)
	l := slog.New(handler)
	globalLogger.Store(l)
}

func Logger() *slog.Logger {
	if l := globalLogger.Load(); l != nil {
		return l
	}
	Init(Stdout)
	return globalLogger.Load()
}

func openLogFile() (*os.File, error) {
	dir := config.CacheDir()
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return nil, err
	}
	path := filepath.Join(dir, "main.log")
	return os.OpenFile(path, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o644)
}

func parseLevel(s string) slog.Level {
	switch strings.ToLower(strings.TrimSpace(s)) {
	case "debug":
		return slog.LevelDebug
	case "warn", "warning":
		return slog.LevelWarn
	case "error":
		return slog.LevelError
	default:
		return slog.LevelInfo
	}
}
