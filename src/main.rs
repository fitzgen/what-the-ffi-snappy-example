extern crate libc;
use libc::{c_char, c_int, size_t};

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
    println!("Hello, world!");
}
