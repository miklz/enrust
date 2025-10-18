use crate::game_state::Move;
use rand::Rng;

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

#[derive(Clone)]
pub struct TranspositionEntry {
    pub hash: u64,               // Zobrist hash of position
    pub depth: u64,              // Search depth this entry was computed at
    pub score: i64,              // Evaluation score
    pub node_type: NodeType,     // Exact, LowerBound, UpperBound
    pub best_move: Option<Move>, // Best move found
    current_age: u8,             // For replacement schemes
}

#[derive(Clone, Copy)]
pub enum NodeType {
    Exact,      // Score is exact evaluation
    LowerBound, // Score is at least this good (alpha)
    UpperBound, // Score is at most this good (beta)
}

#[derive(Clone)]
pub struct TranspositionTable {
    entries: Vec<Option<TranspositionEntry>>,
    size: usize,
}

impl TranspositionTable {
    /// Creates a new table. `size_mb` is the desired size in Megabytes.
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TranspositionEntry>();
        let size = (size_mb * 1024 * 1024) / entry_size;

        Self {
            entries: vec![None; size],
            size,
        }
    }

    /// Resizes the table to a new size in Megabytes.
    pub fn resize(&mut self, new_size_mb: usize) {
        let entry_size = std::mem::size_of::<Option<TranspositionEntry>>();
        let new_size = (new_size_mb * 1024 * 1024) / entry_size;

        self.entries = vec![None; new_size];
        self.size = new_size;
    }

    pub fn probe(&self, hash: u64) -> Option<&TranspositionEntry> {
        let index = (hash % self.size as u64) as usize;

        if let Some(entry) = &self.entries[index] && entry.hash == hash {
            return Some(entry);
        }

        None
    }

    fn store(&mut self, entry: TranspositionEntry) {
        let index = (entry.hash % self.size as u64) as usize;

        // Replacement scheme: always replace if:
        // 1. Empty slot
        // 2. Different position (hash collision)
        // 3. Same position but deeper search or newer
        if let Some(existing) = &self.entries[index] {
            if existing.hash == entry.hash {
                // Same position - replace if deeper search or same depth but newer
                if entry.depth > existing.depth
                    || (entry.depth == existing.depth && entry.current_age >= existing.current_age)
                {
                    self.entries[index] = Some(entry);
                }
            } else {
                // Hash collision - use replacement policy
                if self.should_replace(existing, &entry) {
                    self.entries[index] = Some(entry);
                }
            }
        } else {
            self.entries[index] = Some(entry);
        }
    }

    fn should_replace(&self, existing: &TranspositionEntry, new: &TranspositionEntry) -> bool {
        // Prefer entries from current search (age)
        if new.current_age != existing.current_age {
            return new.current_age == existing.current_age;
        }

        // Prefer deeper searches
        if new.depth != existing.depth {
            return new.depth > existing.depth;
        }

        // As last resort, prefer exact scores over bounds
        matches!(
            (new.node_type, existing.node_type),
            (NodeType::Exact, _) | (_, NodeType::UpperBound) | (_, NodeType::LowerBound)
        )
    }

    /*
    fn new_search(&mut self) {
        self.current_age = self.current_age.wrapping_add(1);
    }
    */

    pub fn save_position(
        &mut self,
        hash: u64,
        depth: u64,
        score: i64,
        node_type: NodeType,
        best_move: &Option<Move>,
    ) {
        self.store(TranspositionEntry {
            hash,
            depth,
            score,
            node_type,
            best_move: best_move.clone(),
            current_age: 0,
        })
    }
}
