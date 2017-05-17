#!/bin/sh

function hook32() {
  adb shell su -c "setprop wrap.$1 LD_PRELOAD=/data/local/tmp/lib/libaassethook_capi.so"
}

function hook64() {
  adb shell su -c "setprop wrap.$1 LD_PRELOAD=/data/local/tmp/lib64/libaassethook_capi.so"
}

function unhook() {
  adb shell su -c "setprop wrap.$1 "'"\"\""'
}

if [ $# -ne 2 ]; then
  echo "uasge: $0 <package> <32|64|unhook>"
  exit
fi

if [ $2 = "32" ]; then
  hook32 $1
elif [ $2 = "64" ]; then
  hook64 $1
elif [ $2 = "unhook" ]; then
  unhook $1
else
  echo "uasge: $0 <package> <32|64|unhook>"
  exit
fi
