.PHONY: tui daemon cli lint test build

tui:
	go run ./cmd/tui

daemon:
	go run ./cmd/daemon

cli:
	go run ./cmd/cli

lint:
	golangci-lint run ./...

test:
	go test ./...

build:
	go build -o bin/hkb-tui ./cmd/tui
	go build -o bin/hkb-daemon ./cmd/daemon
	go build -o bin/hkb ./cmd/cli
