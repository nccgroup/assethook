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

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::string::String;

use std::io::prelude::*;
use std::io::BufReader;

extern crate regex;
use regex::Regex;


fn setup_hooks() {
  let setup_hook_re: Regex = Regex::new(
    r"^setup_hook!\(\s*(?P<name>\w+)\s*,"
  ).unwrap();

  let man_dir_s = env::var("CARGO_MANIFEST_DIR").unwrap();
  let src_dir = Path::new(&man_dir_s).join("src");

  let hooks_pre_rs_path = &src_dir.join("hooks_pre.rs");
  let hooks_pre_rs_file = File::open(&hooks_pre_rs_path).unwrap();

  let hooks_mid_rs_path = &src_dir.join("hooks_mid.rs");
  let mut hooks_mid_rs = File::create(&hooks_mid_rs_path).unwrap();
  let hooks_pre_rs_br = BufReader::new(&hooks_pre_rs_file);

  enum State {
    Off,
    On,
    Process
  };

  let mut s = State::Off;
  let mut setup_hook = String::new();
  for (_, line) in hooks_pre_rs_br.lines().enumerate() {
    let raw_line = &line.unwrap();
    let l = raw_line.trim();

    loop {
      match s {
        State::Off => {
          if l.starts_with("setup_hook!") {
            setup_hook = raw_line.to_string();
            setup_hook += "\n";
            if setup_hook.contains("{") {
              s = State::Process;
            } else {
              s = State::On;
              break;
            }
          } else {
            let _ = hooks_mid_rs.write_all(raw_line.as_bytes());
            let _ = hooks_mid_rs.write_all("\n".as_bytes());
            break;
          }
        }
        State::On => {
          setup_hook += raw_line;
          setup_hook += "\n";
          if setup_hook.contains("{") {
            s = State::Process;
          } else {
            break;
          }
        }

        State::Process => {
          for cap in setup_hook_re.captures_iter(&setup_hook) {
            let name = &cap[1];
            //wrap_dl!($name, LIBANDROID, ($($arg_name: $arg_ty),*) -> $ret);
            let comma = setup_hook.find(',').unwrap();
            let osquig = setup_hook.find('{').unwrap();
            let segment = &setup_hook[comma+1..osquig];
            let rcomma = segment.rfind(',').unwrap();


            let argtypes = &segment[..rcomma].trim();
            let wrap_dl_line = format!("wrap_dl!({}, LIBANDROID, {});\n", name, argtypes);
            let _ = hooks_mid_rs.write_all(wrap_dl_line.as_bytes());

            let mut v: std::vec::Vec<&str> = setup_hook.split(',').collect();

            let _ = v.remove(0);
            let tail = v.join(",");

            let mut setup_hook_inner_line = String::new();
            setup_hook_inner_line += "setup_hook_inner!(";
            setup_hook_inner_line += name;
            setup_hook_inner_line += ", ";
            setup_hook_inner_line += name;
            setup_hook_inner_line += "_safe,";
            setup_hook_inner_line += tail.as_str();
            let _ = hooks_mid_rs.write_all(setup_hook_inner_line.as_bytes());
            let _ = hooks_mid_rs.write_all("\n".as_bytes());
          }
          s = State::Off;
          break;
        }
      }
    }
  }
}

fn wrap_dls() {
  let wrap_dl_re: Regex = Regex::new(
    r"^wrap_dl!\(\s*(?P<name>\w+)\s*,\s*(?P<lib>\w+)\s*,\s*(?P<args>\(.*\))\s*->\s*(?P<ret>.*)\s*\);$"
  ).unwrap();

  let man_dir_s = env::var("CARGO_MANIFEST_DIR").unwrap();
  let src_dir = Path::new(&man_dir_s).join("src");

  let hooks_mid_rs_path = &src_dir.join("hooks_mid.rs");
  let hooks_mid_rs_file = File::open(&hooks_mid_rs_path).unwrap();

  let hooks_rs_path = &src_dir.join("hooks.rs");
  let mut hooks_rs = File::create(&hooks_rs_path).unwrap();
  let hooks_mid_rs_br = BufReader::new(&hooks_mid_rs_file);
  for (_, line) in hooks_mid_rs_br.lines().enumerate() {
    let raw_line = &line.unwrap();
    let l = raw_line.trim();

    if l.starts_with("wrap_dl!") {
      for cap in wrap_dl_re.captures_iter(l) {
        let name = &cap[1];
        let lib = &cap[2];
        let argtypes = &cap[3];
        let ret = &cap[4];

        //note(jtd): ignores whitespace for target
        let _ = hooks_rs.write_all("\n//".as_bytes());
        let _ = hooks_rs.write_all(raw_line.as_bytes());
        let _ = hooks_rs.write_all("\n".as_bytes());
        let _ = hooks_rs.write_all(format!("//type {name}_type = unsafe extern fn{argtypes} -> {ret};

lazy_static! {{
  pub static ref {name}_sym : ll::Symbol<'static, unsafe extern fn{argtypes} -> {ret}>
  = match unsafe {{ {lib}.get(\"{name}\".as_bytes()) }} {{
    Ok(val) => val,
    Err(_) => panic!()
  }};
}}

#[inline]
pub fn {name}_real{argtypes} -> {ret} {{
  unsafe {{
    call_func!({name}_sym, {argtypes})
  }}
}}
",
          name=name, lib=lib, argtypes=argtypes, ret=ret).as_bytes()
        );
        let _ = hooks_rs.write_all("//</wrap_dl!>\n".as_bytes());
        break;
      }
    } else {
      let _ = hooks_rs.write_all(raw_line.as_bytes());
      let _ = hooks_rs.write_all("\n".as_bytes());
    }
  }




}

fn main() {
  setup_hooks();
  wrap_dls();
  ()
}