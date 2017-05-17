#!/bin/sh

adb shell su -c '"umask 002; mkdir -p /data/local/tmp/assethook; mkdir -p /data/local/tmp/lib; mkdir -p /data/local/tmp/lib64; chown root:shell /data/local/tmp/assethook; chown root:shell /data/local/tmp/lib; chown root:shell /data/local/tmp/lib64"'
adb push target/aarch64-linux-android/release/libaassethook_capi.so /data/local/tmp/lib64/
adb shell su -c 'chmod 755 /data/local/tmp/lib64/libaassethook_capi.so'
adb shell sha1sum /data/local/tmp/lib64/libaassethook_capi.so
shasum -a 1 target/aarch64-linux-android/release/libaassethook_capi.so
adb push target/arm-linux-androideabi/release/libaassethook_capi.so /data/local/tmp/lib/
adb shell su -c 'chmod 755 /data/local/tmp/lib/libaassethook_capi.so'
adb shell sha1sum /data/local/tmp/lib/libaassethook_capi.so
shasum -a 1 target/arm-linux-androideabi/release/libaassethook_capi.so
