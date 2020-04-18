# configure this according to you system and your target
export NDK=$HOME/android-ndk-r21
export TARGET=i686-linux-android
export API=16
export HOST=darwin-x86_64
export TOOLCHAIN=$NDK/toolchains/llvm/prebuilt/$HOST
##### end configure

export CC=$TOOLCHAIN/bin/$TARGET$API-clang
export LD=$TOOLCHAIN/bin/$TARGET-ld
export RANLIB=$TOOLCHAIN/bin/$TARGET-ranlib
export STRIP=$TOOLCHAIN/bin/$TARGET-strip
export AR=$TOOLCHAIN/bin/$TARGET-ar
export AS=$TOOLCHAIN/bin/$TARGET-as

cargo build --release --target aarch64-linux-android
cp ../target/aarch64-linux-android/release/libfirma.so ../android/app/src/main/jniLibs/arm64-v8a/
