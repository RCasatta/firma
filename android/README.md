# Android

![Screenshot_20200418-211935](https://user-images.githubusercontent.com/6470319/79669281-bfcced80-81ba-11ea-83af-564f57875bcb.png)
![Screenshot_20200418-145729](https://user-images.githubusercontent.com/6470319/79639852-79fa3000-818e-11ea-97eb-14d493b059b3.png)
![Screenshot_20200418-145740](https://user-images.githubusercontent.com/6470319/79639850-79fa3000-818e-11ea-82ad-4823264634cb.png)
![Screenshot_20200418-150030](https://user-images.githubusercontent.com/6470319/79639849-79619980-818e-11ea-82b7-985636a0eb3b.png)
![Screenshot_20200418-150041](https://user-images.githubusercontent.com/6470319/79639847-78c90300-818e-11ea-8041-5a08618caa23.png)
![Screenshot_20200418-160141](https://user-images.githubusercontent.com/6470319/79639843-76ff3f80-818e-11ea-955a-24a03c75c989.png)

## Beta test

https://github.com/RCasatta/firma/releases/tag/beta_release_0.2.0

## Building

To build the android app you need the rust lib built with the android [ndk](https://developer.android.com/ndk).

For the emulator, modify the `build-android.sh` file to fit your system. Then launch the build. Starting from the root of the repo:

```
cd lib
./build-android.sh
```

The script copy the file in the directory `android/app/src/main/jniLibs/x86/`

At this point you should be able to launch the android app in the emulator, for using the app in the android phone you will need to launch also `build-android-release.sh`.

## Example PSBT

![qr-0](https://user-images.githubusercontent.com/6470319/79686978-745f2180-8244-11ea-8fd9-fc1a685ab0a3.png)
![qr-1](https://user-images.githubusercontent.com/6470319/79686979-74f7b800-8244-11ea-82f9-f8d8b4a011f8.png)
![qr-2](https://user-images.githubusercontent.com/6470319/79686980-74f7b800-8244-11ea-9d1b-185080186bea.png)

## Example Wallet

![qr-0](https://user-images.githubusercontent.com/6470319/79687010-a3759300-8244-11ea-9023-bfd91c7d3bad.png)
![qr-1](https://user-images.githubusercontent.com/6470319/79687012-a40e2980-8244-11ea-82cc-6a9c803ad975.png)
