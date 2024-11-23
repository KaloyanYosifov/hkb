.PHONY: client install_git_hooks

install_git_hooks:
	./scripts/install_git_hooks.sh

client:
	cargo run hkb-client

lint:
	cargo clippy -- -D warnings -A clippy::let_unit_value

format:
	cargo clippy --allow-dirty --allow-staged --fix

test:
	cargo test

build:
	cargo build

-include install_git_hooks
