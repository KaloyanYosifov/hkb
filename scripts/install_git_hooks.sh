#! /usr/bin/env sh

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ROOT_DIR=$SCRIPT_DIR/../
HOOKS_DIR=$ROOT_DIR/.git-hooks
GIT_HOOKS_DIR=$ROOT_DIR/.git/hooks

hooks="$(ls $HOOKS_DIR)"

function install_hook {
    echo "Installing $hook git hook"
    cp "$HOOKS_DIR/$hook" "$GIT_HOOKS_DIR/$hook" && echo "Hook $hook installed" || "Hook $hook failed to install."
}

for hook in $hooks; do
    ! [ -f "$GIT_HOOKS_DIR/$hook" ] && install_hook $hook
done
