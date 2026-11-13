//! UB tests against the unsafe public API, called within its `# Safety` contracts.
#![expect(
    unsafe_code,
    reason = "calls the public unsafe API within its contracts"
)]

mod stdio;
