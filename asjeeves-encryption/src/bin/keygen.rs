//! Utility to generate and print 32 byte key in raw binary that
//! can be used as an API KEY or a Key Encryption Key.
//! Usage: cargo run --bin keygen --features="cli" > /path/to/kek.bin

use asjeeves_encryption::prelude::*;

use std::io::{self, Write};

fn main() -> io::Result<()> {
    let seed = Seed::default();

    let mut rng = seed.rng();

    let kek = KeyEncryptionKey::generate(&mut rng);

    let mut stdout = io::stdout();

    stdout.write_all(kek.as_slice())?;
    stdout.flush()?;

    Ok(())
}
