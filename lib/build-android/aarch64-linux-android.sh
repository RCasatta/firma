#!/bin/bash

. ./build-android/common.sh

export CC=aarch64-linux-android21-clang
export AR=llvm-ar
export RUSTFLAGS="-Clinker=$CC"
export TARGET=aarch64-linux-android
cargo build $RELEASE --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/arm64-v8a/
cp ../target/$TARGET/$TARGET_DIR/libfirma.so ../android/app/src/main/jniLibs/arm64-v8a/
