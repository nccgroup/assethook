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

/*
#[macro_export]
macro_rules! wrap_dl(
  ($name:ident, $lib:ident, ($($arg_name:ident: $arg_ty:ty),*) -> $ret:ty) => (
    interpolate_idents! {
      type [$name _type] = unsafe extern fn($($arg_name: $arg_ty),+) -> $ret;
    }

    interpolate_idents! {
      lazy_static! {
        pub static ref [$name _sym]: ll::Symbol<'static, unsafe extern fn($($arg_name: $arg_ty),+) -> $ret>
        = match unsafe { $lib.get(stringify!($name).as_bytes()) } {
            Ok(val) => val,
            Err(_) => panic!()
         };
      }
    }

    interpolate_idents! {
      #[[inline]]
      pub fn [$name _real]($($arg_name: $arg_ty),+) -> $ret {
        unsafe {
          [$name _sym]($($arg_name),+)
        }
      }
    }
  )
);

#[macro_export]
macro_rules! setup_hook(
  ($name:ident, ($($arg_name:ident: $arg_ty:ty),*) -> $ret:ty, $code:block) => (

    wrap_dl!($name, LIBANDROID, ($($arg_name: $arg_ty),*) -> $ret);

    interpolate_idents! {
      #[[no_ mangle]]
      pub unsafe extern fn $name($($arg_name: $arg_ty),+) -> $ret {
        [$name _safe]($($arg_name),+)
      }
    }

    interpolate_idents! {
      #[[inline]]
      pub fn [$name _safe]($($arg_name: $arg_ty),+) -> $ret {
        $code
      }
    }
  )
);
*/


#[macro_export]
macro_rules! wrap_dl(
  ($name:ident, $lib:ident, ($($arg_name:ident: $arg_ty:ty),*) -> $ret:ty) => ()
);

#[macro_export]
macro_rules! setup_hook(
  ($name:ident, ($($arg_name:ident: $arg_ty:ty),*) -> $ret:ty, $code:block) => ()
);

#[macro_export]
macro_rules! setup_hook_inner(
  ($name:ident, $name_safe:ident, ($($arg_name:ident: $arg_ty:ty),*) -> $ret:ty, $code:block) => (

    //wrap_dl!($name, LIBANDROID, ($($arg_name: $arg_ty),*) -> $ret);

    #[no_mangle]
    pub unsafe extern fn $name($($arg_name: $arg_ty),+) -> $ret {
      $name_safe($($arg_name),+)
    }

    #[inline]
    pub fn $name_safe($($arg_name: $arg_ty),+) -> $ret {
      $code
    }
  )
);
