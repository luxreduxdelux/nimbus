#!/usr/bin/env bash

RUSTFLAGS="-Awarnings" cargo run --bin nimbus-server &
server_process=$!

sleep 1

if ! kill -0 "$server_process" 2>/dev/null; then
    wait "$server_process"
    exit_code=$?
    exit "$exit_code"
fi

RUSTFLAGS="-Awarnings" cargo run --bin nimbus-desktop

kill "$server_process" 2>/dev/null
wait "$server_process" 2>/dev/null