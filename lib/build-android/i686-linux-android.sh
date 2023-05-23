#!/bin/bash

. ./build-android/common.sh

export CC=i686-linux-android21-clang
export AR=llvm-ar
export RUSTFLAGS="-Clinker=$CC -Car=llvm-ar"
export TARGET=i686-linux-android
cargo build $RELEASE --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/x86/
cp ../target/$TARGET/$TARGET_DIR/libfirma.so ../android/app/src/main/jniLibs/x86/
