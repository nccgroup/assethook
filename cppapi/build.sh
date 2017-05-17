#!/bin/sh

cargo build --target=arm-linux-androideabi --release
cargo build --target=aarch64-linux-android --release
cargo build --target=i686-linux-android --release
