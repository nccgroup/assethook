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

#![allow(non_snake_case)]

extern crate libc;
extern crate libloading as ll;

use std;
use asset::*;
use externs::*;
use std::ffi::{CStr, CString};

use regex::Regex;

use std::sync::Mutex;
use std::collections::HashMap;

use std::io::prelude::*;

use std::io::BufReader;
use std::fs::File;

use std::hash::{Hash, Hasher};
use memmap::{Mmap, Protection};
use std::error::Error;

#[macro_export]
macro_rules! call_func(
    ($name:ident, ($($arg_name:ident: $arg_ty:ty),*)) => {
      $name($($arg_name),+)
    }
);

wrap_dl!(AAsset_getLength, LIBANDROID, (asset: *const AAsset) -> libc::off_t);
wrap_dl!(AAsset_getRemainingLength, LIBANDROID, (asset: *const AAsset) -> libc::off_t);
wrap_dl!(AAsset_isAllocated, LIBANDROID, (asset: *const AAsset) -> libc::c_int);
wrap_dl!(AAsset_openFileDescriptor, LIBANDROID, (asset: *mut AAsset, outStart: *mut libc::off_t, outLength: *mut libc::off_t) -> libc::c_int);
wrap_dl!(AAsset_read, LIBANDROID, (asset: *mut AAsset, buf: *mut libc::c_void, count: libc::size_t) -> libc::c_int);
wrap_dl!(AAsset_seek, LIBANDROID, (this: *mut AAsset, offset: libc::off_t, whence: libc::c_int) -> libc::off_t);
wrap_dl!(AAsset_getBuffer, LIBANDROID, (asset: *mut AAsset) -> *const libc::c_void);
wrap_dl!(AAsset_close, LIBANDROID, (asset: *mut AAsset) -> ());

pub struct MmapHolder {
  mmap: Mmap
}

impl Hash for MmapHolder {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.mmap.ptr().hash(state);
  }
}

impl PartialEq for MmapHolder {
  fn eq(&self, other: &MmapHolder) -> bool {
    self.mmap.ptr() == other.mmap.ptr()
  }
}

impl Eq for MmapHolder {

}

#[derive(PartialEq, Eq, Hash)]
pub struct AssetHolder {
  asset: *const Asset,
}
unsafe impl Send for AssetHolder {}

#[derive(PartialEq, Eq, Hash)]
pub struct JAsset {
  asset: *const Asset,
  size: usize,
  pos: usize,
  file: MmapHolder,
  ovt: usize
}
unsafe impl Send for JAsset {}



pub extern fn getLength(this: *const Asset) -> libc::off64_t {
  if cfg!(debug_assertions) {
    unsafe {
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("getLength hook").unwrap().as_ptr());
    }
  }

  let hookmap = HOOKMAP.lock().unwrap();
  let jasset: &JAsset = match hookmap.get(&AssetHolder{asset: this}) {
    Some(j) => j,
    None => return 0
  };
  jasset.size as libc::off64_t
}

pub extern fn getRemainingLength(this: *const Asset) -> libc::off64_t {
  if cfg!(debug_assertions) {
    unsafe {
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("getRemainingLength hook").unwrap().as_ptr());
    }
  }

  let hookmap = HOOKMAP.lock().unwrap();
  let jasset: &JAsset = match hookmap.get(&AssetHolder{asset: this}) {
    Some(j) => j,
    None => return 0
  };
  (jasset.size-jasset.pos) as libc::off64_t
}

pub extern fn isAllocated(_this: *const Asset) -> bool {
  if cfg!(debug_assertions) {
    unsafe {
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("isAllocated hook").unwrap().as_ptr());
    }
  }
  false
}

pub extern fn openFileDescriptor(_this: *const Asset, _outStart: *mut libc::off64_t, _outLength: *mut libc::off64_t) -> libc::c_int {
  if cfg!(debug_assertions) {
    unsafe {
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("openFileDescriptor hook").unwrap().as_ptr());
    }
  }
  -1
}

pub extern fn read(this: *mut Asset, buf: *mut libc::c_void, count: libc::size_t) -> libc::ssize_t {
  if cfg!(debug_assertions) {
    unsafe {
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("read hook").unwrap().as_ptr());
    }
  }

  let mut hookmap = HOOKMAP.lock().unwrap();
  let jasset: &mut JAsset = match hookmap.get_mut(&AssetHolder{asset: this}) {
    Some(j) => j,
    None => return -1
  };

  let rem = jasset.size - jasset.pos;
  if rem == 0 {
    return 0;
  }

  let toread: usize = if count > rem {
    rem
  } else {
    count
  };

  unsafe {
    std::ptr::copy_nonoverlapping(
      jasset.file.mmap.ptr().offset(jasset.pos as isize),
      buf as *mut u8,
      toread
    )
  };
  jasset.pos += toread;
  toread as libc::ssize_t //note: the api itself is broken on this, it takes size_t count, but returns int read
}


pub extern fn seek(this: *mut Asset, offset: libc::off64_t, whence: libc::c_int) -> libc::off64_t {
  if cfg!(debug_assertions) {
    unsafe {
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("seek hook").unwrap().as_ptr());
    }
  }

  let mut hookmap = HOOKMAP.lock().unwrap();
  let jasset: &mut JAsset = match hookmap.get_mut(&AssetHolder{asset: this}) {
    Some(j) => j,
    None => return -1
  };

  match whence {
    libc::SEEK_SET => {
      if offset < 0 {
        return -1;
      }

      jasset.pos = offset as usize;
      return jasset.pos as libc::off64_t;
    },
    libc::SEEK_CUR => {
      let absoff = offset.abs() as usize;
      let newpos = if offset > 0 {
        match jasset.pos.checked_add(absoff) {
          Some(n) => n,
          None => return -1
        }
      } else {
        match jasset.pos.checked_sub(absoff) {
          Some(n) => n,
          None => return -1
        }
      };
      jasset.pos = newpos;
      return newpos as libc::off64_t;
    },
    libc::SEEK_END => {
      if offset > 0 {
        return -1;
      }

      let absoff = offset.abs() as usize;
      let newpos = match jasset.size.checked_sub(absoff) {
        Some(n) => n,
        None => return -1
      };
      jasset.pos = newpos;
      return newpos as libc::off64_t;
    }
    _ => {
      return -1;
    }
  }
}


pub extern fn getBuffer(asset: *mut Asset) -> *const libc::c_void {
  if cfg!(debug_assertions) {
    unsafe {
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("getBuffer hook").unwrap().as_ptr());
    }
  }

  let hookmap = HOOKMAP.lock().unwrap();
  let jasset: &JAsset = match hookmap.get(&AssetHolder{asset: asset}) {
    Some(j) => j,
    None => return 0usize as *const libc::c_void
  };
  jasset.file.mmap.ptr() as *const libc::c_void
}


pub extern fn close(this: *mut Asset) -> () {
  if cfg!(debug_assertions) {
    unsafe {
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("close hook").unwrap().as_ptr());
    }
  }

  let mut hookmap = HOOKMAP.lock().unwrap();
  let jasset = match hookmap.remove(&AssetHolder{asset: this}) {
    Some(j) => j,
    None => return
  };

  {
    //no point in juggling function pointers, just swap back the old vtable
    let a: &mut Asset = unsafe { &mut *this };
    a.vtable = jasset.ovt as *mut AssetVTable;
  }

  let aaptr = unsafe { libc::malloc(std::mem::size_of::<AAsset>()) } as *mut AAsset;
  {
    let aa = unsafe { &mut *aaptr };
    aa.mAsset = this;
  }
  AAsset_close_real(aaptr)
}


fn get_top(p: *const libc::c_void) -> *const libc::c_void {
  let pp = p as *const *const isize;
  unsafe {
    let vtp: *const isize = *pp;
    let offset_to_top: isize = *vtp.offset(-2);
    if offset_to_top == 0 {
      p
    } else {
      get_top(p.offset(offset_to_top))
    }
  }
}

#[inline]
fn hook_asset_vtable(asset: *mut Asset) {
  let vtp = {
    //let mut file_asset_vtable = FILE_ASSET_VTABLE.lock().unwrap();
    let mut file_asset_vtable_d = FILE_ASSET_VTABLE_D.lock().unwrap();
    let vtable_hooks = VTABLE_HOOKS.lock().unwrap();

    //if file_asset_vtable.len() == 0 {
    if file_asset_vtable_d.len() == 0 {

      let at = unsafe {
        &*(get_top(asset as *const libc::c_void) as *mut Asset)
      };

      let atvtp = unsafe { & *(at.vtable) } as *const AssetVTable as *const libc::c_void;

      let vhead: *const libc::c_void = unsafe { atvtp.offset((2 * std::mem::size_of::<usize>()) as isize * -1) };

      let gap = (atvtp as usize) - (vhead as usize);

      let sz = gap + std::mem::size_of::<AssetVTable>();

      let mut block = std::vec::Vec::new();

      block.resize(sz, 0u8);

      unsafe {
        std::ptr::copy_nonoverlapping(vhead as *const u8, block.as_mut_ptr(), sz);
      }

      let bnvt: &mut AssetVTable = unsafe { &mut *(block.as_mut_ptr().offset(gap as isize) as *mut AssetVTable) };

      //let a = unsafe { &*asset };
      //let avt = unsafe { & *(a.vtable) };
      //let mut nvt: AssetVTable = avt.clone();

      bnvt.elements[vtable_hooks.getLength.0] = vtable_hooks.getLength.1 as *const usize as usize;
      bnvt.elements[vtable_hooks.getRemainingLength.0] = vtable_hooks.getRemainingLength.1 as *const usize as usize;
      bnvt.elements[vtable_hooks.isAllocated.0] = vtable_hooks.isAllocated.1 as *const usize as usize;
      bnvt.elements[vtable_hooks.openFileDescriptor.0] = vtable_hooks.openFileDescriptor.1 as *const usize as usize;
      bnvt.elements[vtable_hooks.read.0] = vtable_hooks.read.1 as *const usize as usize;
      bnvt.elements[vtable_hooks.seek.0] = vtable_hooks.seek.1 as *const usize as usize;
      bnvt.elements[vtable_hooks.close.0] = vtable_hooks.close.1 as *const usize as usize;

      //file_asset_vtable.push({
      //  nvt
      //});
      file_asset_vtable_d.push( VTBlockHolder {
        block: block,
        gap: gap
      });
    }
    //let vt: &mut AssetVTable = file_asset_vtable.get_mut(0).unwrap();
    //vt as *mut AssetVTable
    let vtbh: &mut VTBlockHolder = file_asset_vtable_d.get_mut(0).unwrap();
    let bnvtp: &mut AssetVTable = unsafe { &mut *(vtbh.block.as_mut_ptr().offset(vtbh.gap as isize) as *mut AssetVTable) };
    bnvtp as *mut AssetVTable
  };

  let a = unsafe { &mut *asset };
  a.vtable = vtp;
}


//note: we have do pull some shenanigans here due to rust reading in multiple
//      nul chars from procfs reads
pub fn get_pkg_name() -> Option<String> {
  match File::open("/proc/self/cmdline") {
    Ok(file) => {
      let br = BufReader::new(&file);
      let mut ret = String::new();
      for line in br.lines() {
        for c in line.unwrap().chars() {
          if c == '\0' {
            return Some(ret);
          }
          ret.push(c);
        }
      }
      None
    },
    Err(_) => None
  }
}

lazy_static! {
  static ref PKG_VALIDATOR: Regex = Regex::new(r"^[a-zA-Z_.]+$").unwrap();

  static ref HOOKMAP: Mutex<HashMap<AssetHolder, JAsset>> = Mutex::new(HashMap::new());
}


//setup_hook!(_ZN7android10_FileAssetC1Ev, (asset: *mut Asset) -> (), {
//  unsafe {
//    __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("_ZN7android10_FileAssetC1Ev hook").unwrap().as_ptr());
//  }
//  _ZN7android10_FileAssetC1Ev_real(asset);
//
//  //hook_asset_vtable(asset);
//});
//
//
//setup_hook!(_ZN7android16_CompressedAssetC1Ev, (asset: *mut Asset) -> (), {
//  unsafe {
//    __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("_ZN7android16_CompressedAssetC1Ev hook").unwrap().as_ptr());
//  }
//  _ZN7android16_CompressedAssetC1Ev_real(asset);
//
//  //hook_asset_vtable(asset);
//});
//

//Asset* AssetManager::openNonAssetInPathLocked(const char* fileName, AccessMode mode, const asset_path& ap)
setup_hook!(_ZN7android12AssetManager24openNonAssetInPathLockedEPKcNS_5Asset10AccessModeERKNS0_10asset_pathE,
  (this: *mut AssetManager, filename: *const /*libc::c_char*/u8, mode: i32, ap: *const asset_path) -> *mut Asset, {

  let pkg = get_pkg_name().unwrap();

  if !PKG_VALIDATOR.is_match(pkg.as_str()) {

    _ZN7android12AssetManager24openNonAssetInPathLockedEPKcNS_5Asset10AccessModeERKNS0_10asset_pathE_real(this, filename, mode, ap)
  } else {
    let fname = unsafe {
      CStr::from_ptr(filename).to_str().unwrap().to_owned()
    };
    let path_str = "/data/local/tmp/assethook/".to_string() + pkg.as_str() + "/" + &fname;

    let canon = match std::fs::canonicalize(&path_str) {
      Ok(c) => c,
      Err(e) => {
        if cfg!(debug_assertions) {
          if !e.description().starts_with("entity not found") {
            unsafe {
              __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("err: %s").unwrap().as_ptr(), e.description());
            }
          }
        }

        //note: this verifies whether or not the file exists
        return _ZN7android12AssetManager24openNonAssetInPathLockedEPKcNS_5Asset10AccessModeERKNS0_10asset_pathE_real(this, filename, mode, ap)
      }
    };

    if !canon.starts_with("/data/local/tmp/assethook/".to_string() + pkg.as_str() + "/") {
      return _ZN7android12AssetManager24openNonAssetInPathLockedEPKcNS_5Asset10AccessModeERKNS0_10asset_pathE_real(this, filename, mode, ap);
    }


    let wanted_path = "/data/app/".to_string() + pkg.as_str() + "-";

    let apath = unsafe {& *ap };
    if ! std::str::from_utf8(unsafe { CStr::from_ptr(apath.string8_path) }.to_bytes()).unwrap()
         .starts_with(wanted_path.as_str()) {
      return _ZN7android12AssetManager24openNonAssetInPathLockedEPKcNS_5Asset10AccessModeERKNS0_10asset_pathE_real(this, filename, mode, ap);
    }

    unsafe {
      __android_log_print(6, CString::new("NCC").unwrap().as_ptr(),
                          CString::new("assethook: hooking %s").unwrap().as_ptr(), filename);
    }

    let asset = _ZN7android12AssetManager24openNonAssetInPathLockedEPKcNS_5Asset10AccessModeERKNS0_10asset_pathE_real(this, filename, mode, ap);

    if cfg!(debug_assertions) {
      unsafe {
        __android_log_print(6, CString::new("NCC").unwrap().as_ptr(),
                            CString::new("asset addr: %p").unwrap().as_ptr(), asset);
      }
    }

    //if the asset isn't in the apk or if the asset manager detects directory traversal
    if asset.is_null() {
      return asset;
    }

    let file_mmap = match Mmap::open_path(&canon, Protection::Read) {
      Ok(f) => f,
      Err(_) => {
        return asset
      }
    };

    let ovt = {
      let mut a = unsafe { &mut *asset };
      let ovt = a.vtable as usize;
      hook_asset_vtable(a);
      ovt
    };


    let mut hookmap = HOOKMAP.lock().unwrap();
    hookmap.insert(
      AssetHolder{ asset: asset },
       JAsset {
        asset: asset,
        size: file_mmap.len(),
        pos: 0,
        file: MmapHolder{ mmap: file_mmap },
        ovt: ovt
      }
    );
    asset
  }
});


////Asset* AssetManager::openAssetFromFileLocked(const String8& fileName, AccessMode mode);
//setup_hook!(_ZN7android12AssetManager23openAssetFromFileLockedERKNS_7String8ENS_5Asset10AccessModeE,
//(this: *mut AssetManager, fileName: *const libc::c_char, mode: i32) -> *mut Asset, {
//  unsafe {
//    __android_log_print(6, CString::new("NCC").unwrap().as_ptr(),
//                        CString::new("_ZN7android12AssetManager23openAssetFromFileLockedERKNS_7String8ENS_5Asset10AccessModeE").unwrap().as_ptr());
//  }
//
//  unsafe {
//    __android_log_print(6, CString::new("NCC").unwrap().as_ptr(),
//                        CString::new("%s").unwrap().as_ptr(), fileName);
//  }
//
//  _ZN7android12AssetManager23openAssetFromFileLockedERKNS_7String8ENS_5Asset10AccessModeE_real(this, fileName, mode)
//});
//
//
////Asset* AssetManager::openInPathLocked(const char* fileName, AccessMode mode, const asset_path& ap)
//setup_hook!(_ZN7android12AssetManager16openInPathLockedEPKcNS_5Asset10AccessModeERKNS0_10asset_pathE,
//(this: *mut AssetManager, fileName: *const libc::c_char, mode: i32, ap: &asset_path) -> *mut Asset, {
//  unsafe {
//    __android_log_print(6, CString::new("NCC").unwrap().as_ptr(),
//                        CString::new("_ZN7android12AssetManager16openInPathLockedEPKcNS_5Asset10AccessModeERKNS0_10asset_pathE").unwrap().as_ptr());
//  }
//
//  unsafe {
//    __android_log_print(6, CString::new("NCC").unwrap().as_ptr(),
//                        CString::new("%s").unwrap().as_ptr(), fileName);
//  }
//
//  _ZN7android12AssetManager16openInPathLockedEPKcNS_5Asset10AccessModeERKNS0_10asset_pathE_real(this, fileName, mode, ap)
//});
//
////Asset* AssetManager::openIdmapLocked(const struct asset_path& ap) const
//setup_hook!(_ZNK7android12AssetManager15openIdmapLockedERKNS0_10asset_pathE, (this: *const AssetManager, ap: &asset_path) -> *mut Asset, {
//  unsafe {
//    __android_log_print(6, CString::new("NCC").unwrap().as_ptr(),
//    CString::new("_ZNK7android12AssetManager15openIdmapLockedERKNS0_10asset_pathE").unwrap().as_ptr());
//
//
//     __android_log_print(6, CString::new("NCC").unwrap().as_ptr(),
//    CString::new("ap.idmap: %x").unwrap().as_ptr(), ap.typ);
//
//     __android_log_print(6, CString::new("NCC").unwrap().as_ptr(),
//    CString::new("ap.idmap: %p %s").unwrap().as_ptr(), ap.string8_idmap, ap.string8_idmap);
//  }
//
//  let a = _ZNK7android12AssetManager15openIdmapLockedERKNS0_10asset_pathE_real(this, ap);
//
//    unsafe {
//
//     __android_log_print(6, CString::new("NCC").unwrap().as_ptr(),
//    CString::new("a: %p").unwrap().as_ptr(), a);
//  }
//  a
//});
