//! Cryptographic Randomization

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use rand_chacha::ChaCha20Rng;
use rand_core::{CryptoRng, RngCore, SeedableRng};

static STREAM: AtomicU64 = AtomicU64::new(0);

/// Holds a 256-bit seed used for randomization.
/// The default seed uses `rand::FromEntropy::from_entropy` to generate a random value, this
/// uses the underlying OS getrandom() syscall (on linux /dev/random)
/// which is both convient and safe.
/// ## Example
///     use asjeeves_encryption::prelude::*;
///     // Generate a seed from a u64 (useful for testing)
///     let seed = Seed::from(1);
///     // Generate a random seed (recomeneded for production).
///     let seed = Seed::default();
///     // Get an rng
///     let mut rng : Rng = seed.rng();
///     let kek = KeyEncryptionKey::generate(&mut rng);
#[derive(Clone)]
pub struct Seed(Arc<[u8; 32]>);

/// ChaCha20 randomizer.
pub struct Rng(ChaCha20Rng);

impl AsRef<ChaCha20Rng> for Rng {
    fn as_ref(&self) -> &ChaCha20Rng {
        &self.0
    }
}

impl AsMut<ChaCha20Rng> for Rng {
    fn as_mut(&mut self) -> &mut ChaCha20Rng {
        &mut self.0
    }
}

impl CryptoRng for Rng {}

impl fmt::Debug for Rng {
    // We don't actually want debug info for Rng as its cryptographically sensitive.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rng").finish_non_exhaustive()
    }
}

impl RngCore for Rng {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.0.try_fill_bytes(dest)
    }
}

impl Seed {
    /// Returns a randomizer [Rng]. This uses the ChaCha20 Algorithm.
    /// See: [ChaCha Wikipedia Page](https://en.wikipedia.org/wiki/Salsa20#ChaCha_variant)
    pub fn rng(&self) -> Rng {
        let local_stream = STREAM.fetch_add(1, Ordering::Relaxed);

        let mut rng = ChaCha20Rng::from_seed(*self.0);

        rng.set_stream(local_stream);

        Rng(rng)
    }

    fn reset_stream() {
        STREAM.store(0, Ordering::SeqCst)
    }
}

impl fmt::Debug for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Seed").finish_non_exhaustive()
    }
}

impl Default for Seed {
    fn default() -> Self {
        let mut rng = ChaCha20Rng::from_entropy();
        let mut seed = [0u8; 32];

        rng.fill_bytes(&mut seed);

        let seed = Arc::new(seed);

        Self(seed)
    }
}

impl From<u64> for Seed {
    fn from(value: u64) -> Self {
        let mut rng = ChaCha20Rng::seed_from_u64(value);
        let mut seed = [0u8; 32];

        rng.fill_bytes(&mut seed);

        let seed = Arc::new(seed);

        Seed::reset_stream();

        Self(seed)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_generates_a_random_number() {
        {
            let seed = Seed::from(1);
            let mut rng: Rng = seed.rng();

            let mut data = [0u8; 2];

            rng.fill_bytes(&mut data);

            assert_eq!([62u8, 186u8], data);
        }

        {
            let seed = Seed::from(1);
            let mut rng: Rng = seed.rng();

            let mut data = [0u8; 2];

            rng.try_fill_bytes(&mut data).unwrap();

            assert_eq!([62u8, 186u8], data);
        }

        {
            let seed = Seed::from(1);
            let mut rng: Rng = seed.rng();

            let datum: u32 = rng.next_u32();

            assert_eq!(649050686, datum);
        }

        {
            let seed = Seed::from(1);
            let mut rng: Rng = seed.rng();

            let datum: u64 = rng.next_u64();

            assert_eq!(15639741899973048894, datum);
        }
    }

    #[test]
    fn it_obfuscates_sensitive_data_when_debugged() {
        let seed = Seed::default();

        let dbg = format!("{:?}", seed);

        assert_eq!("Seed { .. }", dbg);

        let dbg = format!("{:?}", seed.rng());

        assert_eq!("Rng { .. }", dbg);
    }
}
