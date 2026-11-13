//! Unit tests of the public interface, decided by Rust assertions. The tests hold no unsafe
//! code: the fixtures make every call to the C.
#![deny(unsafe_code)]

#[expect(
    unsafe_code,
    reason = "the stream fixture opens, locks and closes C streams through the raw bindings"
)]
#[path = "../fixtures/stream.rs"]
mod stream;

#[expect(
    unsafe_code,
    reason = "holds a stream's lock, and runs a printf handler on glibc's stream, through the C"
)]
#[path = "../fixtures/stream_unit.rs"]
mod stream_unit;

mod fprintf;
mod iofwrite;
