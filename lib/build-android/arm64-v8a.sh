#!/bin/bash

./build-android/common.sh

export TARGET=aarch64-linux-android

export CC=$TOOLCHAIN/bin/$TARGET$API-clang
export LD=$TOOLCHAIN/bin/$TARGET-ld
export RANLIB=$TOOLCHAIN/bin/$TARGET-ranlib
export STRIP=$TOOLCHAIN/bin/$TARGET-strip
export AR=$TOOLCHAIN/bin/$TARGET-ar
export AS=$TOOLCHAIN/bin/$TARGET-as

cargo build --release --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/arm64-v8a/
cp ../target/$TARGET/release/libfirma.so ../android/app/src/main/jniLibs/arm64-v8a/
