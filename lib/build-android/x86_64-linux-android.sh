#!/bin/bash

. ./build-android/common.sh

export CC=x86_64-linux-android21-clang
export RUSTFLAGS="-Clinker=$CC -Car=x86_64-linux-android-ar"
export TARGET=x86_64-linux-android
cargo build $RELEASE --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/x86_64/
cp ../target/$TARGET/$TARGET_DIR/libfirma.so ../android/app/src/main/jniLibs/x86_64/
