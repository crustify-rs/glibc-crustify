//! Unit tests of the public interface, decided by Rust assertions.
#![expect(
    unsafe_code,
    reason = "the stream fixture opens, locks and closes C streams through the raw bindings"
)]

#[path = "../fixtures/stream.rs"]
mod stream;

mod fprintf;
