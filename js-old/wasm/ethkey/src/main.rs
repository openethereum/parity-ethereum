// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

#![feature(lang_items, core_intrinsics)]
#![feature(start)]
#![feature(link_args)]
#![no_std]
use core::intrinsics;

// Pull in the system libc library for what crt0.o likely requires.
extern crate libc;
extern crate tiny_keccak;
extern crate tiny_secp256k1;

use tiny_secp256k1::{is_valid_secret, create_public_key, ECPointG};

// #[link_args = "-s EXPORTED_FUNCTIONS=['_input_ptr','_secret_ptr','_public_ptr','_address_ptr','_ecpointg','_verify_secret','_brain']"]
// extern {}

use tiny_keccak::Keccak;

pub trait Keccak256<T: Sized> {
    fn keccak256(&self) -> T;
}

impl Keccak256<[u8; 32]> for [u8] {
    #[inline]
    fn keccak256(&self) -> [u8; 32] {
        let mut keccak = Keccak::new_keccak256();
        let mut result = [0u8; 32];
        keccak.update(self);
        keccak.finalize(&mut result);
        result
    }
}

static mut INPUT: [u8; 1024] = [0; 1024];
static mut SECRET: [u8; 32] = [0; 32];
static mut PUBLIC: [u8; 64] = [0; 64];
static mut ADDRESS: [u8; 20] = [0; 20];
static mut G: Option<ECPointG> = None;

#[no_mangle]
pub extern "C" fn ecpointg() -> &'static ECPointG {
    let g = unsafe { &G };

    if let Some(ref g) = *g {
        return g;
    }

    unsafe { G = Some(ECPointG::new()) };
    g.as_ref().expect("value set above; qed")
}

#[no_mangle]
pub extern "C" fn input_ptr() -> *const u8 {
    unsafe { INPUT.as_ptr() }
}

#[no_mangle]
pub extern "C" fn secret_ptr() -> *const u8 {
    unsafe { SECRET.as_ptr() }
}

#[no_mangle]
pub extern "C" fn public_ptr() -> *const u8 {
    unsafe { PUBLIC.as_ptr() }
}

#[no_mangle]
pub extern "C" fn address_ptr() -> *const u8 {
    unsafe { ADDRESS.as_ptr() }
}

#[no_mangle]
pub extern "C" fn verify_secret() -> bool {
    is_valid_secret(unsafe { &SECRET })
}

#[no_mangle]
pub extern "C" fn brain(input_len: usize) {
    let data = unsafe { &INPUT[..input_len] };
    let mut secret_out = unsafe { &mut SECRET };
    let mut public_out = unsafe { &mut PUBLIC };
    let mut address_out = unsafe { &mut ADDRESS };

    let g = ecpointg();
    let mut secret = data.keccak256();

    let mut i = 0;
    loop {
        secret = secret.keccak256();

        match i > 16384 {
            false => i += 1,
            true => {
                if let Some(public) = create_public_key(g, &secret) {
                    let public = &public[1..];
                    let hash = public.keccak256();

                    address_out.copy_from_slice(&hash[12..]);

                    if address_out[0] == 0 {
                        public_out.copy_from_slice(&public);
                        secret_out.copy_from_slice(&secret);
                        return;
                    }
                }
            }
        }
    }
}

// Entry point for this program.
#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    0
}

// These functions are used by the compiler, but not
// for a bare-bones hello world. These are normally
// provided by libstd.
#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() {
}

// This function may be needed based on the compilation target.
#[lang = "eh_unwind_resume"]
#[no_mangle]
pub extern fn rust_eh_unwind_resume() {
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(_msg: core::fmt::Arguments,
                               _file: &'static str,
                               _line: u32) -> ! {
    unsafe { intrinsics::abort() }
}
