.PHONY: client daemon install_git_hooks

install_git_hooks:
	./scripts/install_git_hooks.sh

client:
	cargo run --bin=hkb_client

daemon:
	cargo run --bin=hkb_daemon

lint:
	cargo clippy -- -D warnings -A clippy::let_unit_value -A clippy::only-used-in-recursion -A clippy::comparison_chain

format:
	cargo clippy --allow-dirty --allow-staged --fix

test:
	cargo test

build:
	cargo build

-include install_git_hooks
