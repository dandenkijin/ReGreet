#!/bin/bash

# Ensure XDG_RUNTIME_DIR is set
if test -z "$XDG_RUNTIME_DIR"; then
    export XDG_RUNTIME_DIR="/run/users/$(id -u)"
    if ! test -d "$XDG_RUNTIME_DIR"; then
        mkdir -p "$XDG_RUNTIME_DIR"
        chown "$(id -u):$(id -g)" "$XDG_RUNTIME_DIR"
        chmod 700 "$XDG_RUNTIME_DIR"
    fi
fi

LOG_FILE="/home/greeter/.local/share/regreet/niri-error.log"
mkdir -p "$(dirname "$LOG_FILE")"
exec >>"$LOG_FILE" 2>&1

# Run niri with dbus session, loading the greetd config
exec dbus-run-session niri --config /etc/greetd/niri-greetd.kdl
