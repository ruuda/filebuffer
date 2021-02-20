// Filebuffer -- Fast and simple file reading
// Copyright 2016 Ruud van Asseldonk
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! Filebuffer, a library for fast and simple file reading.
//!
//! # Examples
//!
//! Map a file into memory and access it as a slice of bytes. This is simple and will generally
//! outperform `Read::read_to_end()`.
//!
//! ```
//! use filebuffer::FileBuffer;
//! let fbuffer = FileBuffer::open("src/lib.rs").unwrap();
//! assert_eq!(&fbuffer[3..45], &b"Filebuffer -- Fast and simple file reading"[..]);
//! ```

#![warn(missing_docs)]

use std::cmp;
use std::fs;
use std::io;
use std::ops::Deref;
use std::path::Path;
use std::ptr;
use std::slice;

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

#[cfg(unix)]
use unix::{get_page_size, map_file, prefetch, unmap_file, PlatformData};

#[cfg(all(unix))]
use unix::get_resident;

#[cfg(windows)]
use windows::{get_page_size, get_resident, map_file, prefetch, unmap_file, PlatformData};

/// A memory-mapped file.
///
/// # Safety
///
/// **On Unix-ish platforms, external modifications to the file made after the file buffer was
/// opened can show up in this file buffer.** In particular, if a file is truncated after opening,
/// accessing the removed part causes undefined behavior. On Windows it is possible to prevent this
/// by opening the file in exclusive mode, but that functionality is not available in stable Rust
/// currently. (Filebuffer will be updated after stabilization.)
///
/// It is recommended to ensure that other applications do not write to the file when it is mapped,
/// possibly by marking the file read-only. (Though even this is no guarantee.)
#[derive(Debug)]
pub struct FileBuffer {
    page_size: usize,
    buffer: *const u8,
    length: usize,

    #[allow(dead_code)] // This field is not dead, it might have an effectful destructor.
    platform_data: PlatformData,
}

/// Rounds `size` up to the nearest multiple of `power_of_two`.
fn round_up_to(size: usize, power_of_two: usize) -> usize {
    (size + (power_of_two - 1)) & !(power_of_two - 1)
}

#[test]
fn verify_round_up_to() {
    assert_eq!(1024, round_up_to(23, 1024));
    assert_eq!(1024, round_up_to(1024, 1024));
    assert_eq!(2048, round_up_to(1025, 1024));
}

/// Rounds `size` down to the nearest multiple of `power_of_two`.
fn round_down_to(size: usize, power_of_two: usize) -> usize {
    size & !(power_of_two - 1)
}

#[test]
fn verify_round_down_to() {
    assert_eq!(0, round_down_to(23, 1024));
    assert_eq!(1024, round_down_to(1024, 1024));
    assert_eq!(1024, round_down_to(1025, 1024));
}

impl FileBuffer {
    /// Maps the file at `path` into memory.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<FileBuffer> {
        // Open the `fs::File` so we get all of std's error handling for free, then use it to
        // extract the file descriptor. The file is closed again when `map_file` returns on
        // Unix-ish platforms, but `mmap` only requires the descriptor to be open for the `mmap`
        // call, so this is fine. On Windows, the file must be kept open for the lifetime of the
        // mapping, so `map_file` moves the file into the platform data.
        let mut open_opts = fs::OpenOptions::new();
        open_opts.read(true);

        // TODO: On Windows, set `share_mode()` to read-only. This requires the
        // `open_options_ext` feature that is currently unstable, but it is
        // required to ensure that a different process does not suddenly modify
        // the contents of the file. See also Rust issue 27720.

        let file = open_opts.open(path)?;
        let (buffer, length, platform_data) = map_file(file)?;
        let fbuffer = FileBuffer {
            page_size: get_page_size(),
            buffer: buffer,
            length: length,
            platform_data: platform_data,
        };
        Ok(fbuffer)
    }

    /// Returns the number of bytes resident in physical memory, starting from `offset`.
    ///
    /// The slice `[offset..offset + resident_len]` can be accessed without causing page faults or
    /// disk access. Note that this is only a snapshot, and the kernel might decide to evict pages
    /// or make them resident at any time.
    ///
    /// The returned resident length is at most `length`.
    ///
    /// # Panics
    ///
    /// Panics if the specified range lies outside of the buffer.
    ///
    /// # Remarks
    ///
    /// Windows does not expose a mechanism to query which pages are resident in physical
    /// memory. Therefore this function optimistically claims that the entire range is resident
    /// on Windows.
    pub fn resident_len(&self, offset: usize, length: usize) -> usize {
        // The specified offset and length must lie within the buffer.
        assert!(offset + length <= self.length);

        // This is a no-op for empty files.
        if self.buffer == ptr::null() {
            return 0;
        }

        let aligned_offset = round_down_to(offset, self.page_size);
        let aligned_length = round_up_to(length + (offset - aligned_offset), self.page_size);
        let num_pages = aligned_length / self.page_size;

        // There is a tradeoff here: to store residency information, we need an array of booleans.
        // The requested range can potentially be very large and it is only known at runtime. We
        // could allocate a vector here, but that requires a heap allocation just to get residency
        // information (which might in turn cause a page fault). Instead, check at most 32 pages at
        // once. This means more syscalls for large ranges, but it saves us the heap allocation,
        // and for ranges up to 32 pages (128 KiB typically) there is only one syscall.
        let mut residency = [false; 32];
        let mut pages_checked = 0;
        let mut pages_resident = 0;

        while pages_checked < num_pages {
            let pages_to_check = cmp::min(32, num_pages - pages_checked);
            let check_offset = (aligned_offset + pages_checked * self.page_size) as isize;
            let check_buffer = unsafe { self.buffer.offset(check_offset) };
            let check_length = pages_to_check * self.page_size;
            get_resident(check_buffer, check_length, &mut residency);

            // Count the number of resident pages.
            match residency[..pages_to_check]
                .iter()
                .position(|resident| !resident)
            {
                Some(non_resident) => {
                    // The index of the non-resident page is the number of resident pages.
                    pages_resident += non_resident;
                    break;
                }
                None => {
                    pages_resident += pages_to_check;
                    pages_checked += pages_to_check;
                }
            }
        }

        let resident_length = pages_resident * self.page_size + aligned_offset - offset;

        // Never return more than the requested length. The resident length might be larger than
        // the length of the buffer, because it is rounded up to the page size.
        cmp::min(length, resident_length)
    }

    /// Returns the system page size.
    ///
    /// When the kernel makes the file resident in physical memory, it does so with page
    /// granularity. (In practice this happens in larger chunks, but still in multiples of
    /// the page size.) Therefore, when processing the file in chunks, this is a good chunk
    /// length.
    pub fn chunk_len_hint(&self) -> usize {
        self.page_size
    }

    /// Advises the kernel to make a slice of the file resident in physical memory.
    ///
    /// This method does not block, meaning that when the function returns, the slice is not
    /// necessarily resident. After this function returns, the kernel may read the requested slice
    /// from disk and make it resident. Note that this is only an advice, the kernel need not honor
    /// it.
    ///
    /// To check whether the slice is resident at a later time, use `resident_len()`.
    ///
    /// # Panics
    ///
    /// Panics if the specified range lies outside of the buffer.
    pub fn prefetch(&self, offset: usize, length: usize) {
        // TODO: This function should use `collections::range::RangeArgument` once stabilized.
        // The specified offset and length must lie within the buffer.
        assert!(offset + length <= self.length);

        // This is a no-op for empty files.
        if self.buffer == ptr::null() {
            return;
        }

        let aligned_offset = round_down_to(offset, self.page_size);
        let aligned_length = round_up_to(length + (offset - aligned_offset), self.page_size);

        let buffer = unsafe { self.buffer.offset(aligned_offset as isize) };
        prefetch(buffer, aligned_length);
    }

    /// Leaks the file buffer as a byte slice.
    ///
    /// This prevents the buffer from being unmapped, keeping the file mapped until the program
    /// ends. This is not as bad as it sounds, because the kernel is free to evict pages from
    /// physical memory in case of memory pressure. Because the file is mapped read-only, it can
    /// always be read from disk again.
    ///
    /// If the file buffer is going to be open for the entire duration of the program anyway, this
    /// method can avoid some lifetime issues. Still, it is good practice to close the file buffer
    /// if possible. This method should be a last resort.
    pub fn leak(mut self) -> &'static [u8] {
        let buffer = unsafe { slice::from_raw_parts(self.buffer, self.length) };

        // Prevent `drop()` from freeing the buffer.
        self.buffer = ptr::null();
        self.length = 0;

        buffer
    }
}

// There is no possibility of data races when passing `&FileBuffer` across threads,
// because the buffer is read-only. `&FileBuffer` has no interior mutability.
unsafe impl Sync for FileBuffer {}

// It is safe to move a `FileBuffer` into a different thread.
unsafe impl Send for FileBuffer {}

impl Drop for FileBuffer {
    fn drop(&mut self) {
        if self.buffer != ptr::null() {
            unmap_file(self.buffer, self.length);
        }
    }
}

impl Deref for FileBuffer {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.buffer, self.length) }
    }
}

impl AsRef<[u8]> for FileBuffer {
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}

#[test]
fn open_file() {
    let fbuffer = FileBuffer::open("src/lib.rs");
    assert!(fbuffer.is_ok());
}

#[test]
fn make_resident() {
    let fbuffer = FileBuffer::open("src/lib.rs").unwrap();

    // Touch the first page to make it resident.
    assert_eq!(&fbuffer[3..13], &b"Filebuffer"[..]);

    // Now at least that part should be resident.
    assert_eq!(fbuffer.resident_len(3, 10), 10);
}

#[test]
fn prefetch_is_not_harmful() {
    let fbuffer = FileBuffer::open("src/lib.rs").unwrap();

    // It is impossible to test that this actually works without root access to instruct the kernel
    // to drop its caches, but at least we can verify that calling `prefetch` is not harmful.
    fbuffer.prefetch(0, fbuffer.len());

    // Reading from the file should still work as normal.
    assert_eq!(&fbuffer[3..13], &b"Filebuffer"[..]);
}

#[test]
fn drop_after_leak() {
    let mut bytes = &[0u8][..];
    assert_eq!(bytes[0], 0);
    {
        let fbuffer = FileBuffer::open("src/lib.rs").unwrap();
        bytes = fbuffer.leak();
    }
    assert_eq!(&bytes[3..13], &b"Filebuffer"[..]);
}

#[test]
fn fbuffer_can_be_moved_into_thread() {
    use std::thread;

    let fbuffer = FileBuffer::open("src/lib.rs").unwrap();
    thread::spawn(move || {
        assert_eq!(&fbuffer[3..13], &b"Filebuffer"[..]);
    });
}

#[test]
fn fbuffer_can_be_shared_among_threads() {
    use std::sync;
    use std::thread;

    let fbuffer = FileBuffer::open("src/lib.rs").unwrap();
    let buffer1 = sync::Arc::new(fbuffer);
    let buffer2 = buffer1.clone();
    thread::spawn(move || {
        assert_eq!(&buffer2[3..13], &b"Filebuffer"[..]);
    });
    assert_eq!(&buffer1[17..45], &b"Fast and simple file reading"[..]);
}

#[test]
fn open_empty_file_is_fine() {
    FileBuffer::open("src/empty_file_for_testing.rs").unwrap();
}

#[test]
fn empty_file_prefetch_is_fine() {
    let fbuffer = FileBuffer::open("src/empty_file_for_testing.rs").unwrap();
    fbuffer.prefetch(0, 0);
}

#[test]
fn empty_file_deref_is_fine() {
    let fbuffer = FileBuffer::open("src/empty_file_for_testing.rs").unwrap();
    assert_eq!(fbuffer.iter().any(|_| true), false);
}

#[test]
fn empty_file_has_zero_resident_len() {
    let fbuffer = FileBuffer::open("src/empty_file_for_testing.rs").unwrap();
    assert_eq!(fbuffer.resident_len(0, 0), 0);
}

#[test]
fn page_size_at_least_4096() {
    // There is no reason why the page size cannot be smaller, it is just that in practice there
    // is no platform with a smaller page size, so this tests that `get_page_size()` returns
    // a plausible value.
    assert!(get_page_size() >= 4096);
}
