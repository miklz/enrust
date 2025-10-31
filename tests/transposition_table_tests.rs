#[cfg(test)]
mod basic_tests {
    use enrust::game_state::board::transposition_table::{
        NodeType, TranspositionTable, TranspositionTableData,
    };

    #[test]
    fn test_basic_store_and_probe() {
        let tt = TranspositionTable::new(4); // 4 MB table

        let hash = 0x123456789ABCDEF0;
        let data = TranspositionTableData {
            score: 150,
            depth: 8,
            node_type: NodeType::Exact,
            best_move: 0x1543,
            age: 1,
        };

        // Store and retrieve
        tt.save_position(hash, &data);

        let retrieved = tt.retrieve_position(hash).unwrap();
        assert_eq!(retrieved.score, 150);
        assert_eq!(retrieved.depth, 8);
        assert_eq!(retrieved.node_type, NodeType::Exact);
        assert_eq!(retrieved.best_move, 0x1543);
    }

    #[test]
    fn test_hash_collision_handling() {
        let tt = TranspositionTable::new(4);

        let hash1 = 0x123456789ABCDEF0;
        let hash2 = 0x123456789ABCDEF1; // Different hash

        let data1 = TranspositionTableData {
            score: 100,
            depth: 6,
            node_type: NodeType::Exact,
            best_move: 0x1543,
            age: 1,
        };

        let data2 = TranspositionTableData {
            score: -50,
            depth: 4,
            node_type: NodeType::UpperBound,
            best_move: 0x2543,
            age: 1,
        };

        // Store both entries
        tt.save_position(hash1, &data1);
        tt.save_position(hash2, &data2);

        // Each should retrieve their own data
        let retrieved1 = tt.retrieve_position(hash1).unwrap();
        let retrieved2 = tt.retrieve_position(hash2).unwrap();

        assert_eq!(retrieved1.score, 100);
        assert_eq!(retrieved2.score, -50);
    }

    #[test]
    fn test_empty_entry_returns_none() {
        let tt = TranspositionTable::new(4);

        let hash = 0x123456789ABCDEF0;

        // Should return None for non-existent entry
        assert!(tt.retrieve_position(hash).is_none());
    }

    #[test]
    fn test_overwrite_behavior() {
        let tt = TranspositionTable::new(4);

        let hash = 0x123456789ABCDEF0;

        let data1 = TranspositionTableData {
            score: 100,
            depth: 6,
            node_type: NodeType::Exact,
            best_move: 0x1543,
            age: 1,
        };

        let data2 = TranspositionTableData {
            score: 200,
            depth: 8,
            node_type: NodeType::UpperBound,
            best_move: 0x2543,
            age: 1,
        };

        // Store first entry
        tt.save_position(hash, &data1);
        assert_eq!(tt.retrieve_position(hash).unwrap().score, 100);

        // Overwrite with second entry
        tt.save_position(hash, &data2);
        assert_eq!(tt.retrieve_position(hash).unwrap().score, 200);
    }
}

#[cfg(test)]
mod xor_verification_tests {
    use enrust::game_state::board::transposition_table::{
        NodeType, TranspositionTable, TranspositionTableData,
    };

    #[test]
    fn test_xor_verification_prevents_wrong_data() {
        let tt = TranspositionTable::new(4);

        let hash1 = 0x123456789ABCDEF0;
        let hash2 = 0xFEDCBA9876543210; // Different hash

        let data = TranspositionTableData {
            score: 150,
            depth: 8,
            node_type: NodeType::Exact,
            best_move: 0x1543,
            age: 1,
        };

        // Store with hash1
        tt.save_position(hash1, &data);

        // Try to retrieve with wrong hash - should return None
        assert!(tt.retrieve_position(hash2).is_none());

        // Retrieve with correct hash - should work
        assert!(tt.retrieve_position(hash1).is_some());
    }

    #[test]
    fn test_torn_read_protection() {
        let tt = TranspositionTable::new(4);

        let hash = 0x123456789ABCDEF0;
        let data = TranspositionTableData {
            score: 150,
            depth: 8,
            node_type: NodeType::Exact,
            best_move: 0x1543,
            age: 1,
        };

        tt.save_position(hash, &data);

        // The XOR verification should prevent reading corrupted data
        // even if we have hash collisions
        for _ in 0..1000 {
            let result = tt.retrieve_position(hash);
            assert!(result.is_some());
            let retrieved = result.unwrap();
            assert_eq!(retrieved.score, 150);
            assert_eq!(retrieved.depth, 8);
        }
    }
}

mod concurrent_thread_access_tests {
    use std::sync::Arc;
    use std::thread;

    use enrust::game_state::board::transposition_table::{
        NodeType, TranspositionTable, TranspositionTableData,
    };

    #[test]
    fn test_concurrent_writes() {
        let tt = Arc::new(TranspositionTable::new(8)); // 8 MB table

        let mut handles = vec![];

        for thread_id in 1..=8 {
            let tt_clone = tt.clone();
            let handle = thread::spawn(move || {
                for i in 0..1000 {
                    let hash: u64 = (thread_id * 10000) + i;
                    let data = TranspositionTableData {
                        score: (thread_id * 100 + i) as i16,
                        depth: (i % 20) as u8,
                        node_type: NodeType::try_from(i as u8 % 3).unwrap(),
                        best_move: (i * 10) as u16,
                        age: (thread_id % 4) as u8,
                    };

                    // Write from multiple threads
                    tt_clone.save_position(hash, &data);

                    // Immediate read back
                    let retrieved = tt_clone.retrieve_position(hash);
                    assert!(
                        retrieved.is_some(),
                        "Thread {} failed to read back entry {}",
                        thread_id,
                        i
                    );

                    let retrieved_data = retrieved.unwrap();
                    assert_eq!(retrieved_data.score, (thread_id * 100 + i) as i16);
                    assert_eq!(retrieved_data.depth, (i % 20) as u8);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_concurrent_reads_and_writes() {
        let tt = Arc::new(TranspositionTable::new(8));

        // Pre-populate some entries
        for i in 0..1000 {
            let hash = i * 100;
            let data = TranspositionTableData {
                score: i as i16,
                depth: (i % 10) as u8,
                node_type: NodeType::Exact,
                best_move: (i * 5) as u16,
                age: 1,
            };
            tt.save_position(hash, &data);
        }

        let mut handles = vec![];

        // Reader threads
        for _thread_id in 0..4 {
            let tt_clone = tt.clone();
            let handle = thread::spawn(move || {
                for i in 0..500 {
                    let hash = (i * 100) % 1000;
                    if let Some(entry) = tt_clone.retrieve_position(hash) {
                        assert_eq!(entry.score, (hash / 100) as i16);
                    }
                }
            });
            handles.push(handle);
        }

        // Writer threads
        for thread_id in 0..4 {
            let tt_clone = tt.clone();
            let handle = thread::spawn(move || {
                for i in 0..500 {
                    let hash = 1000 + (thread_id * 1000) + i;
                    let data = TranspositionTableData {
                        score: -(i as i16),
                        depth: (i % 15) as u8,
                        node_type: NodeType::LowerBound,
                        best_move: (i * 3) as u16,
                        age: 2,
                    };
                    tt_clone.save_position(hash, &data);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}

mod transposition_logic_tests {
    use enrust::game_state::GameState;
    use enrust::game_state::Move;
    use enrust::game_state::Piece;
    use enrust::game_state::board::transposition_table::{
        NodeType, TranspositionTable, TranspositionTableData,
    };

    #[test]
    fn test_replace_if_better_logic() {
        let tt = TranspositionTable::new(4);

        let hash = 0x123456789ABCDEF0;

        // Lower depth entry
        let shallow_data = TranspositionTableData {
            score: 100,
            depth: 4,
            node_type: NodeType::Exact,
            best_move: 0x1543,
            age: 1,
        };

        // Higher depth entry - should be considered better
        let deep_data = TranspositionTableData {
            score: 120, // Different score to verify it actually gets replaced
            depth: 8,
            node_type: NodeType::Exact,
            best_move: 0x2543,
            age: 1,
        };

        // Store shallow entry first
        tt.save_position(hash, &shallow_data);
        assert_eq!(tt.retrieve_position(hash).unwrap().depth, 4);
        assert_eq!(tt.retrieve_position(hash).unwrap().score, 100);

        // Replace previous entry with deeper search entry
        tt.save_position(hash, &deep_data);
        let retrieved = tt.retrieve_position(hash).unwrap();
        assert_eq!(retrieved.depth, 8);
        assert_eq!(retrieved.score, 120);
    }

    #[test]
    fn test_tt_data_packing_roundtrip() {
        let tt = TranspositionTable::new(4);

        // Test that packing and unpacking TranspositionTableData works correctly
        let original = TranspositionTableData {
            score: -32768,              // Minimum i16
            depth: 255,                 // Maximum u8
            node_type: NodeType::Exact, // Maximum 2-bit value
            best_move: 0xFFFF,          // Maximum u16
            age: 255,                   // Maximum u8
        };

        tt.save_position(1234, &original);

        let unpacked = tt.retrieve_position(1234).unwrap();

        assert_eq!(original.score, unpacked.score);
        assert_eq!(original.depth, unpacked.depth);
        assert_eq!(original.node_type, unpacked.node_type);
        assert_eq!(original.best_move, unpacked.best_move);
        assert_eq!(original.age, unpacked.age);

        // Test positive score
        let positive = TranspositionTableData {
            score: 32767, // Maximum i16
            depth: 128,
            node_type: NodeType::LowerBound,
            best_move: 0x1234,
            age: 7,
        };

        tt.save_position(2345, &positive);

        let unpacked_pos = tt.retrieve_position(2345).unwrap();

        assert_eq!(positive.score, unpacked_pos.score);
        assert_eq!(positive.depth, unpacked_pos.depth);
        assert_eq!(positive.node_type, unpacked_pos.node_type);
        assert_eq!(positive.best_move, unpacked_pos.best_move);
        assert_eq!(positive.age, unpacked_pos.age);
    }

    fn setup_game_with_fen(fen: &str) -> GameState {
        let mut game = GameState::new(None);
        game.set_fen_position(fen);
        game
    }

    #[test]
    fn test_move_encoding_decoding() {
        // Test your move encoding if you have compact move representation
        let test_cases = vec![
            (75, 85, None, "8/8/4P3/8/8/8/8/8 w - - 0 1"), // Simple move
            (
                85,
                95,
                Some(Piece::WhiteQueen),
                "3n4/4P3/8/8/8/8/8/8 w - - 0 1",
            ), // Promotion
        ];

        for (from, to, promotion, fen) in test_cases {
            let game = setup_game_with_fen(fen);
            let original_move = Move {
                from,
                to,
                piece: Piece::WhitePawn,
                captured_piece: Piece::EmptySquare,
                promotion,
                castling: None,
                en_passant: false,
                en_passant_square: None,
                previous_en_passant: None,
                previous_castling_rights: None,
            };

            let packed = original_move.encode(&game.get_chess_board());
            let unpacked = Move::decode(packed, &game.get_chess_board()).unwrap();
            // Verify compact move contains correct information
            assert_eq!(original_move.from, unpacked.from);
            assert_eq!(original_move.to, unpacked.to);
            assert_eq!(original_move.promotion, unpacked.promotion);
        }
    }

    #[test]
    fn test_score_packing_roundtrip_extreme_values() {
        let tt = TranspositionTable::new(4);

        // Test extreme values and zero
        let extreme_cases = vec![
            -32768, // i16::MIN
            -16384, 0, 16383, 32767, // i16::MAX
        ];

        for &score in &extreme_cases {
            let data = TranspositionTableData {
                score,
                depth: 12,
                node_type: NodeType::LowerBound,
                best_move: 0xFFFF,
                age: 15,
            };

            tt.save_position(score as u64, &data);

            let unpacked = tt.retrieve_position(score as u64).unwrap();

            assert_eq!(
                unpacked.score, data.score,
                "Extreme value {} failed roundtrip",
                data.score
            );
            assert_eq!(unpacked.depth, data.depth);
            assert_eq!(unpacked.node_type, data.node_type);
            assert_eq!(unpacked.best_move, data.best_move);
            assert_eq!(unpacked.age, data.age);
        }
    }
}
