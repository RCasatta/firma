# Android

![Screenshot_20200523-172426](https://images.casatta.it/Screenshot_20200523-172426.png?h=500)
![Screenshot_20200523-172608](https://images.casatta.it/Screenshot_20200523-172608.png?h=500)
![Screenshot_20200523-172628](https://images.casatta.it/Screenshot_20200523-172628.png?h=500)

## Beta test

Checkout releases at https://github.com/RCasatta/firma/releases 

Get testnet version from google play https://play.google.com/store/apps/details?id=it.casatta.testnet&hl=en

## Building

To build the android app you need the rust lib built with the android [ndk](https://developer.android.com/ndk).

For the emulator, modify the `build-android.sh` file to fit your system. Then launch the build. Starting from the root of the repo:

```
cd lib
./build-android.sh
```

The script copy the file in the directory `android/app/src/main/jniLibs/x86/`

At this point you should be able to launch the android app in the emulator, for using the app in the android phone you will need to launch also `build-android-release.sh`.

## Example Transaction

To import the following transaction (PSBT) copy the text, go to "select transaction" -> "+" -> "From Clipboard"

`cHNidP8BAJoCAAAAAljoeiG1ba8MI76OcHBFbDNvfLqlyHV5JPVFiHuyq911AAAAAAD/////g40EJ9DsZQpoqka7CwmK6kQiwHGyyng1Kgd5WdB86h0BAAAAAP////8CcKrwCAAAAAAWABTYXCtx0AYLCcmIauuBXlCZHdoSTQDh9QUAAAAAFgAUAK6pouXw+HaliN9VRuh0LR2HAI8AAAAAAAEAuwIAAAABqtc5MQGL0l+ErkALaISL4J23BurCrBgpi6vucatlb4sAAAAASEcwRAIgWPb8fGoz4bMVSNSByCbAFb0wE1qtQs1neQ2rZtKtJDsCIEoc7SYExnNbY5PltBaR3XiwDwxZQvufdRhW+qk4FX26Af7///8CgPD6AgAAAAAXqRQPuUY0IWlrgsgzryQceMF9295JNIfQ8gonAQAAABepFCnKdPigj4GZlCgYXJe12FLkBj9hh2UAAAABAwQBAAAAAQRHUiEClYO/Oa4KYJdHrRma3dY0+mEIVZ1sXNObTCGD8auW4H8hAtq2H/SaFNtqfQKwzR+7ePxLGDErW05U2uTbovv+9TbXUq4iBgKVg785rgpgl0etGZrd1jT6YQhVnWxc05tMIYPxq5bgfxDZDGpPAAAAgAAAAIAAAACAIgYC2rYf9JoU22p9ArDNH7t4/EsYMStbTlTa5Nui+/71NtcQ2QxqTwAAAIAAAACAAQAAgAABASAAwusLAAAAABepFLf1+vQOPUClpFmx2zU18rcvqSHohwEDBAEAAAABBCIAIIwjUxc3Q7WV37Sge3K6jkLjeX2nTof+fZ10l+OyAokDAQVHUiEDCJ3BDHrG21T5EymvYXMz2ziM6tDCMfcjN50bmQMLAtwhAjrdkE89bc9Z3bkGsN7iNSm3/7ntUOXoYVGSaGAiHw5zUq4iBgI63ZBPPW3PWd25BrDe4jUpt/+57VDl6GFRkmhgIh8OcxDZDGpPAAAAgAAAAIADAACAIgYDCJ3BDHrG21T5EymvYXMz2ziM6tDCMfcjN50bmQMLAtwQ2QxqTwAAAIAAAACAAgAAgAAiAgOppMN/WZbTqiXbrGtXCvBlA5RJKUJGCzVHU+2e7KWHcRDZDGpPAAAAgAAAAIAEAACAACICAn9jmXV9Lv9VoTatAsaEsYOLZVbl8bazQoKpS2tQBRCWENkMak8AAACAAAAAgAUAAIAA`

## Example Wallet

To import the following wallet descriptor copy the json, go to "select wallet" -> "+" -> "From Clipboard"

```json
{
  "name": "first-android",
  "descriptor_main": "wsh(multi(2,tpubD6NzVbkrYhZ4XeQW5Adf6Cho9eaBWTzoCApPf2NGsyFCYx2WVEFWQ9hmuwdJi3WbnG33CqAqFGrZYVrZeUztHoUGmPaxqzp96w2oMu9JCUV/0/*,tpubD6NzVbkrYhZ4WrwU2gJn1bJ1UrZ4kPnGAwXY384rpDhHJmcs2xJkmLm17dF1zpvC1roPWVXqiy2U4Up5dQp94ep1hjjQYS5vUArfT5kP92y/0/*))#q0agyfvx",
  "descriptor_change": "wsh(multi(2,tpubD6NzVbkrYhZ4XeQW5Adf6Cho9eaBWTzoCApPf2NGsyFCYx2WVEFWQ9hmuwdJi3WbnG33CqAqFGrZYVrZeUztHoUGmPaxqzp96w2oMu9JCUV/1/*,tpubD6NzVbkrYhZ4WrwU2gJn1bJ1UrZ4kPnGAwXY384rpDhHJmcs2xJkmLm17dF1zpvC1roPWVXqiy2U4Up5dQp94ep1hjjQYS5vUArfT5kP92y/1/*))#r0dapdc4",
  "fingerprints": [
    "8f335370",
    "6b9128bc"
  ],
  "required_sig": 2,
  "daemon_opts": {
    "url": "http://127.0.0.1:18332",
    "cookie_file": "/Volumes/Transcend/bitcoin-testnet/testnet3/.cookie"
  },
  "created_at_height": 1718227
}
```
