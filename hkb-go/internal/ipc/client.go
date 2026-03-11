package ipc

import (
	"fmt"
	"io"
	"net"
	"sync"

	"github.com/kaloyan-yosifov/hkb-go/internal/config"
)

type Client struct {
	conn  net.Conn
	mu    sync.Mutex
	queue []Event
}

func Connect() (*Client, error) {
	sockPath := config.SocketPath()
	addr, err := net.ResolveUnixAddr("unix", sockPath)
	if err != nil {
		return nil, fmt.Errorf("resolve unix addr: %w", err)
	}

	conn, err := net.DialUnix("unix", nil, addr)
	if err != nil {
		return nil, fmt.Errorf("connect to daemon: %w", err)
	}

	return &Client{conn: conn}, nil
}

func FromConn(conn net.Conn) *Client {
	return &Client{conn: conn}
}

func (c *Client) ReadEvent() (Event, error) {
	var firstBuf [FrameSize]byte
	if _, err := io.ReadFull(c.conn, firstBuf[:]); err != nil {
		return Event{}, fmt.Errorf("read frame: %w", err)
	}

	first := FrameFromBytes(firstBuf)

	if first.RelatedFrames == 1 {
		return AssembleFrames([]Frame{first})
	}

	frames := make([]Frame, 0, first.RelatedFrames)
	frames = append(frames, first)

	for i := uint8(1); i < first.RelatedFrames; i++ {
		var buf [FrameSize]byte
		if _, err := io.ReadFull(c.conn, buf[:]); err != nil {
			return Event{}, fmt.Errorf("read frame %d: %w", i+1, err)
		}
		frames = append(frames, FrameFromBytes(buf))
	}

	return AssembleFrames(frames)
}

func (c *Client) SendEvent(event Event) error {
	frames, err := FramesFromEvent(event)
	if err != nil {
		return err
	}

	for _, f := range frames {
		buf := f.ToBytes()
		if _, err := c.conn.Write(buf[:]); err != nil {
			return fmt.Errorf("write frame: %w", err)
		}
	}

	return nil
}

func (c *Client) QueueEvent(event Event) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.queue = append(c.queue, event)
}

func (c *Client) Flush() error {
	c.mu.Lock()
	if len(c.queue) == 0 {
		c.mu.Unlock()
		return nil
	}
	event := c.queue[0]
	c.queue = c.queue[1:]
	c.mu.Unlock()

	return c.SendEvent(event)
}

func (c *Client) Close() error {
	return c.conn.Close()
}
