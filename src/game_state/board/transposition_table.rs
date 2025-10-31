use rand::Rng;
use std::sync::atomic::{AtomicU64, Ordering};

/// Pre-computed random numbers for Zobrist hashing of chess positions.
///
/// Zobrist hashing is a technique used to uniquely identify chess positions
/// by XOR-ing random numbers associated with each piece placement and game state.
/// This enables efficient position tracking and repetition detection.
///
/// # Fields
/// - `pieces`: 64 squares × 12 pieces (6 types × 2 colors) random values
/// - `side_to_move`: Random value for white/black turn
/// - `castling_rights`: 4 random values for [White QueenSide, White KingSide, Black QueenSide, Black KingSide]
/// - `en_passant`: 8 random values for each file where en passant can occur
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
    /// Generates a new Zobrist structure with cryptographically secure random numbers.
    ///
    /// This should be called once and shared across all board instances for consistency.
    /// Uses `rand::rng()` for random number generation.
    ///
    /// # Performance
    /// - Initialization is O(64×12) = 768 random number generations
    /// - Should be done once at program start
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

/// Compact 64-bit representation of transposition table data.
///
/// Bit layout:
/// - Bits 0-15:   score (i16 stored as u16 using two's complement)
/// - Bits 16-23:  depth (u8, 0-255 plies)
/// - Bits 24-25:  node type (2 bits: 0=exact, 1=upper, 2=lower)
/// - Bits 26-41:  best move (16 bits: from_square 6b, to_square 6b, promotion 4b)
/// - Bits 42-49:  age (u8, for replacement schemes)
/// - Bits 50-63:  unused/reserved (14 bits)
///
/// # Best Move Encoding
/// The 16-bit best move is encoded as:
/// - Bits 0-5:   from square (0-63)
/// - Bits 6-11:  to square (0-63)
/// - Bits 12-15: promotion piece flags (queen=0x1, rook=0x2, bishop=0x4, knight=0x8)
pub struct TranspositionTableData {
    pub score: i16,          // Evaluation score
    pub depth: u8,           // Search depth this entry was computed at
    pub node_type: NodeType, // Exact, LowerBound, UpperBound
    pub best_move: u16,      // Best move found in compact form (uci characters)
    pub age: u8,             // For replacement schemes
}

/// A single lock-free entry in the transposition table using XOR verification.
///
/// Uses two AtomicU64 values to enable lock-free concurrent access:
/// - `hash_xor_data`: hash_key ^ packed_data for consistency verification
/// - `data`: packed TranspositionTableData as u64
///
/// # Safety
/// The XOR verification ensures that reads either get complete valid data
/// or detect corruption through hash mismatch, preventing torn reads.
pub struct TranspositionEntry {
    pub hash_xor_data: AtomicU64,
    pub data: AtomicU64,
}

/// Classification of transposition table entries for alpha-beta search.
///
/// Determines how the stored score should be interpreted during search.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NodeType {
    /// The score is an exact evaluation of the position.
    /// Can be used directly without re-searching.
    Exact,
    /// The score is a lower bound (alpha cutoff occurred).
    /// The true evaluation is at least this good.
    LowerBound,
    /// The score is an upper bound (beta cutoff occurred).
    /// The true evaluation is at most this good.
    UpperBound,
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

/// A lock-free transposition table for caching chess position evaluations.
///
/// Stores previously computed search results to avoid re-searching positions.
/// Uses atomic operations and XOR verification for thread-safe concurrent access.
///
/// # Replacement Policy
/// Implements a depth-preferred replacement scheme:
/// 1. Always replace empty slots
/// 2. For same position: replace if deeper search or same depth but newer
/// 3. For collisions: replace based on depth, age, and node type priority
pub struct TranspositionTable {
    entries: Box<[TranspositionEntry]>,
    size: usize,
}

impl TranspositionEntry {
    /// Extracts the score from packed 64-bit data.
    ///
    /// # Arguments
    /// * `data` - Packed 64-bit value containing all entry data
    ///
    /// # Returns
    /// The evaluation score as i16 (supports negative values via two's complement)
    fn score(data: u64) -> i16 {
        ((data & 0xFFFF) as u16) as i16
    }

    /// Extracts the depth from packed 64-bit data.
    ///
    /// # Arguments
    /// * `data` - Packed 64-bit value containing all entry data
    ///
    /// # Returns
    /// The depth value as u8
    fn depth(data: u64) -> u8 {
        ((data >> 16) & 0xFF) as u8
    }

    /// Extracts the node type from packed 64-bit data.
    ///
    /// # Arguments
    /// * `data` - Packed 64-bit value containing all entry data
    ///
    /// # Returns
    /// The Node Type (Exact, LowerBound, UpperBound) value
    fn node_type(data: u64) -> NodeType {
        let node_type = ((data >> 24) & 0b11) as u8;
        NodeType::try_from(node_type).unwrap()
    }

    /// Extracts the encoded best move from packed 64-bit data.
    ///
    /// # Arguments
    /// * `data` - Packed 64-bit value containing all entry data
    ///
    /// # Returns
    /// The encoded best move (<from><to><promotion>) in 16 bits as u16
    fn best_move(data: u64) -> u16 {
        ((data >> 26) & 0xFFFF) as u16
    }

    /// Extracts the age of the entry from packed 64-bit data.
    ///
    /// # Arguments
    /// * `data` - Packed 64-bit value containing all entry data
    ///
    /// # Returns
    /// The entry age as u8
    fn age(data: u64) -> u8 {
        ((data >> 42) & 0xFF) as u8
    }

    /// Loads the XOR verification field using relaxed memory ordering.
    ///
    /// # Returns
    /// The current value of `hash_xor_data` (hash_key ^ packed_data)
    ///
    /// # Memory Ordering
    /// Uses `Ordering::Relaxed` because this is typically called as part of
    /// a larger atomic operation sequence where the probe method handles
    /// proper synchronization.
    ///
    /// # Safety
    /// This method alone doesn't guarantee consistency - always use with
    /// `get_data()` and verify with XOR operation.
    fn get_hash_xor_data(&self) -> u64 {
        self.hash_xor_data.load(Ordering::Relaxed)
    }

    /// Loads the packed data field using relaxed memory ordering.
    ///
    /// # Returns
    /// The packed 64-bit value containing score, depth, node type, best move, and age
    ///
    /// # Memory Ordering
    /// Uses `Ordering::Relaxed` because consistency is verified through the
    /// XOR check in the probe method. Multiple reads may be performed to
    /// detect torn writes.
    ///
    /// # Usage
    /// Always call this in conjunction with `get_hash_xor_data()` and
    /// verify: `(hash_xor_data ^ data) == expected_hash`
    fn get_data(&self) -> u64 {
        self.data.load(Ordering::Relaxed)
    }

    /// Stores a new XOR verification value with release memory ordering.
    ///
    /// # Arguments
    /// * `hash` - The new hash_xor_data value (hash_key ^ packed_data)
    ///
    /// # Memory Ordering
    /// Uses `Ordering::Release` to ensure all previous writes are visible
    /// to other threads that acquire this store. This creates a happens-before
    /// relationship with any subsequent `get_data()` calls using `Acquire`.
    ///
    /// # Typical Usage
    /// This should be called before `set_data()` when initializing an entry,
    /// but after `set_data()` when updating to maintain consistency.
    fn set_hash_xor_data(&self, hash: u64) {
        self.hash_xor_data.store(hash, Ordering::Release);
    }

    /// Stores new packed data with release memory ordering.
    ///
    /// # Arguments
    /// * `data` - Packed 64-bit value containing all transposition data
    ///
    /// # Memory Ordering
    /// Uses `Ordering::Release` to ensure this write and all previous writes
    /// are visible to other threads that perform an acquire load. This prevents
    /// other threads from seeing stale or partially updated data.
    ///
    /// # Consistency
    /// For proper XOR verification, the corresponding `hash_xor_data` should
    /// be set to `hash_key ^ data` either before or after this call, depending
    /// on the desired atomicity guarantees.
    fn set_data(&self, data: u64) {
        self.data.store(data, Ordering::Release);
    }

    /// Creates a new, empty transposition table entry.
    ///
    /// # Returns
    /// A `TranspositionEntry` with both atomic fields initialized to zero.
    ///
    /// # Initial State
    /// - `hash_xor_data`: 0
    /// - `data`: 0
    /// - `is_empty()` will return `true`
    ///
    /// # Usage
    /// Used during transposition table initialization to populate the entry array.
    /// Empty entries are skipped during probing until they are written to.
    fn new() -> Self {
        Self {
            hash_xor_data: AtomicU64::new(0),
            data: AtomicU64::new(0),
        }
    }

    /// Checks if the entry is empty (both fields are zero).
    ///
    /// Used for quick emptiness checks without full hash verification.
    fn is_empty(&self) -> bool {
        self.data.load(Ordering::Relaxed) == 0 && self.hash_xor_data.load(Ordering::Relaxed) == 0
    }
}

impl TranspositionTable {
    /// Creates a new transposition table with the specified size.
    ///
    /// # Arguments
    /// * `size_mb` - Table size in megabytes
    ///
    /// # Calculation
    /// Total entries = (size_mb × 1024 × 1024) / size_of::<TranspositionEntry>()
    /// Each entry is 16 bytes (two u64), so 1MB holds ~65,536 entries
    ///
    /// # Example
    /// ```
    /// use enrust::game_state::TranspositionTable;
    ///
    /// let tt = TranspositionTable::new(128); // 128MB table, ~8M entries
    /// ```
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

    /// Resizes the transposition table to a new size, discarding all existing entries.
    ///
    /// # Arguments
    /// * `new_size_mb` - New table size in megabytes
    ///
    /// # Behavior
    /// - Allocates a new entry array of the specified size
    /// - Initializes all entries to empty state (zero values)
    /// - Discards all previously stored positions (complete cache flush)
    /// - Updates the internal size tracking
    ///
    /// # Memory Calculation
    /// Uses `size_of::<TranspositionEntry>()` to store raw entries.
    ///
    /// # Example
    /// ```
    /// use enrust::game_state::TranspositionTable;
    ///
    /// let mut tt = TranspositionTable::new(64); // 64MB table
    /// tt.resize(128); // Now 128MB table, all previous entries lost
    /// ```
    ///
    /// # Note
    /// This operation is expensive and should be used sparingly, typically
    /// during engine configuration rather than during search.
    pub fn resize(&mut self, new_size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TranspositionEntry>();
        let new_size = (new_size_mb * 1024 * 1024) / entry_size;

        let entries: Vec<TranspositionEntry> =
            (0..new_size).map(|_| TranspositionEntry::new()).collect();

        Self {
            entries: entries.into_boxed_slice(),
            size: new_size,
        }
    }

    /// Internal method to probe the transposition table for a specific hash.
    ///
    /// # Arguments
    /// * `hash` - Zobrist hash of the position to look up
    ///
    /// # Returns
    /// * `Some(u64)` - Packed data if a valid entry is found and passes XOR verification
    /// * `None` - If no entry exists or hash verification fails
    ///
    /// # Algorithm
    /// 1. Compute index using modulo operation: `hash % table_size`
    /// 2. Check if entry is non-empty (quick check without verification)
    /// 3. Load both atomic fields with relaxed ordering
    /// 4. Verify data consistency: `(hash_xor_data ^ data) == hash`
    /// 5. Return data only if verification passes
    ///
    /// # XOR Verification
    /// The verification ensures that:
    /// - The data wasn't corrupted during concurrent writes
    /// - The entry actually belongs to the requested position (not a hash collision)
    /// - The read captured a consistent snapshot of both fields
    ///
    /// # Performance
    /// This method is designed for minimal overhead in the hot path of search.
    /// Uses relaxed memory ordering since XOR verification provides the consistency guarantee.
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

    /// Internal method to store an entry in the transposition table.
    ///
    /// # Arguments
    /// * `hash` - Zobrist hash of the position
    /// * `data` - Packed 64-bit data containing evaluation results
    ///
    /// # Replacement Strategy
    /// Implements a depth-preferential replacement policy with three cases:
    ///
    /// ## 1. Empty Slot
    /// Always store the new entry when the target slot is empty.
    ///
    /// ## 2. Same Position (Hash Match)
    /// Replace existing entry if:
    /// - New search depth is greater than existing depth, OR
    /// - Same depth but new entry has equal or newer age
    ///
    /// ## 3. Different Position (Hash Collision)
    /// Use `should_replace()` heuristic to decide whether to replace:
    /// - Prefer entries from current search generation (age)
    /// - Prefer deeper searches
    /// - Prefer exact scores over bound scores
    ///
    /// # Memory Ordering
    /// Uses release stores for both fields to ensure proper visibility
    /// to other threads. The order of stores doesn't matter for consistency
    /// since readers verify with XOR and handle torn reads.
    ///
    /// # Thread Safety
    /// This method is lock-free and can be called concurrently from multiple
    /// threads. Hash collisions are handled gracefully with the replacement policy.
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

    /// Internal replacement policy for hash collisions.
    ///
    /// Determines whether a new entry should replace an existing one when
    /// different positions hash to the same table index.
    ///
    /// # Priority Order
    /// 1. **Age**: Prefer entries from the current search generation
    /// 2. **Depth**: Prefer entries from deeper searches
    /// 3. **Node Type**: Prefer exact scores over bound scores
    ///
    /// # Arguments
    /// * `existing` - Reference to the currently stored entry
    /// * `new_data` - Packed data of the new entry to consider
    ///
    /// # Returns
    /// `true` if the new entry should replace the existing one, `false` otherwise
    ///
    /// # Note
    /// This is a simplified replacement policy. More sophisticated engines
    /// might consider additional factors like search time or node count.
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

    /// Stores a position evaluation in the transposition table.
    ///
    /// # Arguments
    /// * `hash` - Zobrist hash of the position
    /// * `transposition_data` - Evaluation results to store
    ///
    /// # Thread Safety
    /// This method is lock-free and can be called concurrently from multiple threads.
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

    /// Retrieves a position evaluation from the transposition table.
    ///
    /// # Arguments
    /// * `hash` - Zobrist hash of the position to look up
    ///
    /// # Returns
    /// * `Some(TranspositionTableData)` if valid entry found
    /// * `None` if no entry exists or hash verification fails
    ///
    /// # Verification
    /// Uses XOR verification to ensure data consistency and prevent hash collisions.
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
