package ipc

import (
	"fmt"
	"net"
	"os"
	"path/filepath"

	"github.com/kaloyan-yosifov/hkb-go/internal/config"
)

type Server struct {
	listener *net.UnixListener
	addr     string
}

func Bind() (*Server, error) {
	sockPath := config.SocketPath()

	if err := os.MkdirAll(filepath.Dir(sockPath), 0o755); err != nil {
		return nil, fmt.Errorf("create socket dir: %w", err)
	}

	_ = os.Remove(sockPath)

	addr, err := net.ResolveUnixAddr("unix", sockPath)
	if err != nil {
		return nil, fmt.Errorf("resolve unix addr: %w", err)
	}

	listener, err := net.ListenUnix("unix", addr)
	if err != nil {
		return nil, fmt.Errorf("listen unix: %w", err)
	}

	return &Server{
		listener: listener,
		addr:     sockPath,
	}, nil
}

func (s *Server) Accept() (net.Conn, error) {
	conn, err := s.listener.Accept()
	if err != nil {
		return nil, fmt.Errorf("accept: %w", err)
	}
	return conn, nil
}

func (s *Server) Addr() string {
	return s.addr
}

func (s *Server) Close() error {
	err := s.listener.Close()
	_ = os.Remove(s.addr)
	return err
}
