//! A stream is C's: safe code builds no `File` value, which glibc would not know as an open
//! stream, nor a zeroed one, whose NULL lock a stdio routine would take.

use ffibox::CZeroable;
use glibc_bindings::libc::libio::genops::File;

fn main() {
    let _stream = File::zeroed();
}
