#!/bin/bash

if [ -z "$DEBUG" ]
then
  export TARGET_DIR="release"
  export RELEASE="--release"
else
  export TARGET_DIR="debug"
  export RELEASE=""
fi

[ -z "$NDK" ] && echo "NDK is unset or set to the empty string (eg. $HOME/android-ndk-r21)" && exit 1
[ -z "$HOST" ] && echo "HOST is unset or set to the empty string (eg. linux-x86_64 or darwin-x86_64)" && exit 1


export TOOLCHAIN=$NDK/toolchains/llvm/prebuilt/$HOST
export PATH=$PATH:$TOOLCHAIN/bin