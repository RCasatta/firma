#!/bin/bash

. ./build-android/common.sh

export CC=armv7a-linux-androideabi21-clang
export RUSTFLAGS="-Clinker=$CC -Car=arm-linux-androideabi-ar"
export TARGET=armv7-linux-androideabi
cargo build --release --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/armv7a/
cp ../target/$TARGET/release/libfirma.so ../android/app/src/main/jniLibs/armeabi-v7a/
