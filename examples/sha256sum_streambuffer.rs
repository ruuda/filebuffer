// Streambuffer -- Fast asynchronous file reading
// Copyright 2016 Ruud van Asseldonk
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

// This example implements the `sha256sum` program in Rust using the Streambuffer library. Compare
// with `sha256sum_streambuffer` which uses the IO primitives in the standard library.
//
// Printing hashes in the order the files were specified is left as an excercise for the reader.

use std::collections::VecDeque;
use std::cmp;
use std::env;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use streambuffer::StreamBuffer;

extern crate crypto;
extern crate streambuffer;

struct State {
    fname: String,
    fstream: StreamBuffer,
    offset: usize,
    hasher: Sha256
}

/// Continues hashing until that would block. Returns the state if hashing is not done, plus a
/// boolean indicating whether progress was made.
fn hash(mut state: State) -> Option<(State, bool)> {
    let mut made_progress = false;
    loop {
        let bytes_left = state.fstream.len() - state.offset;
        let chunk_len = cmp::min(bytes_left, state.fstream.chunk_len_hint());

        // Keep on hashing slices until a slice is not resident.
        match state.fstream.try_slice(state.offset, chunk_len) {
            Some(slice) => {
                state.hasher.input(slice);
                state.offset += chunk_len;
                made_progress = true;

                if state.offset == state.fstream.len() {
                    // Match the output format of `sha256sum`, which has two spaces between
                    // the hash and name.
                    println!("{}  {}", state.hasher.result_str(), state.fname);
                    return None;
                }
            }
            None => break
        }
    }

    // Not done yet, try again later when more data is resident.
    Some((state, made_progress))
}

fn open_new(files: &mut VecDeque<String>) -> State {
    let fname = files.pop_front().unwrap();
    let fstream = StreamBuffer::open(&fname).expect("failed to open file");
    State {
        fname: fname,
        fstream: fstream,
        offset: 0,
        hasher: Sha256::new(),
    }
}

fn main() {
    let mut files: VecDeque<String> = env::args().skip(1).collect();
    let mut queue = VecDeque::new();

    loop {
        match queue.pop_front() {
            Some(state) => match hash(state) {
                Some((state, made_progress)) => {
                    if !made_progress && !files.is_empty() {
                        // If no progress was made, we are going through the queue too quickly.
                        // Open a new file so we have more to do. Push to the front, so that we
                        // immediately kick off the prefetch (or continue to make progress if the
                        // file happened to be resident already).
                        queue.push_front(open_new(&mut files));
                    }
                    queue.push_back(state);
                }
                None => {
                    // The file is done. It would be possible to open a new file here to keep the
                    // queue full, but it is probably better not to do that; new files will be
                    // opened if no progress is made, so this keeps the open files to a minimum.
                }
            },
            None => {
                if files.is_empty() {
                    // All tasks are done, and there are no new files. End of the program.
                    return;
                } else {
                    queue.push_front(open_new(&mut files));
                }
            }
        }
    }
}

