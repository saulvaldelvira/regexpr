//! C bindings

use std::ffi::{c_char, c_ulong, CStr};
use std::ptr;
use crate::{Regex, RegexMatcher};

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

/// Returns an iterator over all the matches found in the source string
///
/// # Safety
/// Ensure that.
/// 1) regex is a valid pointer to a Regex struct
/// 2) src is a valid NULL terminated C-String
/// 3) You call `regex_matcher_free` on the returned pointer after you're done
#[no_mangle]
pub unsafe extern "C"
fn regex_find_matches(regex: *const Regex, src: *const c_char) -> *mut RegexMatcher<'static> {
    let src = unsafe { CStr::from_ptr(src) };
    let Ok(src) = src.to_str() else { return ptr::null_mut() };

    let matcher = unsafe { &*regex } .find_matches(src);
    let matcher = Box::new(matcher);
    Box::into_raw(matcher)
}

#[repr(C)]
pub struct Span {
    offset: c_ulong,
    len: c_ulong,
}

/// Gets the next match from the matcher.
/// Span is filled with the offset and len of the match
///
/// Returns true if there's another match, false if the iterator is over
///
/// # Safety
/// Ensure that.
/// 1) matcher is a valid pointer to a `RegexMatcher`
/// 2) span is a valid pointer to a Span struct
#[no_mangle]
pub unsafe extern "C"
fn regex_matcher_next(matcher: *mut RegexMatcher<'_>, span: *mut Span) -> bool {
    match unsafe { &mut *matcher } .next() {
        Some(m) => {
            *span = Span {
                offset: m.span().0 as c_ulong,
                len: m.slice().len() as c_ulong,
            };
            true
        },
        None => false
    }
}

/// Frees the regex matcher
///
/// # Safety
/// Ensure that matcher is a valid pointer
#[no_mangle]
pub unsafe extern "C"
fn regex_matcher_free(matcher: *mut RegexMatcher<'_>) {
    drop( Box::from_raw(matcher) );
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
    drop(r);
}
