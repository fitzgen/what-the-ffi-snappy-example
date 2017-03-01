extern crate libc;
use libc::{c_char, c_int, size_t};
use std::env;
use std::error;
use std::fs;
use std::io::{self, Read, Write};
use std::path;
use std::process;

extern {
    fn snappy_compress(input: *const c_char,
                       input_length: size_t,
                       compressed: *mut c_char,
                       compressed_length: *mut size_t)
                       -> c_int;

    fn snappy_uncompress(compressed: *const c_char,
                         compressed_length: size_t,
                         uncompressed: *mut c_char,
                         uncompressed_length: *mut size_t)
                         -> c_int;

    fn snappy_max_compressed_length(source_length: size_t) -> size_t;

    fn snappy_uncompressed_length(compressed: *const c_char,
                                  compressed_length: size_t,
                                  result: *mut size_t)
                                  -> c_int;
}

fn main() {
    let result = match Command::parse(env::args().skip(1)) {
        Some(Command::Compress { from, to }) => compress(&from, &to),
        Some(Command::Decompress { from, to }) => {
            decompress(&from, &to)
        }
        None => other_io_error("invalid args"),
    };

    if let Err(e) = result {
        println!("error: {}", e);
        process::exit(1);
    }
}

/// This program has two commands: compress or decompress from one file to
/// another.
enum Command {
    Compress {
        from: path::PathBuf,
        to: path::PathBuf,
    },
    Decompress {
        from: path::PathBuf,
        to: path::PathBuf,
    },
}

impl Command {
    /// Parse a `Command` from the given command line arguments.
    fn parse<'a, I>(args: I) -> Option<Self>
        where I: IntoIterator<Item = String>
    {
        let mut args = args.into_iter().fuse();

        let cmd = args.next();
        let from = args.next();
        let to = args.next();
        let none = args.next();

        match (cmd, from, to, none) {
            (Some(ref cmd), Some(ref from), Some(ref to), None)
                if cmd == "compress" =>
            {
                Some(Command::Compress {
                    from: path::PathBuf::from(from),
                    to: path::PathBuf::from(to),
                })
            }
            (Some(ref cmd), Some(ref from), Some(ref to), None)
                if cmd == "decompress" =>
            {
                Some(Command::Decompress {
                    from: path::PathBuf::from(from),
                    to: path::PathBuf::from(to),
                })
            }
            _ => None,
        }
    }
}

/// Tiny helper to create a `std::io::Error` error result of kind
/// `std::io::ErrorKind::Other` from the given argument.
fn other_io_error<E>(e: E) -> io::Result<()>
    where E: Into<Box<error::Error + Send + Sync>>
{
    Err(io::Error::new(io::ErrorKind::Other, e))
}

/// Compress the existing file at `from` and put the results in `to`.
fn compress(from: &path::Path, to: &path::Path) -> io::Result<()> {
    let mut from_file = fs::File::open(from)?;

    let mut contents = vec![];
    from_file.read_to_end(&mut contents)?;

    let mut compressed_len = unsafe {
        snappy_max_compressed_length(contents.len() as _)
    };

    let mut compressed = vec![0; compressed_len];
    unsafe {
        let result = snappy_compress(contents.as_ptr() as *const _,
                                     contents.len() as _,
                                     compressed.as_mut_ptr() as *mut _,
                                     &mut compressed_len);
        if result != 0 {
            return other_io_error(format!("snappy error result: {}", result));
        }
        if compressed_len >= compressed.len() {
            return other_io_error("bad compressed length from snappy");
        }
        compressed.set_len(compressed_len as _);
    }

    let mut to_file = fs::File::create(to)?;
    to_file.write_all(&compressed)
}

/// Decompress the compressed file at `from` and put the results in `to`.
fn decompress(from: &path::Path, to: &path::Path) -> io::Result<()> {
    let mut from_file = fs::File::open(from)?;

    let mut contents = vec![];
    from_file.read_to_end(&mut contents)?;

    let mut uncompressed_len = 0;
    unsafe {
        let result = snappy_uncompressed_length(
            contents.as_ptr() as *const _,
            contents.len() as _,
            &mut uncompressed_len as *mut _);
        if result != 0 {
            return other_io_error(format!("snappy error result: {}", result));
        }
    }

    let mut uncompressed = vec![0; uncompressed_len];
    unsafe {
        let result = snappy_uncompress(
            contents.as_ptr() as *const _,
            contents.len() as _,
            uncompressed.as_mut_ptr() as *mut _,
            &mut uncompressed_len as *mut _);
        if result != 0 {
            return other_io_error(format!("snappy error result: {}", result));
        }
        if uncompressed.len() != uncompressed_len {
            return other_io_error("bad uncompressed length from snappy");
        }
    }

    let mut to_file = fs::File::create(to)?;
    to_file.write_all(&uncompressed)
}
