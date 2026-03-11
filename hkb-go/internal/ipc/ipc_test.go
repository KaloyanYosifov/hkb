package ipc

import (
	"net"
	"os"
	"testing"

	"github.com/kaloyan-yosifov/hkb-go/internal/dateutil"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage"
)

func tempSock(t *testing.T) string {
	t.Helper()
	f, err := os.CreateTemp("", "hkb-*.sock")
	if err != nil {
		t.Fatalf("create temp: %v", err)
	}
	path := f.Name()
	f.Close()
	os.Remove(path)
	t.Cleanup(func() { os.Remove(path) })
	return path
}

func TestClientServerIntegration(t *testing.T) {
	sockPath := tempSock(t)

	addr, err := net.ResolveUnixAddr("unix", sockPath)
	if err != nil {
		t.Fatalf("resolve: %v", err)
	}

	listener, err := net.ListenUnix("unix", addr)
	if err != nil {
		t.Fatalf("listen: %v", err)
	}
	defer func() {
		listener.Close()
		os.Remove(sockPath)
	}()

	reminder := storage.ReminderData{
		ID:        7,
		Note:      "Integration test",
		RemindAt:  mustDate("2024-06-01 09:00:00"),
		CreatedAt: mustDate("2024-04-14 08:00:00"),
	}

	errCh := make(chan error, 1)
	eventCh := make(chan Event, 1)

	go func() {
		conn, err := listener.Accept()
		if err != nil {
			errCh <- err
			return
		}
		serverClient := FromConn(conn)
		defer serverClient.Close()

		ev, err := serverClient.ReadEvent()
		if err != nil {
			errCh <- err
			return
		}
		eventCh <- ev
	}()

	clientConn, err := net.DialUnix("unix", nil, addr)
	if err != nil {
		t.Fatalf("dial: %v", err)
	}
	client := FromConn(clientConn)
	defer client.Close()

	event, err := NewReminderCreatedEvent(reminder)
	if err != nil {
		t.Fatalf("create event: %v", err)
	}

	if err := client.SendEvent(event); err != nil {
		t.Fatalf("send event: %v", err)
	}

	select {
	case err := <-errCh:
		t.Fatalf("server error: %v", err)
	case received := <-eventCh:
		if received.Type != EventReminderCreated {
			t.Errorf("type: got %s, want %s", received.Type, EventReminderCreated)
		}
		r, err := received.ReminderData()
		if err != nil {
			t.Fatalf("reminder data: %v", err)
		}
		if r.ID != 7 {
			t.Errorf("id: got %d, want 7", r.ID)
		}
		if r.Note != "Integration test" {
			t.Errorf("note: got %q, want %q", r.Note, "Integration test")
		}
	}
}

func TestClientQueueAndFlush(t *testing.T) {
	sockPath := tempSock(t)

	addr, err := net.ResolveUnixAddr("unix", sockPath)
	if err != nil {
		t.Fatalf("resolve: %v", err)
	}

	listener, err := net.ListenUnix("unix", addr)
	if err != nil {
		t.Fatalf("listen: %v", err)
	}
	defer func() {
		listener.Close()
		os.Remove(sockPath)
	}()

	eventCh := make(chan Event, 2)
	go func() {
		conn, err := listener.Accept()
		if err != nil {
			return
		}
		serverClient := FromConn(conn)
		defer serverClient.Close()

		for i := 0; i < 2; i++ {
			ev, err := serverClient.ReadEvent()
			if err != nil {
				return
			}
			eventCh <- ev
		}
	}()

	clientConn, err := net.DialUnix("unix", nil, addr)
	if err != nil {
		t.Fatalf("dial: %v", err)
	}
	client := FromConn(clientConn)
	defer client.Close()

	ev1, _ := NewReminderDeletedEvent(1)
	ev2, _ := NewReminderDeletedEvent(2)

	client.QueueEvent(ev1)
	client.QueueEvent(ev2)

	if err := client.Flush(); err != nil {
		t.Fatalf("flush 1: %v", err)
	}
	if err := client.Flush(); err != nil {
		t.Fatalf("flush 2: %v", err)
	}

	// Third flush should be a no-op (empty queue)
	if err := client.Flush(); err != nil {
		t.Fatalf("flush 3 (noop): %v", err)
	}

	for i := 0; i < 2; i++ {
		received := <-eventCh
		id, err := received.ReminderID()
		if err != nil {
			t.Fatalf("reminder id %d: %v", i, err)
		}
		if id != int64(i+1) {
			t.Errorf("event %d: got id %d, want %d", i, id, i+1)
		}
	}
}

func TestBidirectionalCommunication(t *testing.T) {
	sockPath := tempSock(t)

	addr, err := net.ResolveUnixAddr("unix", sockPath)
	if err != nil {
		t.Fatalf("resolve: %v", err)
	}

	listener, err := net.ListenUnix("unix", addr)
	if err != nil {
		t.Fatalf("listen: %v", err)
	}
	defer func() {
		listener.Close()
		os.Remove(sockPath)
	}()

	done := make(chan struct{})
	go func() {
		conn, err := listener.Accept()
		if err != nil {
			return
		}
		sc := FromConn(conn)
		defer sc.Close()

		ev, err := sc.ReadEvent()
		if err != nil {
			return
		}

		reply, _ := NewReminderDeletedEvent(99)
		_ = sc.SendEvent(reply)
		_ = ev
		close(done)
	}()

	clientConn, err := net.DialUnix("unix", nil, addr)
	if err != nil {
		t.Fatalf("dial: %v", err)
	}
	client := FromConn(clientConn)
	defer client.Close()

	reminder := storage.ReminderData{
		ID:        1,
		Note:      "ping",
		RemindAt:  dateutil.Now(),
		CreatedAt: dateutil.Now(),
	}
	event, _ := NewReminderCreatedEvent(reminder)
	if err := client.SendEvent(event); err != nil {
		t.Fatalf("send: %v", err)
	}

	reply, err := client.ReadEvent()
	if err != nil {
		t.Fatalf("read reply: %v", err)
	}

	id, err := reply.ReminderID()
	if err != nil {
		t.Fatalf("reply id: %v", err)
	}
	if id != 99 {
		t.Errorf("reply id: got %d, want 99", id)
	}

	<-done
}
