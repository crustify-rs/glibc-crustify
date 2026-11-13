//! A stream is C's: safe code builds no `File` from a stream's layout either, whatever bytes
//! it holds, since glibc would not know the value as an open stream and a handle to it
//! would reach a stream no lock or list of glibc's covers.

use glibc_bindings::libc::ffi;
use glibc_bindings::libc::libio::genops::File;

fn main() {
    let _stream: File = ffi::_IO_FILE::default().into();
}
