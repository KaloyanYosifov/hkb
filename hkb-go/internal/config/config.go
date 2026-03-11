package config

import (
	"os"
	"path/filepath"
	"runtime"
)

const appName = "hkb"

func homeDir() string {
	home, err := os.UserHomeDir()
	if err != nil {
		panic("unable to determine home directory: " + err.Error())
	}
	return home
}

func DataDir() string {
	home := homeDir()
	switch runtime.GOOS {
	case "darwin":
		return filepath.Join(home, "Library", "Application Support", appName)
	default:
		if xdg := os.Getenv("XDG_DATA_HOME"); xdg != "" {
			return filepath.Join(xdg, appName)
		}
		return filepath.Join(home, ".local", "share", appName)
	}
}

func CacheDir() string {
	home := homeDir()
	switch runtime.GOOS {
	case "darwin":
		return filepath.Join(home, "Library", "Caches", appName)
	default:
		if xdg := os.Getenv("XDG_CACHE_HOME"); xdg != "" {
			return filepath.Join(xdg, appName)
		}
		return filepath.Join(home, ".cache", appName)
	}
}

func DBPath() string {
	return filepath.Join(DataDir(), "db.sqlite")
}

func SocketPath() string {
	return filepath.Join(DataDir(), "hkb.sock")
}

func SoundDir() string {
	return filepath.Join(DataDir(), "sound")
}

func EnsureDataDir() error {
	return os.MkdirAll(DataDir(), 0o755)
}
