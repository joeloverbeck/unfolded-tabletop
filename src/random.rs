//! Seeded randomness extracted from Arkham: one number of state, an index and a
//! shuffle. Consumers decide what the numbers mean and how the state is saved.

/// A SplitMix64 generator whose whole state is one `u64`, so a consumer can keep
/// it in saved state and resume the same sequence from it.
///
/// The numbers a state produces are a contract with consumers' saved games.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SeededRandom {
    state: u64,
}

impl SeededRandom {
    /// Starts from `seed`, or resumes from a value that [`Self::state`] returned.
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// The whole state. Save it, and pass it to [`Self::new`] to continue.
    pub const fn state(&self) -> u64 {
        self.state
    }

    /// An index below `upper`, each one equally likely.
    ///
    /// # Panics
    ///
    /// When `upper` is zero: an empty range has no index to choose.
    pub fn index(&mut self, upper: usize) -> usize {
        assert!(upper > 0, "no index can be chosen from an empty range");
        let upper = upper as u64;
        // Values below the threshold are drawn again, so that no index is
        // favoured when `upper` does not divide the generator's range.
        let threshold = upper.wrapping_neg() % upper;
        loop {
            let value = self.step();
            if value >= threshold {
                return (value % upper) as usize;
            }
        }
    }

    /// Shuffles `items` in place, every order equally likely.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for last in (1..items.len()).rev() {
            let other = self.index(last + 1);
            items.swap(last, other);
        }
    }

    fn step(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
}
