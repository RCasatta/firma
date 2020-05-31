#!/bin/bash

[ -z "$NDK" ] && echo "NDK is unset or set to the empty string (eg. $HOME/android-ndk-r21)" && exit 1
[ -z "$HOST" ] && echo "HOST is unset or set to the empty string (eg. linux-x86_64 or darwin-x86_64)" && exit 1

export PATH=$PATH:$NDK/toolchains/llvm/prebuilt/$HOST/bin/
echo $PATH
export API=21
export TOOLCHAIN=$NDK/toolchains/llvm/prebuilt/$HOST

export TARGET=armv7a-linux-androideabi
export CC=$TOOLCHAIN/bin/$TARGET$API-clang
export LD=$TOOLCHAIN/bin/$TARGET-ld
export TARGET=armv7-linux-androideabi
cargo build --release --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/armeabi-v7a/
cp ../target/$TARGET/release/libfirma.so ../android/app/src/main/jniLibs/armeabi-v7a/

export TARGET=aarch64-linux-android
export CC=$TOOLCHAIN/bin/$TARGET$API-clang
export LD=$TOOLCHAIN/bin/$TARGET-ld
cargo build --release --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/arm64-v8a/
cp ../target/$TARGET/release/libfirma.so ../android/app/src/main/jniLibs/arm64-v8a/

export TARGET=i686-linux-android
export CC=$TOOLCHAIN/bin/$TARGET$API-clang
export LD=$TOOLCHAIN/bin/$TARGET-ld
cargo build --release --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/x86/
cp ../target/$TARGET/release/libfirma.so ../android/app/src/main/jniLibs/x86/

export TARGET=x86_64-linux-android
export CC=$TOOLCHAIN/bin/$TARGET$API-clang
export LD=$TOOLCHAIN/bin/$TARGET-ld
cargo build --release --target $TARGET
mkdir -p ../android/app/src/main/jniLibs/x86_64/
cp ../target/$TARGET/release/libfirma.so ../android/app/src/main/jniLibs/x86_64/
