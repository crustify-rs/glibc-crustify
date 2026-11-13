//! Equivalence tests of the public interface against the C it wraps.
#![expect(
    unsafe_code,
    reason = "calls the C through the raw bindings, opens its streams through the fixture, and reads and writes the C globals the wrappers stand in for"
)]

#[path = "../fixtures/stream.rs"]
mod stream;

mod fprintf;
mod stdio;
