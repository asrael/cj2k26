pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        let mut rng = Self(seed.max(1));
        rng.next();
        rng
    }

    pub fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;

        self.0
    }

    pub fn range(&mut self, n: u32) -> u32 {
        self.next() % n
    }

    pub fn f32(&mut self) -> f32 {
        self.next() as f32 / u32::MAX as f32
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new(u32::from_be_bytes(*b"CLNK"))
    }
}
