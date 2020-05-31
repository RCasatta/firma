#!/bin/bash

. ./build-android/common.sh

export RUSTFLAGS="-Clinker=i686-linux-android21-clang -Car=i686-linux-android-ar"
export TARGET=i686-linux-android
cargo build --release --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/x86/
cp ../target/$TARGET/release/libfirma.so ../android/app/src/main/jniLibs/x86/

