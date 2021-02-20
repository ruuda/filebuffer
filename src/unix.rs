// Filebuffer -- Fast and simple file reading
// Copyright 2016 Ruud van Asseldonk
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! This mod contains the platform-specific implementations of functions based on the libc crate
//! that is available on Unix-ish platforms.

use std::fs;
use std::io;
use std::mem;
use std::os::unix::io::AsRawFd;
use std::ptr;

extern crate libc;

#[derive(Debug)]
pub struct PlatformData;

pub fn map_file(file: fs::File) -> io::Result<(*const u8, usize, PlatformData)> {
    let fd = file.as_raw_fd();
    let length = (file.metadata()?).len();

    if length > usize::max_value() as u64 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "file is larger than address space",
        ));
    }

    // Don't try to map anything if the file is empty.
    if length == 0 {
        return Ok((ptr::null(), 0, PlatformData));
    }

    let result = unsafe {
        libc::mmap(
            ptr::null_mut(),
            length as usize,
            libc::PROT_READ,
            libc::MAP_PRIVATE,
            fd,
            0,
        )
    };

    if result == libc::MAP_FAILED {
        Err(io::Error::last_os_error())
    } else {
        Ok((result as *const u8, length as usize, PlatformData))
    }
}

pub fn unmap_file(buffer: *const u8, length: usize) {
    let result = unsafe { libc::munmap(buffer as *mut libc::c_void, length) };

    // `munmap` only fails due to incorrect usage, which is a program error, not a runtime failure.
    assert!(result == 0);
}

/// Writes whether the pages in the range starting at `buffer` with a length of `length` bytes
/// are resident in physical memory into `residency`. The size of `residency` must be at least
/// `length / page_size`. Both `buffer` and `length` must be a multiple of the page size.
pub fn get_resident(buffer: *const u8, length: usize, residency: &mut [bool]) {
    use std::thread;

    let result = unsafe {
        // Note: the libc on BSD descendants uses a signed char for residency_char while
        // glibc uses an unsigned one, which is why we use an type-inferred cast here.
        let residency_char = residency.as_mut_ptr() as *mut _;
        assert_eq!(1, mem::size_of_val(&*residency_char));
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

/// Requests the kernel to make the specified range of bytes resident in physical memory. `buffer`
/// must be page-aligned.
pub fn prefetch(buffer: *const u8, length: usize) {
    let result = unsafe { libc::madvise(buffer as *mut libc::c_void, length, libc::MADV_WILLNEED) };

    // Any returned error code indicates a programming error, not a runtime error.
    assert_eq!(0, result);
}

pub fn get_page_size() -> usize {
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };

    // Assert that the page size is a power of two, which is assumed when the page size is used.
    assert!(page_size != 0);
    assert_eq!(0, page_size & (page_size - 1));

    page_size
}
