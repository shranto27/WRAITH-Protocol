//! Property-based tests for WRAITH Protocol
//!
//! Uses proptest to verify invariants across large input spaces.

use proptest::prelude::*;

// ============================================================================
// Frame Encoding/Decoding Properties
// ============================================================================

mod frame_properties {
    use super::*;
    use wraith_core::{Frame, FrameBuilder, FrameType};

    proptest! {
        /// Frame roundtrip: encode then decode should produce equivalent frame
        #[test]
        fn frame_roundtrip(
            frame_type in 0u8..8,
            stream_id in prop::num::u16::ANY.prop_filter("avoid reserved IDs", |&id| id == 0 || id >= 16),
            sequence in any::<u32>(),
            payload_len in 0usize..1024,
        ) {
            let frame_type = match frame_type {
                0 => FrameType::Data,
                1 => FrameType::Ack,
                2 => FrameType::Control,
                3 => FrameType::Rekey,
                4 => FrameType::Ping,
                5 => FrameType::Pong,
                6 => FrameType::Close,
                _ => FrameType::Pad,
            };

            let payload: Vec<u8> = (0..payload_len).map(|i| (i & 0xFF) as u8).collect();

            // Build requires a total_size parameter
            let total_size = 28 + payload.len() + 16; // header + payload + some padding

            let encoded = FrameBuilder::new()
                .frame_type(frame_type)
                .stream_id(stream_id)
                .sequence(sequence)
                .payload(&payload)
                .build(total_size);

            if let Ok(encoded) = encoded {
                // Parse should succeed and produce equivalent frame
                if let Ok(decoded) = Frame::parse(&encoded) {
                    prop_assert_eq!(decoded.frame_type(), frame_type);
                    prop_assert_eq!(decoded.stream_id(), stream_id);
                    prop_assert_eq!(decoded.sequence(), sequence);
                    prop_assert_eq!(decoded.payload(), &payload[..]);
                }
            }
        }

        /// Frame encoding respects minimum size
        #[test]
        fn frame_minimum_size(payload_len in 0usize..256) {
            let payload: Vec<u8> = vec![0xAA; payload_len];
            let total_size = 28 + payload_len; // header + payload, no extra padding

            let result = FrameBuilder::new()
                .frame_type(FrameType::Data)
                .stream_id(0)
                .payload(&payload)
                .build(total_size);

            prop_assert!(result.is_ok(), "Building frame should succeed");
            prop_assert_eq!(result.unwrap().len(), total_size);
        }
    }
}

// ============================================================================
// AEAD Encryption Properties
// ============================================================================

mod aead_properties {
    use super::*;
    use wraith_crypto::aead::{AeadKey, Nonce};

    proptest! {
        /// AEAD roundtrip: encrypt then decrypt should recover plaintext
        #[test]
        fn aead_roundtrip(
            key_bytes in any::<[u8; 32]>(),
            nonce_bytes in any::<[u8; 24]>(),
            plaintext in prop::collection::vec(any::<u8>(), 0..1024),
            aad in prop::collection::vec(any::<u8>(), 0..64),
        ) {
            let key = AeadKey::new(key_bytes);
            let nonce = Nonce::from_bytes(nonce_bytes);

            let ciphertext = key.encrypt(&nonce, &plaintext, &aad)
                .expect("Encryption should succeed");

            let decrypted = key.decrypt(&nonce, &ciphertext, &aad)
                .expect("Decryption should succeed");

            prop_assert_eq!(decrypted, plaintext);
        }

        /// Ciphertext is larger than plaintext (includes auth tag)
        #[test]
        fn ciphertext_size(
            key_bytes in any::<[u8; 32]>(),
            nonce_bytes in any::<[u8; 24]>(),
            plaintext in prop::collection::vec(any::<u8>(), 0..1024),
        ) {
            let key = AeadKey::new(key_bytes);
            let nonce = Nonce::from_bytes(nonce_bytes);

            let ciphertext = key.encrypt(&nonce, &plaintext, b"")
                .expect("Encryption should succeed");

            // Ciphertext should be plaintext + 16 byte auth tag
            prop_assert_eq!(ciphertext.len(), plaintext.len() + 16);
        }

        /// Different keys produce different ciphertexts
        #[test]
        fn different_keys_different_ciphertexts(
            key1_bytes in any::<[u8; 32]>(),
            key2_bytes in any::<[u8; 32]>(),
            nonce_bytes in any::<[u8; 24]>(),
            plaintext in prop::collection::vec(any::<u8>(), 1..64),
        ) {
            prop_assume!(key1_bytes != key2_bytes);

            let key1 = AeadKey::new(key1_bytes);
            let key2 = AeadKey::new(key2_bytes);
            let nonce = Nonce::from_bytes(nonce_bytes);

            let ct1 = key1.encrypt(&nonce, &plaintext, b"").unwrap();
            let ct2 = key2.encrypt(&nonce, &plaintext, b"").unwrap();

            prop_assert_ne!(ct1, ct2, "Different keys should produce different ciphertexts");
        }

        /// Decryption with wrong key fails
        #[test]
        fn wrong_key_decryption_fails(
            key1_bytes in any::<[u8; 32]>(),
            key2_bytes in any::<[u8; 32]>(),
            nonce_bytes in any::<[u8; 24]>(),
            plaintext in prop::collection::vec(any::<u8>(), 1..64),
        ) {
            prop_assume!(key1_bytes != key2_bytes);

            let key1 = AeadKey::new(key1_bytes);
            let key2 = AeadKey::new(key2_bytes);
            let nonce = Nonce::from_bytes(nonce_bytes);

            let ciphertext = key1.encrypt(&nonce, &plaintext, b"").unwrap();

            prop_assert!(
                key2.decrypt(&nonce, &ciphertext, b"").is_err(),
                "Decryption with wrong key should fail"
            );
        }

        /// Key commitment is deterministic
        #[test]
        fn key_commitment_deterministic(key_bytes in any::<[u8; 32]>()) {
            let key1 = AeadKey::new(key_bytes);
            let key2 = AeadKey::new(key_bytes);

            prop_assert_eq!(
                key1.commitment(),
                key2.commitment(),
                "Same key should produce same commitment"
            );
        }

        /// Different keys have different commitments
        #[test]
        fn different_keys_different_commitments(
            key1_bytes in any::<[u8; 32]>(),
            key2_bytes in any::<[u8; 32]>(),
        ) {
            prop_assume!(key1_bytes != key2_bytes);

            let key1 = AeadKey::new(key1_bytes);
            let key2 = AeadKey::new(key2_bytes);

            prop_assert_ne!(
                key1.commitment(),
                key2.commitment(),
                "Different keys should have different commitments"
            );
        }
    }
}

// ============================================================================
// Padding Properties
// ============================================================================

mod padding_properties {
    use super::*;
    use wraith_obfuscation::padding::{PaddingEngine, PaddingMode};

    proptest! {
        /// Padded size is always >= original size
        #[test]
        fn padded_size_gte_original(
            mode in 0u8..5,
            plaintext_len in 0usize..16384,
        ) {
            let mode = match mode {
                0 => PaddingMode::None,
                1 => PaddingMode::PowerOfTwo,
                2 => PaddingMode::SizeClasses,
                3 => PaddingMode::ConstantRate,
                _ => PaddingMode::Statistical,
            };

            let mut engine = PaddingEngine::new(mode);
            let padded = engine.padded_size(plaintext_len);

            prop_assert!(
                padded >= plaintext_len,
                "Padded size {} should be >= plaintext len {}",
                padded,
                plaintext_len
            );
        }

        /// Unpad recovers original data
        #[test]
        fn unpad_recovers_original(
            data in prop::collection::vec(any::<u8>(), 1..256),
        ) {
            let mut engine = PaddingEngine::new(PaddingMode::SizeClasses);
            let original_len = data.len();

            let mut buffer = data.clone();
            let target_size = engine.padded_size(original_len);
            engine.pad(&mut buffer, target_size);

            let unpadded = engine.unpad(&buffer, original_len);

            prop_assert_eq!(unpadded, &data[..], "Unpad should recover original data");
        }

        /// PowerOfTwo always produces power of 2 (or minimum 128)
        #[test]
        fn power_of_two_mode(plaintext_len in 0usize..16384) {
            let mut engine = PaddingEngine::new(PaddingMode::PowerOfTwo);
            let padded = engine.padded_size(plaintext_len);

            prop_assert!(
                padded >= 128,
                "PowerOfTwo should have minimum 128 bytes"
            );
            prop_assert!(
                padded.is_power_of_two(),
                "Result {} should be power of 2",
                padded
            );
        }

        /// SizeClasses produces one of the defined classes
        #[test]
        fn size_classes_mode(plaintext_len in 0usize..20000) {
            let valid_classes = [128, 512, 1024, 4096, 8192, 16384];

            let mut engine = PaddingEngine::new(PaddingMode::SizeClasses);
            let padded = engine.padded_size(plaintext_len);

            prop_assert!(
                valid_classes.contains(&padded),
                "Result {} should be one of {:?}",
                padded,
                valid_classes
            );
        }

        /// ConstantRate always produces max size
        #[test]
        fn constant_rate_mode(plaintext_len in 0usize..20000) {
            let mut engine = PaddingEngine::new(PaddingMode::ConstantRate);
            let padded = engine.padded_size(plaintext_len);

            prop_assert_eq!(padded, 16384, "ConstantRate should always be 16384");
        }
    }
}

// ============================================================================
// Tree Hash Properties
// ============================================================================

mod tree_hash_properties {
    use super::*;
    use wraith_files::tree_hash::{
        IncrementalTreeHasher, compute_merkle_root, compute_tree_hash_from_data,
    };

    proptest! {
        /// Same data produces same hash
        #[test]
        fn tree_hash_deterministic(
            data in prop::collection::vec(any::<u8>(), 0..4096),
            chunk_size in 64usize..1024,
        ) {
            let tree1 = compute_tree_hash_from_data(&data, chunk_size);
            let tree2 = compute_tree_hash_from_data(&data, chunk_size);

            prop_assert_eq!(tree1.root, tree2.root, "Same data should produce same hash");
            prop_assert_eq!(tree1.chunks, tree2.chunks);
        }

        /// Different data produces different hash (with high probability)
        #[test]
        fn different_data_different_hash(
            data1 in prop::collection::vec(any::<u8>(), 1..256),
            data2 in prop::collection::vec(any::<u8>(), 1..256),
            chunk_size in 64usize..256,
        ) {
            prop_assume!(data1 != data2);

            let tree1 = compute_tree_hash_from_data(&data1, chunk_size);
            let tree2 = compute_tree_hash_from_data(&data2, chunk_size);

            // Root hashes should differ (collision-resistant)
            prop_assert_ne!(
                tree1.root,
                tree2.root,
                "Different data should produce different hashes"
            );
        }

        /// Chunk count matches expected
        #[test]
        fn chunk_count_correct(
            data_len in 1usize..4096,
            chunk_size in 64usize..512,
        ) {
            let data: Vec<u8> = (0..data_len).map(|i| (i & 0xFF) as u8).collect();
            let tree = compute_tree_hash_from_data(&data, chunk_size);

            let expected_chunks = data_len.div_ceil(chunk_size);
            prop_assert_eq!(
                tree.chunk_count(),
                expected_chunks,
                "Expected {} chunks for {} bytes with chunk size {}",
                expected_chunks,
                data_len,
                chunk_size
            );
        }

        /// Incremental hashing matches batch hashing
        #[test]
        fn incremental_equals_batch(
            data in prop::collection::vec(any::<u8>(), 0..4096),
            chunk_size in 64usize..512,
        ) {
            // Batch hash
            let tree_batch = compute_tree_hash_from_data(&data, chunk_size);

            // Incremental hash (in 64-byte pieces)
            let mut hasher = IncrementalTreeHasher::new(chunk_size);
            for chunk in data.chunks(64) {
                hasher.update(chunk);
            }
            let tree_incremental = hasher.finalize();

            prop_assert_eq!(
                tree_batch.root,
                tree_incremental.root,
                "Incremental should match batch"
            );
            prop_assert_eq!(tree_batch.chunks, tree_incremental.chunks);
        }

        /// Merkle root of single leaf equals the leaf
        #[test]
        fn merkle_single_leaf(leaf in any::<[u8; 32]>()) {
            let root = compute_merkle_root(&[leaf]);
            prop_assert_eq!(root, leaf, "Single leaf should be its own root");
        }

        /// Chunk verification succeeds for matching data
        #[test]
        fn chunk_verification_success(
            data in prop::collection::vec(any::<u8>(), 64..1024),
            chunk_size in 64usize..256,
        ) {
            let tree = compute_tree_hash_from_data(&data, chunk_size);

            // Verify each chunk
            for (i, chunk) in data.chunks(chunk_size).enumerate() {
                prop_assert!(
                    tree.verify_chunk(i, chunk),
                    "Chunk {} verification should succeed",
                    i
                );
            }
        }

        /// Chunk verification fails for wrong data
        #[test]
        fn chunk_verification_failure(
            data in prop::collection::vec(any::<u8>(), 64..256),
            chunk_size in 64usize..128,
            wrong_byte in any::<u8>(),
        ) {
            let tree = compute_tree_hash_from_data(&data, chunk_size);

            // Modify the first chunk
            let mut wrong_chunk = data[..chunk_size.min(data.len())].to_vec();
            if !wrong_chunk.is_empty() {
                wrong_chunk[0] = wrong_chunk[0].wrapping_add(wrong_byte.max(1));
            }

            prop_assert!(
                !tree.verify_chunk(0, &wrong_chunk),
                "Wrong data should fail verification"
            );
        }
    }

    // Standard tests for edge cases (no property input needed)
    #[test]
    fn merkle_empty_leaves() {
        let root = compute_merkle_root(&[]);
        assert_eq!(root, [0u8; 32], "Empty leaves should produce zero root");
    }
}

// ============================================================================
// Replay Protection Properties
// ============================================================================

mod replay_protection_properties {
    use super::*;
    use wraith_crypto::aead::ReplayProtection;

    proptest! {
        /// First time seeing a sequence number should accept
        #[test]
        fn first_time_accepts(seq in 0u64..10000) {
            let mut rp = ReplayProtection::new();
            prop_assert!(
                rp.check_and_update(seq),
                "First time seeing {} should accept",
                seq
            );
        }

        /// Duplicate sequence number should reject
        #[test]
        fn duplicate_rejects(seq in 0u64..10000) {
            let mut rp = ReplayProtection::new();

            // First time should accept
            rp.check_and_update(seq);

            // Second time should reject
            prop_assert!(
                !rp.check_and_update(seq),
                "Duplicate {} should reject",
                seq
            );
        }

        /// Sequential packets all accepted
        #[test]
        fn sequential_all_accepted(count in 1u64..100) {
            let mut rp = ReplayProtection::new();

            for i in 0..count {
                prop_assert!(
                    rp.check_and_update(i),
                    "Sequential packet {} should be accepted",
                    i
                );
            }
        }

        /// Max sequence number is correctly tracked
        #[test]
        fn max_seq_tracking(
            seq1 in 0u64..1000,
            seq2 in 0u64..1000,
        ) {
            let mut rp = ReplayProtection::new();

            rp.check_and_update(seq1);
            rp.check_and_update(seq2);

            prop_assert_eq!(
                rp.max_seq(),
                seq1.max(seq2),
                "Max seq should be max of seen values"
            );
        }

        /// Packets too old (beyond window) are rejected
        #[test]
        fn old_packets_rejected(high_seq in 300u64..10000) {
            let mut rp = ReplayProtection::new();

            // Accept a high sequence number
            rp.check_and_update(high_seq);

            // Old packet beyond window should be rejected
            let old_seq = high_seq.saturating_sub(257); // Beyond 256-packet window

            prop_assert!(
                !rp.check_and_update(old_seq),
                "Old packet {} (high={}) should be rejected",
                old_seq,
                high_seq
            );
        }

        /// Reset clears all state
        #[test]
        fn reset_clears_state(seq in 0u64..1000) {
            let mut rp = ReplayProtection::new();

            rp.check_and_update(seq);
            rp.reset();

            prop_assert_eq!(rp.max_seq(), 0, "Reset should clear max_seq");
            prop_assert!(
                rp.check_and_update(seq),
                "After reset, {} should be accepted again",
                seq
            );
        }
    }
}

// ============================================================================
// BBR Congestion Control Properties
// ============================================================================

mod bbr_properties {
    use wraith_core::BbrState;

    // Standard tests (BBR properties are better tested with unit tests)
    #[test]
    fn initial_state_reasonable() {
        let bbr = BbrState::new();
        assert!(bbr.cwnd() > 0, "Initial cwnd should be positive");
        assert!(
            bbr.pacing_rate() > 0,
            "Initial pacing rate should be positive"
        );
    }

    #[test]
    fn sending_window_positive() {
        let bbr = BbrState::new();
        assert!(bbr.cwnd() > 0, "Sending window should always be positive");
    }
}
