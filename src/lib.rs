// Streambuffer -- Fast asynchronous file reading
// Copyright 2016 Ruud van Asseldonk
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! Streambuffer, a library for fast asynchronous file reading.
//!
//! # Examples
//!
//! Map a file into memory and access it as an array of bytes. This is simple and will generally
//! outperform `Read::read_to_end()`, but it will block upon first access.
//!
//! ```
//! use streambuffer::StreamBuffer;
//! let fstream = StreamBuffer::open("src/lib.rs").unwrap();
//! let buffer = fstream.as_slice();
//! assert_eq!(buffer[3..49], b"Streambuffer -- Fast asynchronous file reading"[..]);
//! ```
//!
//! TODO: More and better (non-blocking) examples.

#![warn(missing_docs)]

use std::io;
use std::fs;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::slice;

extern crate libc;

/// A memory-mapped file.
pub struct StreamBuffer {
  buffer: *const u8,
  length: usize,
}

fn map_file(file: &fs::File) -> io::Result<(*const u8, usize)> {
    let fd = file.as_raw_fd();
    let length = try!(file.metadata()).len();

    if length > usize::max_value() as u64 {
        return Err(io::Error::new(io::ErrorKind::Other, "file is larger than address space"));
    }

    if length == 0 {
        return Err(io::Error::new(io::ErrorKind::Other, "file has size zero"));
    }

    let null = 0 as *mut libc::c_void;
    let result = unsafe {
        libc::mmap(null, length as usize, libc::PROT_READ, libc::MAP_PRIVATE, fd, 0)
    };

    if result == libc::MAP_FAILED {
        Err(io::Error::last_os_error())
    } else {
        Ok((result as *const u8, length as usize))
    }
}

fn unmap_file(buffer: *const u8, length: usize) {
    let result = unsafe {
        libc::munmap(buffer as *mut libc::c_void, length)
    };

    // `munmap` only fails due to incorrect usage, which is a program error, not a runtime failure.
    assert!(result == 0);
}

impl StreamBuffer {
    /// Maps the file at `path` into memory.
    ///
    /// TODO: Document what happens when the file is changed after opening.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<StreamBuffer> {
        // Open the `fs::File` so we get all of std's error handling for free, then use it to
        // extract the file descriptor. The file is closed again when it goes out of scope, but
        // `mmap` only requires the descriptor to be open for the `mmap` call, so this is fine.
        let file = try!(fs::File::open(path));
        let (buffer, length) = try!(map_file(&file));
        let fstream = StreamBuffer {
            buffer: buffer,
            length: length,
        };
        Ok(fstream)
    }

    /// Returns the file contents as a slice.
    ///
    /// Accessing elements of the slice might cause a page fault,
    /// blocking until the data has been read from disk.
    pub fn as_slice(&self) -> &[u8] {
       unsafe { slice::from_raw_parts(self.buffer, self.length) }
    }
}

impl Drop for StreamBuffer {
    fn drop(&mut self) {
        unmap_file(self.buffer, self.length);
    }
}

#[test]
fn open_file() {
    let fstream = StreamBuffer::open("src/lib.rs");
    assert!(fstream.is_ok());
}
