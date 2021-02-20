use std::time;

use rand::Rng;
use shared::Id;

/// Generate a new ID using the current Unix timestamp (in milliseconds) combined with a 10-byte random number.
pub fn generate_with_timestamp() -> Id {
    // Get Unix timestamp in milliseconds:
    let timestamp = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_millis();
    // Generate a 10 byte random number:
    let random: u128 = rand::thread_rng().gen_range(0..10000000000);
    // Most significant 6 bytes are the timestamp, least significant 10 bytes are the random number:
    Id::new((timestamp << 80) + random)
}

/// Generate a new ID using a 16-byte random number.
pub fn generate_random() -> Id {
    let random: u128 = rand::thread_rng().gen();
    Id::new(random)
}
