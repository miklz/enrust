use rand::Rng;
use std::sync::atomic::{AtomicU64, Ordering};

/// Zobriest structure that holds all pre-computed random numbers
#[derive(Clone)]
pub struct Zobrist {
    // [square_index][piece_index]
    pub pieces: [[u64; 12]; 64],
    pub side_to_move: u64,
    // [WQS, WKS, BQS, BKS]
    pub castling_rights: [u64; 4],
    // [file a, file b, ..., file h]
    pub en_passant: [u64; 8],
}

impl Zobrist {
    // Generates the random numbers. This should only be called once.
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let mut zobrist = Zobrist {
            pieces: [[0; 12]; 64],
            side_to_move: rng.random(),
            castling_rights: [rng.random(), rng.random(), rng.random(), rng.random()],
            en_passant: [
                rng.random(),
                rng.random(),
                rng.random(),
                rng.random(),
                rng.random(),
                rng.random(),
                rng.random(),
                rng.random(),
            ],
        };

        for square in 0..64 {
            for piece in 0..12 {
                zobrist.pieces[square][piece] = rng.random();
            }
        }
        zobrist
    }
}

impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
    }
}

// 64-bit data layout:
// 0-15:  score (i16 stored as u16)
// 16-23: depth (u8)
// 24-25: node type (2 bits: 0=exact, 1=upper, 2=lower)
// 26-41: best move (16 bits: from 6 bits (0-63), to 6 bits (0-63), promoted_piece - queen 0x1, rook 0x2, bishop 0x4, knight 0x8)
// 42-49: age (u8)
// 50-63: unused (14 bits) - can be used for extended info
pub struct TranspositionTableData {
    pub score: i16,          // Evaluation score
    pub depth: u8,           // Search depth this entry was computed at
    pub node_type: NodeType, // Exact, LowerBound, UpperBound
    pub best_move: u16,      // Best move found in compact form (uci characters)
    pub age: u8,             // For replacement schemes
}

pub struct TranspositionEntry {
    pub hash_xor_data: AtomicU64,
    pub data: AtomicU64,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NodeType {
    Exact,      // Score is exact evaluation
    LowerBound, // Score is at least this good (alpha)
    UpperBound, // Score is at most this good (beta)
}

impl TryFrom<u8> for NodeType {
    type Error = (); // You can define a custom error type here

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NodeType::Exact),
            1 => Ok(NodeType::LowerBound),
            2 => Ok(NodeType::UpperBound),
            _ => Err(()), // Handle invalid u8 values
        }
    }
}

pub struct TranspositionTable {
    entries: Box<[TranspositionEntry]>,
    size: usize,
}

impl TranspositionEntry {
    fn score(data: u64) -> i16 {
        ((data & 0xFFFF) as u16) as i16
    }

    fn depth(data: u64) -> u8 {
        ((data >> 16) & 0xFF) as u8
    }

    fn node_type(data: u64) -> NodeType {
        let node_type = ((data >> 24) & 0b11) as u8;
        NodeType::try_from(node_type).unwrap()
    }

    fn best_move(data: u64) -> u16 {
        ((data >> 26) & 0xFFFF) as u16
    }

    fn age(data: u64) -> u8 {
        ((data >> 42) & 0xFF) as u8
    }

    fn get_hash_xor_data(&self) -> u64 {
        self.hash_xor_data.load(Ordering::Relaxed)
    }

    fn get_data(&self) -> u64 {
        self.data.load(Ordering::Relaxed)
    }

    fn set_hash_xor_data(&self, hash: u64) {
        self.hash_xor_data.store(hash, Ordering::Release);
    }

    fn set_data(&self, data: u64) {
        self.data.store(data, Ordering::Release);
    }

    fn new() -> Self {
        Self {
            hash_xor_data: AtomicU64::new(0),
            data: AtomicU64::new(0),
        }
    }

    fn is_empty(&self) -> bool {
        self.data.load(Ordering::Relaxed) == 0 && self.hash_xor_data.load(Ordering::Relaxed) == 0
    }
}

impl TranspositionTable {
    /// Creates a new table. `size_mb` is the desired size in Megabytes.
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TranspositionEntry>();
        let size = (size_mb * 1024 * 1024) / entry_size;

        let entries: Vec<TranspositionEntry> =
            (0..size).map(|_| TranspositionEntry::new()).collect();

        Self {
            entries: entries.into_boxed_slice(),
            size,
        }
    }

    /// Resizes the table to a new size in Megabytes.
    pub fn resize(&mut self, new_size_mb: usize) {
        let entry_size = std::mem::size_of::<Option<TranspositionEntry>>();
        let new_size = (new_size_mb * 1024 * 1024) / entry_size;

        let entries: Vec<TranspositionEntry> =
            (0..new_size).map(|_| TranspositionEntry::new()).collect();

        self.entries = entries.into_boxed_slice();
        self.size = new_size;
    }

    fn probe(&self, hash: u64) -> Option<u64> {
        let index = (hash % self.size as u64) as usize;

        if !&self.entries[index].is_empty() {
            let entry = &self.entries[index];
            let entry_xor = entry.get_hash_xor_data();
            let entry_data = entry.get_data();
            if (entry_xor ^ entry_data) == hash {
                return Some(entry_data);
            }
        }

        None
    }

    fn store(&self, hash: u64, data: u64) {
        let index = (hash % self.size as u64) as usize;
        let hash_xor_data = hash ^ data;

        // Replacement scheme: always replace if:
        // 1. Empty slot
        // 2. Different position (hash collision)
        // 3. Same position but deeper search or newer
        if !&self.entries[index].is_empty() {
            let existing = &self.entries[index];
            // Get atomic values
            let existing_xor = existing.get_hash_xor_data();

            if existing_xor == hash_xor_data {
                let existing_data = existing.get_data();
                // Same position - replace if deeper search or same depth but newer
                if TranspositionEntry::depth(data) > TranspositionEntry::depth(existing_data)
                    || (TranspositionEntry::depth(data) == TranspositionEntry::depth(existing_data)
                        && TranspositionEntry::age(data) >= TranspositionEntry::age(existing_data))
                {
                    self.entries[index].set_hash_xor_data(hash_xor_data);
                    self.entries[index].set_data(data);
                }
            } else {
                // Hash collision - use replacement policy
                if self.should_replace(existing, data) {
                    self.entries[index].set_hash_xor_data(hash_xor_data);
                    self.entries[index].set_data(data);
                }
            }
        } else {
            self.entries[index].set_hash_xor_data(hash_xor_data);
            self.entries[index].set_data(data);
        }
    }

    fn should_replace(&self, existing: &TranspositionEntry, new_data: u64) -> bool {
        let existing_data_entry = existing.get_data();
        // Prefer entries from current search (age)
        if TranspositionEntry::age(new_data) != TranspositionEntry::age(existing_data_entry) {
            return TranspositionEntry::age(new_data)
                == TranspositionEntry::age(existing_data_entry);
        }

        // Prefer deeper searches
        if TranspositionEntry::depth(new_data) != TranspositionEntry::depth(existing_data_entry) {
            return TranspositionEntry::depth(new_data)
                > TranspositionEntry::depth(existing_data_entry);
        }

        // As last resort, prefer exact scores over bounds
        let new_node_type = TranspositionEntry::node_type(new_data);
        let existing_node_type = TranspositionEntry::node_type(existing_data_entry);
        matches!(
            (new_node_type, existing_node_type),
            (NodeType::Exact, _) | (_, NodeType::UpperBound) | (_, NodeType::LowerBound)
        )
    }

    /*
    fn new_search(&mut self) {
        self.age = self.age.wrapping_add(1);
    }
    */

    pub fn save_position(&self, hash: u64, transposition_data: &TranspositionTableData) {
        let mut data_u64 = 0;

        // Pack Entry into 64 bits
        data_u64 |= transposition_data.score as u16 as u64;
        data_u64 |= (transposition_data.depth as u64) << 16;
        data_u64 |= (transposition_data.node_type as u64) << 24;
        data_u64 |= (transposition_data.best_move as u64) << 26;
        data_u64 |= (transposition_data.age as u64) << 42;

        self.store(hash, data_u64);
    }

    pub fn retrieve_position(&self, hash: u64) -> Option<TranspositionTableData> {
        if let Some(data) = self.probe(hash) {
            let saved_position = TranspositionTableData {
                score: TranspositionEntry::score(data),
                depth: TranspositionEntry::depth(data),
                node_type: TranspositionEntry::node_type(data),
                best_move: TranspositionEntry::best_move(data),
                age: TranspositionEntry::age(data),
            };

            return Some(saved_position);
        }

        None
    }
}
