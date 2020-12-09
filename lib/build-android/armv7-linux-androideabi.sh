#!/bin/bash

. ./build-android/common.sh

export CC=armv7a-linux-androideabi21-clang
export RUSTFLAGS="-Clinker=$CC -Car=arm-linux-androideabi-ar"
export TARGET=armv7-linux-androideabi
cargo build $RELEASE --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/armeabi-v7a/
cp ../target/$TARGET/$TARGET_DIR/libfirma.so ../android/app/src/main/jniLibs/armeabi-v7a/
