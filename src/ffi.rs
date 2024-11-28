use std::ffi::{c_char, CStr};
use std::ptr;

use crate::Regex;

/// Compile the given string into a regex
///
/// # Safety
/// Ensure that.
/// 1) src is a valid NULL terminated C-String
/// 2) out is a valid pointer to a destination Regex struct
#[no_mangle]
pub unsafe extern "C"
fn regex_compile(src: *const c_char) -> *mut Regex {
    let src = unsafe { CStr::from_ptr(src) };
    let Ok(src) = src.to_str() else { return ptr::null_mut() };

    let Ok(regex) = Regex::compile(src) else { return ptr::null_mut() };
    Box::into_raw(Box::new(regex))
}

/// Test if the given string matches the regex
///
/// # Safety
/// Ensure that.
/// 1) regex is a valid pointer to a Regex struct
/// 2) src is a valid NULL terminated C-String
#[no_mangle]
pub unsafe extern "C"
fn regex_test(regex: *const Regex, src: *const c_char) -> bool {
    let src = unsafe { CStr::from_ptr(src) };
    let Ok(src) = src.to_str() else { return false };

    unsafe { &*regex } .test(src)
}

/// Frees the regex structure
///
/// # Safety
/// Ensure that.
/// 1) regex is a valid pointer to a Regex struct that HAS NOT BEEN FREED before
#[no_mangle]
pub unsafe extern "C"
fn regex_free(regex: *mut Regex) {
    let r = Box::from_raw(regex);
    drop(r)
}
