#!/bin/bash

. ./build-android/common.sh

export CC=x86_64-linux-android21-clang
export AR=llvm-ar
export RUSTFLAGS="-Clinker=$CC"
export TARGET=x86_64-linux-android
cargo build $RELEASE --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/x86_64/
cp ../target/$TARGET/$TARGET_DIR/libfirma.so ../android/app/src/main/jniLibs/x86_64/
