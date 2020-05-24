# configure this according to you system and your target
export NDK=$HOME/android-ndk-r21
export TARGET=armv7a-linux-androideabi
export RUST_TARGET=armv7-linux-androideabi
export TOOL_TARGET=arm-linux-androideabi
export API=21
export HOST=darwin-x86_64
export TOOLCHAIN=$NDK/toolchains/llvm/prebuilt/$HOST
##### end configure

export CC=$TOOLCHAIN/bin/$TARGET$API-clang
export LD=$TOOLCHAIN/bin/$TOOL_TARGET-ld
export RANLIB=$TOOLCHAIN/bin/$TOOL_TARGET-ranlib
export STRIP=$TOOLCHAIN/bin/$TOOL_TARGET-strip
export AR=$TOOLCHAIN/bin/$TOOL_TARGET-ar
export AS=$TOOLCHAIN/bin/$TOOL_TARGET-as

cargo build --release --target $RUST_TARGET
mkdir -p ../android/app/src/main/jniLibs/armv7a/
cp ../target/$RUST_TARGET/release/libfirma.so ../android/app/src/main/jniLibs/armeabi-v7a/
