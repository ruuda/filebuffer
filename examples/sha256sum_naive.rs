// Filebuffer -- Fast and simple file reading
// Copyright 2016 Ruud van Asseldonk
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

// This example implements the `sha256sum` program in Rust using the IO primitives in the
// standard library. Compare with `sha256sum_filebuffer` which uses the Filebuffer library.

use std::env;
use std::fs;
use std::io;
use std::io::BufRead;
use crypto::digest::Digest;
use crypto::sha2::Sha256;

extern crate crypto;

fn main() {
    for fname in env::args().skip(1) {
        let file = fs::File::open(&fname).expect("failed to open file");
        let mut reader = io::BufReader::new(file);
        let mut hasher = Sha256::new();

        loop {
            let consumed_len = {
                let buffer = reader.fill_buf().expect("failed to read from file");
                if buffer.len() == 0 {
                    // End of file.
                    break;
                }
                hasher.input(buffer);
                buffer.len()
            };
            reader.consume(consumed_len);
        }

        // Match the output format of `sha256sum`, which has two spaces between the hash and name.
        println!("{}  {}", hasher.result_str(), fname);
    }
}
