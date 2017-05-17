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

use asset::*;
use hooks::*;
use externs::*;

use std::ffi::CString;


#[repr(C)]
pub struct FakeAAsset<'a> {
  pub mAsset: &'a mut FakeAsset
}

#[repr(C)]
pub struct FakeAsset {
  pub vtable: &'static FakeAssetVTable,
  pub fakedata: [usize; 100], //padding to limit accidental memory corruption
}

#[repr(C)]
pub struct FakeAssetVTable {
  pub slots: [fn(*mut FakeAsset) -> usize; 20]
}

pub struct LastCalled {
  pub slot: usize
}

lazy_static! {
  pub static ref LAST_CALLED: Mutex<LastCalled> = Mutex::new(LastCalled { slot: 0xff });
}

fn call0(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 0;
  0
}

fn call1(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 1;
  0
}

fn call2(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 2;
  0
}

fn call3(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 3;
  0
}

fn call4(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 4;
  0
}

fn call5(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 5;
  0
}

fn call6(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 6;
  0
}

fn call7(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 7;
  0
}

fn call8(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 8;
  0
}

fn call9(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 9;
  0
}

fn call10(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 10;
  0
}

fn call11(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 11;
  0
}

fn call12(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 12;
  0
}

fn call13(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 13;
  0
}

fn call14(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 14;
  0
}

fn call15(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 15;
  0
}

fn call16(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 16;
  0
}

fn call17(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 17;
  0
}

fn call18(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 18;
  0
}

fn call19(_this: *mut FakeAsset) -> usize {
  let mut last_called = LAST_CALLED.lock().unwrap();
  last_called.slot = 19;
  0
}

pub static fake_asset_vtable: FakeAssetVTable = FakeAssetVTable {
  slots: [
    call0, call1, call2, call3, call4, call5, call6, call7, call8, call9,
    call10,call11,call12,call13,call14,call15,call16,call17,call18,call19
  ]
};


pub fn probe_vtable_slots(hooks: &mut AssetVTableHooks) {
  //note: we have to do libc allocation b/c AAsset_close calls delete
  //      and the AAsset destructor calls delete on mAsset
  let faamptr = unsafe { libc::calloc(1, std::mem::size_of::<FakeAAsset>()) } as *mut FakeAAsset;
  let famptr = unsafe { libc::calloc(1, std::mem::size_of::<FakeAsset>()) } as *mut FakeAsset;

  let faam = unsafe { &mut *faamptr };
  let fam = unsafe { &mut *famptr };

  fam.vtable = &fake_asset_vtable;

  faam.mAsset = fam;

  let aa = faamptr as *mut AAsset;
  AAsset_getLength_real(aa);
  hooks.getLength.0 = {
    LAST_CALLED.lock().unwrap().slot
  };

  AAsset_getRemainingLength_real(aa);
  hooks.getRemainingLength.0 = {
    LAST_CALLED.lock().unwrap().slot
  };

  AAsset_isAllocated_real(aa);
  hooks.isAllocated.0 = {
    LAST_CALLED.lock().unwrap().slot
  };

  let mut start: libc::off_t = 0;
  let mut length: libc::off_t = 0;
  AAsset_openFileDescriptor_real(aa, &mut start as *mut libc::off_t, &mut length as *mut libc::off_t);
  hooks.openFileDescriptor.0 = {
    LAST_CALLED.lock().unwrap().slot
  };

  AAsset_read_real(aa, 0 as *mut libc::c_void, 0);
  hooks.read.0 = {
    LAST_CALLED.lock().unwrap().slot
  };

  AAsset_seek_real(aa, 0, 0);
  hooks.seek.0 = {
    LAST_CALLED.lock().unwrap().slot
  };

  AAsset_getBuffer_real(aa);
  hooks.getBuffer.0 = {
    LAST_CALLED.lock().unwrap().slot
  };

  AAsset_close_real(aa);
  hooks.close.0 = {
    LAST_CALLED.lock().unwrap().slot
  };

  if cfg!(debug_assertions) {
    unsafe {
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("getLength => slot: %u").unwrap().as_ptr(), hooks.getLength.0);
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("getRemainingLength => slot: %u").unwrap().as_ptr(), hooks.getRemainingLength.0);
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("isAllocated => slot: %u").unwrap().as_ptr(), hooks.isAllocated.0);
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("openFileDescriptor => slot: %u").unwrap().as_ptr(), hooks.openFileDescriptor.0);
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("read => slot: %u").unwrap().as_ptr(), hooks.read.0);
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("seek => slot: %u").unwrap().as_ptr(), hooks.seek.0);
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("getBuffer => slot: %u").unwrap().as_ptr(), hooks.getBuffer.0);
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("close => slot: %u").unwrap().as_ptr(), hooks.close.0);
    }
  }
}


