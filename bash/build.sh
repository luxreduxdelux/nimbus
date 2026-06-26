#!/usr/bin/env bash

BUILD_DATE=$(date +%Y%m%d%H%M%S)

cargo build --release --bin client-desktop
cargo build --release --bin server

zip -9 -j nimbus_$BUILD_DATE.zip target/release/client-desktop target/release/server