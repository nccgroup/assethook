/*
Copyright (c) 2017 NCC Group
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions
are met:
1. Redistributions of source code must retain the above copyright
   notice, this list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright
   notice, this list of conditions and the following disclaimer in the
   documentation and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
SUCH DAMAGE.
*/

extern crate libc;

use std;
use std::sync::Mutex;

use hooks;

pub struct AssetManager {}

#[repr(C)]
pub struct AAsset {
  pub mAsset: *mut Asset
}

#[repr(C)]
pub struct Asset {
  pub vtable: *mut AssetVTable
}

#[repr(C)]
pub struct asset_path {
  //http://androidxref.com/6.0.1_r10/xref/system/core/include/utils/String8.h
  pub string8_path: *const /*libc::c_char*/u8,
  pub typ: i32,
  pub string8_idmap: *const /*libc::c_char*/u8,
  pub isSystemOverlay: bool
}

#[repr(C)]
#[derive(Clone)] //just a tinge of buffer overread (from a static/safe-ish part of memory ;)
pub struct AssetVTable {
  pub elements: [usize; 20],
  pub elements2: [usize; 20], //padding
}

#[repr(C)]
pub struct AssetVTableHooks {
  pub getLength: (usize, extern fn(*const Asset) -> libc::off64_t),
  pub getRemainingLength: (usize, extern fn(*const Asset) -> libc::off64_t),
  pub isAllocated: (usize, extern fn(*const Asset) -> bool),
  pub openFileDescriptor: (usize, extern fn(*const Asset, *mut libc::off64_t,*mut libc::off64_t) -> libc::c_int),
  pub read: (usize, extern fn(*mut Asset, *mut libc::c_void, libc::size_t) -> libc::ssize_t),
  pub seek: (usize, extern fn(*mut Asset, libc::off64_t, libc::c_int) -> libc::off64_t),
  pub getBuffer: (usize, extern fn(*mut Asset) -> *const libc::c_void),
  pub close: (usize, extern fn(*mut Asset) -> ()),
}

#[repr(C)]
pub struct VTBlockHolder {
  pub block: std::vec::Vec<u8>,
  pub gap: usize
}

lazy_static! {
  pub static ref VTABLE_HOOKS: Mutex<AssetVTableHooks> = Mutex::new(AssetVTableHooks {
    getLength: (0xff, hooks::getLength),
    getRemainingLength: (0xff, hooks::getRemainingLength),
    isAllocated: (0xff, hooks::isAllocated),
    openFileDescriptor: (0xff, hooks::openFileDescriptor),
    read: (0xff, hooks::read),
    seek: (0xff, hooks::seek),
    getBuffer: (0xff, hooks::getBuffer),
    close: (0xff, hooks::close),
  });

  //note: current fake vtable strategy should "just werk" w/o separating between subclass vtables
  //pub static ref FILE_ASSET_VTABLE: Mutex<std::vec::Vec<AssetVTable>> = Mutex::new(std::vec::Vec::new());
  pub static ref FILE_ASSET_VTABLE_D: Mutex<std::vec::Vec<VTBlockHolder>> = Mutex::new(std::vec::Vec::new());
  //pub static ref COMPRESSED_ASSET_VTABLE: Mutex<std::vec::Vec<AssetVTable>> = Mutex::new(std::vec::Vec::new());
}
