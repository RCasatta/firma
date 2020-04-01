export NDK_HOME=$HOME/android-ndk-r21
export PATH=$PATH:$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin
export CC_aarch64_linux_android=aarch64-linux-android21-clang

cargo build --release --target aarch64-linux-android
cp ../target/aarch64-linux-android/release/libfirma.so ../android/app/src/main/jniLibs/arm64-v8a/
