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

#![feature(plugin)]
#![plugin(interpolate_idents)]

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

extern crate libc;
extern crate regex;
extern crate libloading as ll;
extern crate memmap;

#[macro_use]
extern crate lazy_static;


use regex::Regex;

use std::sync::Mutex;
use std::collections::HashMap;

use std::io::prelude::*;

use std::io::BufReader;
use std::fs::File;

use std::hash::{Hash, Hasher};
use memmap::{Mmap, Protection};

use std::str;
use std::ffi::{CStr, CString};

macro_rules! setup_hook(
  ($name:ident, ($($arg_name:ident: $arg_ty:ty),*) -> $ret:ty, $code:block) => (
    interpolate_idents! {
      type [$name _type] = unsafe extern fn($($arg_name: $arg_ty),+) -> $ret;
    }

    interpolate_idents! {
      lazy_static! {
        static ref [$name _sym]: ll::Symbol<'static, unsafe extern fn($($arg_name: $arg_ty),+) -> $ret>
        = match unsafe { LIBANDROID.get(stringify!($name).as_bytes()) } {
            Ok(val) => val,
            Err(_) => panic!()
         };
      }
    }

    interpolate_idents! {
      #[[inline]]
      fn [$name _real]($($arg_name: $arg_ty),+) -> $ret {
        unsafe {
          [$name _sym]($($arg_name),+)
        }
      }
    }

    interpolate_idents! {
      #[[no_ mangle]]
      pub unsafe extern fn $name($($arg_name: $arg_ty),+) -> $ret {
        [$name _safe]($($arg_name),+)
      }
    }

    interpolate_idents! {
      #[[inline]]
      fn [$name _safe]($($arg_name: $arg_ty),+) -> $ret {
        $code
      }
    }

  )
);


extern {
  pub fn __android_log_print(prio: i32, tag: *const libc::c_char, fmt: *const libc::c_char, ...) -> libc::c_int;
}

pub struct AAssetManager {}
pub struct AAsset {}

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
pub struct AAssetHolder {
  asset: *const AAsset,
}
unsafe impl Send for AAssetHolder {}

#[derive(PartialEq, Eq, Hash)]
pub struct JAsset {
  asset: *const AAsset,
  size: usize,
  pos: usize,
  file: MmapHolder,
}
unsafe impl Send for JAsset {}



pub extern fn init() {
  unsafe {
    __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("assethook init").unwrap().as_ptr());
  }
}

#[link_section = ".ctors"]
pub static CONSTRUCTOR: extern fn() = init;


lazy_static! {
  static ref PKG_VALIDATOR: Regex = Regex::new(r"^[a-zA-Z_.]+$").unwrap();

  static ref HOOKMAP: Mutex<HashMap<AAssetHolder, JAsset>> = Mutex::new(HashMap::new());

  static ref LIBANDROID: ll::Library = {
    let libandroid_path = match std::mem::size_of::<usize>() {
      4 => "/system/lib/libandroid.so",
      8 => "/system/lib64/libandroid.so",
      _ => "/system/lib/libandroid.so"
    };


    unsafe {__android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("attempting to load ".to_string() + &libandroid_path).unwrap().as_ptr()); }

    match ll::Library::new(libandroid_path) {
      Ok(val) => {
        unsafe { __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("loaded libandroid.so").unwrap().as_ptr()); }
        val
      },
      Err(_) => {
        unsafe { __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("failed to load libandroid.so").unwrap().as_ptr()); }
        panic!()
      }
    }
  };
}

setup_hook!(AAssetManager_open, (mgr: *mut AAssetManager, filename: *const libc::c_char, mode: i32) -> *mut AAsset, {
  unsafe {
    __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("AAssetManager_open for %s").unwrap().as_ptr(), filename);
  }

  let pkg = get_pkg_name().unwrap();

  if !PKG_VALIDATOR.is_match(pkg.as_str()) {
    AAssetManager_open_real(mgr, filename, mode)
  } else {
    let fname = unsafe {
      CStr::from_ptr(filename).to_str().unwrap().to_owned()
    };
    let path_str = "/data/local/tmp/assethook/".to_string() + pkg.as_str() + "/assets/" + &fname;

    let canon = match std::fs::canonicalize(path_str) {
      Ok(c) => c,
      Err(_) => return AAssetManager_open_real(mgr, filename, mode) //verifies whether or not the file exists
    };

    if !canon.starts_with("/data/local/tmp/assethook/".to_string() + pkg.as_str() + "/") {
      return AAssetManager_open_real(mgr, filename, mode);
    }

    let asset = AAssetManager_open_real(mgr, filename, mode);

    let file_mmap = match Mmap::open_path(&canon, Protection::Read) {
      Ok(f) => f,
      Err(_) => return asset
    };

    let mut hookmap = HOOKMAP.lock().unwrap();
    hookmap.insert(
      AAssetHolder{ asset: asset },
      JAsset{
        asset: asset,
        size: file_mmap.len(),
        pos: 0,
        file: MmapHolder{ mmap: file_mmap }
      }
    );
    asset
  }
});




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


setup_hook!(AAsset_close, (asset: *mut AAsset) -> (), {
  let mut hookmap = HOOKMAP.lock().unwrap();
  match hookmap.remove(&AAssetHolder{asset: asset}) {
    Some(_) => (),
    None => ()
  };
  AAsset_close_real(asset);
});

setup_hook!(AAsset_getLength, (asset: *mut AAsset) -> libc::off_t, {
  let hookmap = HOOKMAP.lock().unwrap();
  let jasset: &JAsset = match hookmap.get(&AAssetHolder{asset: asset}) {
    Some(j) => j,
    None => return AAsset_getLength_real(asset)
  };
  jasset.size as libc::off_t
});

setup_hook!(AAsset_getLength64, (asset: *mut AAsset) -> libc::off64_t, {
  let hookmap = HOOKMAP.lock().unwrap();
  let jasset: &JAsset = match hookmap.get(&AAssetHolder{asset: asset}) {
    Some(j) => j,
    None => return AAsset_getLength64_real(asset)
  };
  jasset.size as libc::off64_t
});

setup_hook!(AAsset_getRemainingLength, (asset: *mut AAsset) -> libc::off_t, {
  let hookmap = HOOKMAP.lock().unwrap();
  let jasset: &JAsset = match hookmap.get(&AAssetHolder{asset: asset}) {
    Some(j) => j,
    None => return AAsset_getRemainingLength_real(asset)
  };
  (jasset.size - jasset.pos) as libc::off_t
});

setup_hook!(AAsset_getRemainingLength64, (asset: *mut AAsset) -> libc::off64_t, {
  let hookmap = HOOKMAP.lock().unwrap();
  let jasset: &JAsset = match hookmap.get(&AAssetHolder{asset: asset}) {
    Some(j) => j,
    None => return AAsset_getRemainingLength64_real(asset)
  };
  (jasset.size - jasset.pos) as libc::off64_t
});

setup_hook!(AAsset_read, (asset: *mut AAsset, buf: *mut libc::c_void, count: libc::size_t) -> libc::c_int, {
  let mut hookmap = HOOKMAP.lock().unwrap();
  let jasset: &mut JAsset = match hookmap.get_mut(&AAssetHolder{asset: asset}) {
    Some(j) => j,
    None => return AAsset_read_real(asset, buf, count)
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
  toread as libc::c_int //note: the api itself is broken on this, it takes size_t count, but returns int read
});

setup_hook!(AAsset_seek, (asset: *mut AAsset, offset: libc::off_t, whence: libc::c_int) -> libc::off_t, {
  let mut hookmap = HOOKMAP.lock().unwrap();
  let jasset: &mut JAsset = match hookmap.get_mut(&AAssetHolder{asset: asset}) {
    Some(j) => j,
    None => return AAsset_seek_real(asset, offset, whence)
  };

  match whence {
    libc::SEEK_SET => {
      if offset < 0 {
        return -1;
      }

      jasset.pos = offset as usize;
      return jasset.pos as libc::off_t;
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
      return newpos as libc::off_t;
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
      return newpos as libc::off_t;
    }
    _ => {
      return -1;
    }
  }
});

setup_hook!(AAsset_seek64, (asset: *mut AAsset, offset: libc::off64_t, whence: libc::c_int) -> libc::off64_t, {
  let mut hookmap = HOOKMAP.lock().unwrap();
  let jasset: &mut JAsset = match hookmap.get_mut(&AAssetHolder{asset: asset}) {
    Some(j) => j,
    None => return AAsset_seek64_real(asset, offset, whence)
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
});

setup_hook!(AAsset_getBuffer, (asset: *mut AAsset) -> *const libc::c_void, {
  let hookmap = HOOKMAP.lock().unwrap();
  let jasset: &JAsset = match hookmap.get(&AAssetHolder{asset: asset}) {
    Some(j) => j,
    None => return AAsset_getBuffer_real(asset)
  };
  jasset.file.mmap.ptr() as *const libc::c_void
});

setup_hook!(AAsset_isAllocated, (asset: *mut AAsset) -> libc::c_int, {
  let hookmap = HOOKMAP.lock().unwrap();
  match hookmap.get(&AAssetHolder{asset: asset}) {
    Some(_) => (),
    None => return AAsset_isAllocated_real(asset)
  };
  false as libc::c_int
});

setup_hook!(AAsset_openFileDescriptor, (asset: *mut AAsset, outStart: *mut libc::off_t, outLength: *mut libc::off_t) -> libc::c_int, {
  let hookmap = HOOKMAP.lock().unwrap();
  match hookmap.get(&AAssetHolder{asset: asset}) {
    Some(_) => return -1, //note: naive implementation is as easily detected as no implementation
    None => return AAsset_openFileDescriptor_real(asset, outStart, outLength)
  }
});

setup_hook!(AAsset_openFileDescriptor64, (asset: *mut AAsset, outStart: *mut libc::off64_t, outLength: *mut libc::off64_t) -> libc::c_int, {
  let hookmap = HOOKMAP.lock().unwrap();
  match hookmap.get(&AAssetHolder{asset: asset}) {
    Some(_) => return -1, //note: naive implementation is as easily detected as no implementation
    None => return AAsset_openFileDescriptor64_real(asset, outStart, outLength)
  }
});


