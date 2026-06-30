#!/usr/bin/env bash

RUSTFLAGS="-Awarnings" cargo run --bin server &
server_process=$!

RUSTFLAGS="-Awarnings" cargo run --bin client-desktop

kill "$server_process" 2>/dev/null
wait "$server_process" 2>/dev/null