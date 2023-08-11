// A random bool generator.
// fastrand::bool() generates a whole u64 and then discards 63 bits.
// My BoolGen hopefully generates booleans faster.

pub struct BoolGen {
    rng: fastrand::Rng,
    bits: u64,
    mask: u64,
}

impl BoolGen {
    pub fn new() -> Self {
        Self {
            rng: fastrand::Rng::new(),
            bits: 0,
            mask: 0,
        }
    }

    pub fn gen(&mut self) -> bool {
        if self.mask == 0 {
            self.bits = self.rng.u64(..);
            self.mask = 1;
        }
        debug_assert!(self.mask.count_ones() == 1);
        let result = self.bits & self.mask != 0;
        self.mask = self.mask.wrapping_shl(1);
        result
    }
}
