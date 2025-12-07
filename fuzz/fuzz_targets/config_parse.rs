//! Fuzz test for configuration file parsing
//!
//! Tests that arbitrary TOML input doesn't cause panics or crashes when
//! parsed as WRAITH configuration.

#![no_main]

use libfuzzer_sys::fuzz_target;
use wraith_core::node::NodeConfig;

fuzz_target!(|data: &[u8]| {
    // Try parsing as UTF-8 TOML
    if let Ok(s) = std::str::from_utf8(data) {
        // Attempt to parse as TOML
        let _: Result<toml::Value, _> = toml::from_str(s);

        // Attempt to deserialize as NodeConfig
        // This will fail for invalid configs, but shouldn't panic
        let _: Result<NodeConfig, _> = toml::from_str(s);
    }
});
