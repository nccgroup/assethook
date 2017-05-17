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
extern crate libloading as ll;

use std;

use std::ffi::{/*CStr,*/ CString};

extern {
  pub fn __android_log_print(prio: i32, tag: *const /*libc::c_char*/u8, fmt: *const /*libc::c_char*/u8, ...) -> libc::c_int;
}

lazy_static! {
  pub static ref LIBANDROID: ll::Library = {
    let libandroid_path = match std::mem::size_of::<usize>() {
      4 => "/system/lib/libandroid.so",
      8 => "/system/lib64/libandroid.so",
      _ => "/system/lib/libandroid.so"
    };

    if cfg!(debug_assertions) {
      unsafe {
        __android_log_print(6, CString::new("NCC").unwrap().as_ptr(),
                            CString::new("attempting to load ".to_string() + &libandroid_path).unwrap().as_ptr());
        }
    }

    match ll::Library::new(libandroid_path) {
      Ok(val) => {
        if cfg!(debug_assertions) {
          unsafe { __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("loaded libandroid.so").unwrap().as_ptr()); }
        }
        val
      },
      Err(_) => {
        unsafe { __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("failed to load libandroid.so").unwrap().as_ptr()); }
        panic!()
      }
    }
  };

/*
  pub static ref LIBCPP: ll::Library = {
    let libcpp_path = match std::mem::size_of::<usize>() {
      4 => "/system/lib/libc++.so",
      8 => "/system/lib64/libc++.so",
      _ => "/system/lib/libc++.so"
    };

    if cfg!(debug_assertions) {
      unsafe {
        __android_log_print(6, CString::new("NCC").unwrap().as_ptr(),
                            CString::new("attempting to load ".to_string() + &libcpp_path).unwrap().as_ptr());
        }
    }

    match ll::Library::new(libcpp_path) {
      Ok(val) => {
        if cfg!(debug_assertions) {
          unsafe { __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("loaded libc++.so").unwrap().as_ptr()); }
        }
        val
      },
      Err(_) => {
        unsafe { __android_log_print(6, CString::new("NCC").unwrap().as_ptr(), CString::new("failed to load libc++.so").unwrap().as_ptr()); }
        panic!()
      }
    }
  };
*/
}




