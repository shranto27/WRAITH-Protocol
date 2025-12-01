//! Fuzz target for padding operations
//!
//! Tests that the padding engine correctly handles arbitrary input sizes.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use wraith_obfuscation::padding::{PaddingEngine, PaddingMode};

#[derive(Debug, Arbitrary)]
struct PaddingInput {
    mode: u8,
    plaintext_len: usize,
    data: Vec<u8>,
}

fuzz_target!(|input: PaddingInput| {
    // Select padding mode based on arbitrary byte
    let mode = match input.mode % 5 {
        0 => PaddingMode::None,
        1 => PaddingMode::PowerOfTwo,
        2 => PaddingMode::SizeClasses,
        3 => PaddingMode::ConstantRate,
        _ => PaddingMode::Statistical,
    };

    let mut engine = PaddingEngine::new(mode);

    // Fuzz padded_size - should never panic
    let target_size = engine.padded_size(input.plaintext_len);

    // Verify invariants
    if mode != PaddingMode::None {
        assert!(
            target_size >= input.plaintext_len,
            "Padded size should be >= plaintext len"
        );
    }

    // Fuzz pad operation
    let mut buffer = input.data.clone();
    engine.pad(&mut buffer, target_size);

    // Fuzz unpad operation
    let original_len = input.plaintext_len.min(buffer.len());
    let _ = engine.unpad(&buffer, original_len);

    // Fuzz overhead calculation
    let _ = engine.overhead(input.plaintext_len);
});
