# Android



## Building

To build the android app you need the rust lib built with the android [ndk](https://developer.android.com/ndk).

For the emulator, modify the `build-android.sh` file to fit your system. Then launch the build.

```
cd lib
./build-android.sh
```

The script copy the file in the directory `android/app/src/main/jniLibs/x86/`

At this point you should be able to launch the android app in the emulator, for using the app in the android phone you will need to launch also `build-android-release.sh`.
