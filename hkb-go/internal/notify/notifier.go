package notify

import (
	"fmt"
	"os/exec"
	"runtime"
	"time"
)

func Notify(summary, body string, timeout time.Duration) error {
	switch runtime.GOOS {
	case "darwin":
		return notifyMacOS(summary, body)
	case "linux":
		return notifyLinux(summary, body, timeout)
	default:
		return fmt.Errorf("notifications not supported on %s", runtime.GOOS)
	}
}

func notifyMacOS(summary, body string) error {
	script := fmt.Sprintf(
		`display notification %q with title %q`,
		body, summary,
	)
	return exec.Command("osascript", "-e", script).Run()
}

func notifyLinux(summary, body string, timeout time.Duration) error {
	ms := fmt.Sprintf("%d", timeout.Milliseconds())
	return exec.Command("notify-send", "-t", ms, summary, body).Run()
}
