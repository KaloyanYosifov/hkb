.PHONY: client install_git_hooks

install_git_hooks:
	./scripts/install_git_hooks.sh

client:
	cargo run hkb-client

-include install_git_hooks
