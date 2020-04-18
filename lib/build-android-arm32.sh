# configure this according to you system and your target
export NDK=$HOME/android-ndk-r21
#export TARGET=i686-linux-android
#export TARGET=aarch64-linux-android
export HOST=darwin-x86_64
export TARGETREST=arm-linux-androideabi
export TARGET=armv7-linux-androideabi
export TARGETCC=armv7a-linux-androideabi
export API=16
export TOOLCHAIN=$NDK/toolchains/llvm/prebuilt/$HOST
##### end configure

#export CC_i686_linux_android=$TOOLCHAIN/bin/$TARGET$API-clang
#export CXX_i686_linux_android=$TOOLCHAIN/bin/$TARGET$API-clang++

#export CC_aarch64-linux-android=$TOOLCHAIN/bin/$TARGET$API-clang
#export CXX_aarch64-linux-android=$TOOLCHAIN/bin/$TARGET$API-clang++

export CC=$TOOLCHAIN/bin/$TARGETCC$API-clang
# export CXX=$TOOLCHAIN/bin/$TARGET$API-clang++

export LD=$TOOLCHAIN/bin/$TARGETREST-ld
export RANLIB=$TOOLCHAIN/bin/$TARGETREST-ranlib
export STRIP=$TOOLCHAIN/bin/$TARGETREST-strip
export AR=$TOOLCHAIN/bin/$TARGETREST-ar
export AS=$TOOLCHAIN/bin/$TARGETREST-as

cargo build --target armv7-linux-androideabi
#cp ../target/$TARGET/debug/libfirma.so ../android/app/src/main/jniLibs/x86/
cp ../target/$TARGET/debug/libfirma.so ../android/app/src/main/jniLibs/armeabi-v7a/
