# (A)AssetHook

(A)AssetHook is an `LD_PRELOAD`-based hooking library that allows for replacing
APK asset files dynamically without modifying an APK. It's also written in rust.
It redirects asset file loads to a filepath on an Android device under
 `/data/local/tmp/assethook/com.pkg.name`. Needless to say, due to this
(really SEAndroid's ridiculous DRM scheme that now blocks app access to
/data/local/tmp), AssetHook currently requires that the SELinux mode be set
to permissive. If the raw asset path (typically `assets/path/to/file.name`)
exists under the respective package's AssetHook path, the replacement will
be loaded instead. AssetHook consists of two separate `LD_PRELOAD` hook
implementations, one for the "public" C API, and one for the internal C++ API.
In general, use the C++ hook, but if it breaks on a new version of Android,
try the C one (just note the limitations of it described below).

## Why?

Because things like React Native use the native asset manager API, which
can't be hooked via things like Xposed.

## C++ API (`cppapi_assethook`) Hook

A "relatively stable" LD_PRELOAD-based hook that hooks the underlying C++ asset
implementation called by the public C API and the Java APIs among other things.
As this lower level API is occasionally used by other internal Android code to
load/check other static resources from APKs, it can also be used to replace
those files (if read via this API) by placing replacements under non-`assets/`
APK paths.

***Note:*** While files such as `res/layout/activity_main.xml` are at least
opened via this API, they are not read with it and cannot be replaced at
runtime with AssetHook.

## Legacy C API (`capi_aassethook`) Hook

*(Note the two 'a's in "aasset")*

A "conservative" `LD_PRELOAD`-based hook that won't hook the Java-level
`android.content.res.AssetManager`. As the C API it hooks (`AAsset*`) is public, this is
a stable hook to be used if new changes in the internal C++ implementation
break the C++ API hook. Using this hook, only files under
`/data/local/tmp/assethook/com.example.name/assets/` can be hooked.

# Usage

1. Install AAssetHook and enable it for a particular app package
1. Push (as `shell` to make sure the file is world-readable) replacement files:

    ```bash
    adb push index.android.bundle.mod /data/local/tmp/assethook/com.pkg.name/assets/index.android.bundle
    ```
1. Restart the hooked app

# Building

***Note:*** Shared objects for 32-bit and 64-bit arm are embedded in the repository.

***Note:*** The C API hooking `AAssetHook` uses some rust syntax extensions to cut down on a tremendous amount of
boilerplate, and therefore needs to be compile using rust nightly. It is known
to build using rust at commit `daf8c1dfc` from `2016-12-05` (in case internal
changes in the rust compiler break the build).

1. Install the Android NDK
1. Install rust via rustup (<https://github.com/rust-lang-nursery/rustup.rs>)
1. Run the following commands:

    ```bash
    rustup install nightly # C API hook only
    rustup default nightly # C API hook only
    rustup target add aarch64-linux-android
    rustup target add arm-linux-androideabi
    rustup target add i686-linux-android # AVD/HAXM
    mkdir ~/.cargo/toolchains
    cd ~/.cargo/toolchains
    /path/to/ndk/build/tools/make-standalone-toolchain.sh \
      --platform=android-22 --toolchain=arm-linux-android-4.9 \
      --install-dir=android-22-arm-toolchain`
    /path/to/ndk/build/tools/make-standalone-toolchain.sh \
      --platform=android-22 --toolchain=aarch64-linux-android-4.9 \
      --install-dir=android-22-aarch64-toolchain`
    /path/to/ndk/build/tools/make-standalone-toolchain.sh \
      --platform=android-22 --toolchain=x86-linux-android-4.9 \
      --install-dir=android-22-x86-toolchain`
    ```
1. Add the following to `~/.cargo/config`

    ```
    [target.arm-linux-androideabi]
    linker = "/path/to/home/.cargo/toolchains/android-22-arm-toolchain/bin/clang"

    [target.aarch64-linux-android]
    linker = "/path/to/home/.cargo/toolchains/android-22-aarch64-toolchain/bin/clang"
    
    [target.i686-linux-android]
    linker = "/path/to/home/.cargo/toolchains/android-22-x86-toolchain/bin/clang"
    ```
1. `cd /path/to/aassethook/cppapi`
1. `cargo build --target=arm-linux-androideabi --release`
1. `cargo build --target=aarch64-linux-android --release`
1. `cargo build --target=i686-linux-android --release`

***Note:*** The `build.sh` script in the `cppapi` directory performs the last three commands.

# Installing

***Note:*** If falling back to the `capi` implementation, swap all instances of
`cppapi` for `capi`.

First install SELinuxModeChanger from
[F-Droid](https://f-droid.org/repository/browse/?fdfilter=selinux&fdid=com.mrbimc.selinux)
or [MrBIMC/SELinuxModeChanger](https://github.com/MrBIMC/SELinuxModeChanger),
and set it to permissive mode.

***Note:*** You can also just run `/system/bin/setenforce 0` after each boot.

```bash
cd cppapi
./install.sh
```

# Enabling

***Note:*** If falling back to the `capi` implementation, swap all instances of
`cppapi` for `capi`.

```bash
cd cppapi
./hook.sh <pkg.name> <32|64|unhook>
```

# Future Work

* Migrate hook file storage to an application's local file directory to bypass the need for SEAndroid disabling
* Resource hooking
