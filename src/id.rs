use std::{convert::TryInto, fmt, str};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Id {
    value: u128
}

impl Id {
    pub fn new(value: u128) -> Self { Id { value } }

    pub fn decode(s: &str) -> Option<Self> {
        let bytes = base64::decode_config(s, base64::STANDARD_NO_PAD).ok()?;
        let id = Id::new(u128::from_be_bytes(bytes.try_into().ok()?));
        Some(id)
    }

    pub fn encode(&self) -> String {
        let bytes = u128::to_be_bytes(self.value);
        base64::encode_config(bytes, base64::STANDARD_NO_PAD)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{} (0x{:032X})", self.encode(), self.value) }
}

#[cfg(test)]
mod tests {
    use super::Id;

    #[test]
    fn conversions() {
        let x = Id::new(0xDEADBEEF);
        assert_eq!(Id::decode(&x.encode()), Some(x));
    }
}
