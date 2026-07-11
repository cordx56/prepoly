//! Standard-stream runtime primitives.
//!
//! File I/O lives in the fs library (a plugin over raw descriptors); the
//! runtime keeps only what the import-free prelude needs -- writing a string
//! to stdout (`pp_print_str`/`pp_println_str` in `crate::alloc`) and reading
//! bytes from stdin for `input()`.

use std::fs::File;
use std::io::Read;
use std::mem::ManuallyDrop;
use std::os::fd::FromRawFd;

use crate::alloc::{pp_arr_new, typed_result, typed_result_err};
use crate::rt::Header;

/// `_stdin_read(n) -> uint8[]!`: up to `n` bytes from standard input (fewer
/// at end-of-input). The primitive behind the prelude's `input()`, so reading
/// a line needs no `File` value.
pub extern "C-unwind" fn pp_stdin_read(n: i64) -> *mut Header {
    unsafe {
        // Borrow descriptor 0 without taking ownership, so the borrow ending
        // does not close it.
        let mut stdin = ManuallyDrop::new(File::from_raw_fd(0));
        let mut buf = vec![0u8; n.max(0) as usize];
        match stdin.read(&mut buf) {
            Ok(got) => {
                let arr = pp_arr_new(1, got as i64);
                let data = *((arr as *mut u8).offset(32) as *mut *mut u8);
                std::ptr::copy_nonoverlapping(buf.as_ptr(), data, got);
                typed_result(true, |p| *(p as *mut *mut Header) = arr)
            }
            Err(e) => typed_result_err(&e.to_string()),
        }
    }
}
