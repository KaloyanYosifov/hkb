package config

import (
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
)

func TestDataDirContainsAppName(t *testing.T) {
	dir := DataDir()
	if !strings.HasSuffix(dir, appName) {
		t.Errorf("DataDir() = %q, want suffix %q", dir, appName)
	}
}

func TestCacheDirContainsAppName(t *testing.T) {
	dir := CacheDir()
	if !strings.HasSuffix(dir, appName) {
		t.Errorf("CacheDir() = %q, want suffix %q", dir, appName)
	}
}

func TestDBPathEndsWithSqlite(t *testing.T) {
	p := DBPath()
	if !strings.HasSuffix(p, "db.sqlite") {
		t.Errorf("DBPath() = %q, want suffix db.sqlite", p)
	}
}

func TestSocketPathEndsWithSock(t *testing.T) {
	p := SocketPath()
	if !strings.HasSuffix(p, "hkb.sock") {
		t.Errorf("SocketPath() = %q, want suffix hkb.sock", p)
	}
}

func TestSoundDirEndsWithSound(t *testing.T) {
	p := SoundDir()
	if !strings.HasSuffix(p, "sound") {
		t.Errorf("SoundDir() = %q, want suffix sound", p)
	}
}

func TestDataDirPlatform(t *testing.T) {
	home := homeDir()
	dir := DataDir()
	switch runtime.GOOS {
	case "darwin":
		expected := filepath.Join(home, "Library", "Application Support", appName)
		if dir != expected {
			t.Errorf("DataDir() = %q, want %q", dir, expected)
		}
	case "linux":
		expected := filepath.Join(home, ".local", "share", appName)
		if dir != expected {
			t.Errorf("DataDir() = %q, want %q", dir, expected)
		}
	}
}

func TestEnsureDataDir(t *testing.T) {
	tmpDir := t.TempDir()
	orig := os.Getenv("XDG_DATA_HOME")
	t.Setenv("XDG_DATA_HOME", tmpDir)
	defer func() { _ = os.Setenv("XDG_DATA_HOME", orig) }()

	if runtime.GOOS == "darwin" {
		t.Skip("XDG_DATA_HOME not used on darwin")
	}

	err := EnsureDataDir()
	if err != nil {
		t.Fatalf("EnsureDataDir() error = %v", err)
	}

	expected := filepath.Join(tmpDir, appName)
	info, err := os.Stat(expected)
	if err != nil {
		t.Fatalf("directory not created: %v", err)
	}
	if !info.IsDir() {
		t.Fatalf("%q is not a directory", expected)
	}
}
