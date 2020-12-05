//! Associated RAID-PIR types

use std::ops::BitXorAssign;

/**
 * Type for arbitrarily-sized byte arrays used as RAID-PIR database elements.
 */
#[derive(Clone, Default)]
pub struct RaidPirData {
    data: Vec<u8>,
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

impl BitXorAssign for RaidPirData {
    fn bitxor_assign(&mut self, rhs: Self) {
        let rhs_iter = rhs.into_iter();

        // This makes sure that items created with default() are not empty
        // after being XORed into.
        if self.data.len() < rhs_iter.size_hint().0 {
            self.data.resize(rhs_iter.size_hint().0, 0);
        }

        for (a, b) in self.data.iter_mut().zip(rhs_iter) {
            *a ^= b;
        }
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
