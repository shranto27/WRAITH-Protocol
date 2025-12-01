//! Fuzz target for DHT message parsing
//!
//! Tests that the DHT message parser correctly handles arbitrary input without panicking.

#![no_main]

use libfuzzer_sys::fuzz_target;
use wraith_discovery::dht::DhtMessage;

fuzz_target!(|data: &[u8]| {
    // Fuzz the DHT message parser with arbitrary bytes
    // The parser should never panic, only return Ok or Err
    let _ = DhtMessage::from_bytes(data);

    // Also test decryption path with arbitrary key
    let key = [0u8; 32];
    let _ = DhtMessage::decrypt(data, &key);
});
