//! Associated RAID-PIR types

use std::ops::{BitXor, BitXorAssign};

/**
 * Type for arbitrarily-sized byte arrays used as RAID-PIR database elements.
 */
#[derive(Clone, Default)]
pub struct RaidPirData {
    /// Underlying data
    pub data: Vec<u8>,
}

impl RaidPirData {
    /**
     * Construct a new object, cloning the given slice.
     */
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /**
     * Returns reference to data as slice.
     */
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl std::fmt::Debug for RaidPirData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

impl BitXor for RaidPirData {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut new = self.clone();

        // This makes sure that items created with default() are not empty
        // after being XORed into.
        if new.data.len() < rhs.data.len() {
            new.data.resize(rhs.data.len(), 0);
        }

        new.data.iter_mut().zip(rhs.data.iter()).for_each(|(a, b)| {
            *a ^= b;
        });

        new
    }
}

impl BitXorAssign for RaidPirData {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.data.iter_mut().zip(rhs.data.iter()).for_each(|(a,b)| {
            *a ^= b;
        });
    }
}

impl IntoIterator for RaidPirData {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl Into<Vec<u8>> for RaidPirData {
    fn into(self) -> Vec<u8> {
        self.data
    }
}
