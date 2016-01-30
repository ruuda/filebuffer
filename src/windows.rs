// Filebuffer -- Fast and simple file reading
// Copyright 2016 Ruud van Asseldonk
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! This mod contains the platform-specific implementations for Windows based on the winapi crate.

use std::fs;
use std::io;
use std::os::windows::io::AsRawHandle;
use std::ptr;

extern crate winapi;

pub fn map_file(file: &fs::File) -> io::Result<(*const u8, usize)> {
    let handle = file.as_raw_handle();
    let length = try!(file.metadata()).len();

    if length > usize::max_value() as u64 {
        return Err(io::Error::new(io::ErrorKind::Other, "file is larger than address space"));
    }

    // Don't try to map anything if the file is empty.
    if length == 0 {
        return Ok((ptr::null(), 0));
    }

    // TODO: Implement this.
    Err(io::Error::new(io::ErrorKind::Other, "not implemented"))
}

pub fn unmap_file(buffer: *const u8, length: usize) {
    // TODO: Implement this.
}

/// See also `unix::get_resident`.
pub fn get_resident(buffer: *const u8, length: usize, residency: &mut [bool]) {
    // TODO: Implement this.
}

/// See also `unix::prefetch`.
pub fn prefetch(buffer: *const u8, length: usize) {
    // TODO: Implement this.
}

pub fn get_page_size() -> usize {
    // TODO: Implement this properly.
    4096
}
