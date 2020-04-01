export NDK_HOME=$HOME/android-ndk-r21
#export NDK_HOME=$HOME/AndroidSDK/sdk/ndk/20.0.5594570
export PATH=$PATH:$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin
export CC_i686_linux_android=i686-linux-android16-clang

cargo build --target i686-linux-android
cp ../target/i686-linux-android/debug/libfirma.so ../android/app/src/main/jniLibs/x86/
