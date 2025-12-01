//! Fuzz target for frame parsing
//!
//! Tests that the frame parser correctly handles arbitrary input without panicking.

#![no_main]

use libfuzzer_sys::fuzz_target;
use wraith_core::Frame;

fuzz_target!(|data: &[u8]| {
    // Fuzz the frame parser with arbitrary bytes
    // The parser should never panic, only return Ok or Err
    let _ = Frame::parse(data);

    // Also test scalar parsing path
    let _ = Frame::parse_scalar(data);
});
