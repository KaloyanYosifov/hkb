package notify

import (
	"log/slog"
	"os/exec"
	"path/filepath"
	"runtime"
	"sync"

	"github.com/kaloyan-yosifov/hkb-go/internal/config"
)

var (
	audioCh   chan string
	audioOnce sync.Once
)

func InitAudio(logger *slog.Logger) {
	audioOnce.Do(func() {
		audioCh = make(chan string, 32)
		go audioLoop(logger)
	})
}

func Play(filename string) {
	if audioCh == nil {
		return
	}
	path := filepath.Join(config.SoundDir(), filename)
	select {
	case audioCh <- path:
	default:
	}
}

func audioLoop(logger *slog.Logger) {
	for path := range audioCh {
		if err := playFile(path); err != nil {
			logger.Warn("audio playback failed", "path", path, "error", err)
		}
	}
}

func playFile(path string) error {
	switch runtime.GOOS {
	case "darwin":
		return exec.Command("afplay", path).Run()
	case "linux":
		return exec.Command("aplay", "-q", path).Run()
	default:
		return nil
	}
}
