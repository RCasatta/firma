#!/bin/bash

. ./build-android/common.sh

export RUSTFLAGS="-Clinker=aarch64-linux-android21-clang -Car=aarch64-linux-android-ar"
export TARGET=aarch64-linux-android
cargo build --release --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/arm64-v8a/
cp ../target/$TARGET/release/libfirma.so ../android/app/src/main/jniLibs/arm64-v8a/
