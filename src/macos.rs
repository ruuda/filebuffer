// Filebuffer -- Fast and simple file reading
// Copyright 2016 Ruud van Asseldonk
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! This mod contains the platform-specific implementations of functions based on the libc crate
//! that is available on Mac OS X platforms.

use std::fs;
use std::io;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::thread;

extern crate libc;

/// Writes whether the pages in the range starting at `buffer` with a length of `length` bytes
/// are resident in physical memory into `residency`. The size of `residency` must be at least
/// `length / page_size`. Both `buffer` and `length` must be a multiple of the page size.
pub fn get_resident(buffer: *const u8, length: usize, residency: &mut [bool]) {
    let result = unsafe {
        // Note that the type here is a signed char, unlike the libc on other
        // platforms. The regular version of this function is in unix.rs.
        let residency_char = residency.as_mut_ptr() as *mut libc::c_char;
        libc::mincore(buffer as *mut libc::c_void, length, residency_char)
    };

    // Any error code except EAGAIN indicates a programming error.
    assert!(result == libc::EAGAIN || result == 0);

    // In the rare occasion that the kernel is busy, yield so we don't spam the kernel with
    // `mincore` calls, then try again.
    if result == libc::EAGAIN {
        thread::yield_now();
        get_resident(buffer, length, residency)
    }
}
