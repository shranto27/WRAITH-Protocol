//! Fuzz test for CLI argument parsing
//!
//! Tests that arbitrary CLI arguments don't cause panics or crashes.

#![no_main]

use libfuzzer_sys::fuzz_target;
use wraith_core::node::NodeConfig;

fuzz_target!(|data: &[u8]| {
    // Convert fuzzer input to string arguments
    if let Ok(s) = std::str::from_utf8(data) {
        // Split into arguments on whitespace
        let args: Vec<&str> = s.split_whitespace().collect();

        if args.is_empty() {
            return;
        }

        // Try parsing as CLI args (simulating CLI commands)
        // Test various CLI command patterns that might be parsed

        // Test 1: Try parsing as send command arguments
        if args.len() >= 2 && args[0] == "send" {
            let _ = parse_send_args(&args[1..]);
        }

        // Test 2: Try parsing as receive command arguments
        if args.len() >= 1 && args[0] == "receive" {
            let _ = parse_receive_args(&args[1..]);
        }

        // Test 3: Try parsing as daemon command arguments
        if args.len() >= 1 && args[0] == "daemon" {
            let _ = parse_daemon_args(&args[1..]);
        }

        // Test 4: Try parsing configuration values
        for arg in &args {
            // Test parsing as port number
            let _ = arg.parse::<u16>();

            // Test parsing as socket address
            let _ = arg.parse::<std::net::SocketAddr>();

            // Test parsing as boolean
            let _ = arg.parse::<bool>();
        }
    }
});

// Simulated CLI parsing functions (these represent the actual CLI parsing logic)
fn parse_send_args(args: &[&str]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Missing file path".to_string());
    }

    let _file_path = args[0];

    // Parse optional peer ID
    if args.len() > 1 {
        let _peer_id = args[1];
        // Try parsing as hex peer ID (64 hex chars = 32 bytes)
        if args[1].len() == 64 {
            let _ = hex::decode(args[1]);
        }
    }

    Ok(())
}

fn parse_receive_args(args: &[&str]) -> Result<(), String> {
    // Parse optional output directory
    if !args.is_empty() {
        let _output_dir = args[0];
    }

    Ok(())
}

fn parse_daemon_args(args: &[&str]) -> Result<(), String> {
    // Parse optional config file
    if !args.is_empty() {
        let _config_file = args[0];
    }

    Ok(())
}
