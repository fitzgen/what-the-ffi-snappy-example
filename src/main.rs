use std::env;
use std::error;
use std::fs;
use std::io::{self, Read, Write};
use std::path;
use std::process;

mod bindings;

mod snappy {
    #![allow(unreachable_patterns)]

    use bindings::*;
    use std::error;
    use std::fmt;

    /// Errors that can occur when compressing or uncompressing data with
    /// snappy.
    #[derive(Clone, Copy, Debug)]
    pub enum Error {
        InvalidInput,
        BufferTooSmall,
        UnexpectedLength,
        Unknown,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", error::Error::description(self))
        }
    }

    impl error::Error for Error {
        fn description(&self) -> &'static str {
            match *self {
                Error::InvalidInput => "invalid input",
                Error::BufferTooSmall => "buffer too small",
                Error::UnexpectedLength => "unexpected length",
                Error::Unknown => "unknown",
            }
        }
    }

    /// Compress the given input and return the compressed output.
    pub fn compress(input: &[u8]) -> Result<Vec<u8>, Error> {
        let mut compressed_len = unsafe {
            snappy_max_compressed_length(input.len() as _)
        };

        let mut compressed = vec![0; compressed_len];
        unsafe {
            let result = snappy_compress(input.as_ptr() as *const _,
                                         input.len() as _,
                                         compressed.as_mut_ptr() as *mut _,
                                         &mut compressed_len);
            match result {
                snappy_status::SNAPPY_OK => {
                    if compressed_len >= compressed.len() {
                        Err(Error::UnexpectedLength)
                    } else {
                        compressed.set_len(compressed_len as _);
                        Ok(compressed)
                    }
                }
                snappy_status::SNAPPY_INVALID_INPUT => Err(Error::InvalidInput),
                snappy_status::SNAPPY_BUFFER_TOO_SMALL => Err(Error::BufferTooSmall),
                _ => Err(Error::Unknown),
            }
        }
    }

    /// Decompress the given compressed input and return the uncompressed
    /// output.
    pub fn decompress(compressed: &[u8]) -> Result<Vec<u8>, Error> {
        let mut uncompressed_len = 0;
        unsafe {
            let result = snappy_uncompressed_length(
                compressed.as_ptr() as *const _,
                compressed.len() as _,
                &mut uncompressed_len);
            match result {
                snappy_status::SNAPPY_OK => Ok(()),
                snappy_status::SNAPPY_INVALID_INPUT => Err(Error::InvalidInput),
                snappy_status::SNAPPY_BUFFER_TOO_SMALL => Err(Error::BufferTooSmall),
                _ => Err(Error::Unknown),
            }?
        }

        let mut uncompressed = vec![0; uncompressed_len];
        unsafe {
            let result = snappy_uncompress(
                compressed.as_ptr() as *const _,
                compressed.len() as _,
                uncompressed.as_mut_ptr() as *mut _,
                &mut uncompressed_len);
            match result {
                snappy_status::SNAPPY_OK => {
                    if uncompressed.len() != uncompressed_len {
                        Err(Error::UnexpectedLength)
                    } else {
                        Ok(uncompressed)
                    }
                }
                snappy_status::SNAPPY_INVALID_INPUT => Err(Error::InvalidInput),
                snappy_status::SNAPPY_BUFFER_TOO_SMALL => Err(Error::BufferTooSmall),
                _ => Err(Error::Unknown),
            }
        }
    }
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
fn other_io_error<T, E>(e: E) -> io::Result<T>
    where E: Into<Box<error::Error + Send + Sync>>
{
    Err(io::Error::new(io::ErrorKind::Other, e))
}

/// Compress the existing file at `from` and put the results in `to`.
fn compress(from: &path::Path, to: &path::Path) -> io::Result<()> {
    let mut from_file = fs::File::open(from)?;

    let mut contents = vec![];
    from_file.read_to_end(&mut contents)?;

    let compressed = snappy::compress(&contents)
        .or_else(|e| other_io_error(e))?;

    let mut to_file = fs::File::create(to)?;
    to_file.write_all(&compressed)
}

/// Decompress the compressed file at `from` and put the results in `to`.
fn decompress(from: &path::Path, to: &path::Path) -> io::Result<()> {
    let mut from_file = fs::File::open(from)?;

    let mut contents = vec![];
    from_file.read_to_end(&mut contents)?;

    let uncompressed = snappy::decompress(&contents)
        .or_else(|e| other_io_error(e))?;

    let mut to_file = fs::File::create(to)?;
    to_file.write_all(&uncompressed)
}
