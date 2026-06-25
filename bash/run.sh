#!/usr/bin/env bash

cargo run --bin server &
server_process=$!

cargo run --bin client-desktop

kill "$server_process" 2>/dev/null
wait "$server_process" 2>/dev/null