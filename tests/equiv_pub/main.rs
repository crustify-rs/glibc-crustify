//! Equivalence tests of the public interface against the C it wraps.
#![expect(
    unsafe_code,
    reason = "calls the C through the raw bindings, and opens its streams through the fixture"
)]

#[path = "../fixtures/stream.rs"]
mod stream;

mod fprintf;
