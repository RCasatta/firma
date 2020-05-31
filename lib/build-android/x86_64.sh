#!/bin/bash

. ./build-android/common.sh

export RUSTFLAGS="-Clinker=x86_64-linux-android21-clang -Car=x86_64-linux-android-ar"
export TARGET=x86_64-linux-android
cargo build --release --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/x86_64/
cp ../target/$TARGET/release/libfirma.so ../android/app/src/main/jniLibs/x86_64/

