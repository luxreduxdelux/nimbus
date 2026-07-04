#!/usr/bin/env bash

BUILD_DATE=$(date +%Y%m%d%H%M%S)
BUILD_TYPE=""
BUILD_PATH=""
BUILD_NAME="linux"
BUILD_EXTENSION=""

if [ "$1" = "--build-NT" ]; then
    BUILD_TYPE="--target x86_64-pc-windows-gnu"
    BUILD_PATH="x86_64-pc-windows-gnu/"
    BUILD_NAME="NT"
    BUILD_EXTENSION=".exe"
fi

cargo build $BUILD_TYPE --release --bin nimbus-desktop
cargo build $BUILD_TYPE --release --bin nimbus-server

zip -9 -j nimbus_${BUILD_NAME}_${BUILD_DATE}.zip target/${BUILD_PATH}release/nimbus-desktop${BUILD_EXTENSION} target/${BUILD_PATH}release/nimbus-server${BUILD_EXTENSION}