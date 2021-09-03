0) branch
   * git checkout -b release_x.y.z
1) firma lib
   * cd lib
   * update lib/Cargo.toml, remove "-dev" suffix in version
   * cargo test
   * git add -u && git commit -m "bump lib to version x.y.z"
   * cargo publish
2) firma cli 
   * cd cli
   * update cli/Cargo.toml, remove "-dev" suffix in version
   * change firma lib dep from path to x.y.z
   * cargo test  
   * git add -u && git commit -m "bump cli to version x.y.z"
   * cargo publish
3) firma android   
   * update android/app/build.gradle versionCode and versionName to version 1.y
   * from lib `NDK=/Users/casatta/android-ndk-r21d HOST=darwin-x86_64 ./build-android/all.sh`
   * from Android Studio run UI tests (delete storage data from device if used for manual tests)
   * from Android Studio "Generate Signed App" for networkTestnetRelease (requires keystore password) select BOTH signature checkbox
   * git add -u && git commit -m "bump android to version 1.y"
4) github
   * git push github branch, wait CI pass
   * merge on master
   * git tag -s -a release_x.y.z -m "release x.y.z"
   * git push --tags github master
   * publish testnet signed app on android store
5) dev
   * update lib/Cargo.toml, bump version to x.y+1.0, add -dev
   * update cli/Cargo.toml, bump version to x.y+1.0, add -dev
   * update cli/Cargo.toml, change firma dep from crates to local path
   * git add -u && git commit -m "bump version to dev"