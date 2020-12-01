//! Associated RAID-PIR types

use std::ops::BitXorAssign;

use generic_array::{ArrayLength, GenericArray};

/**
 * Type for arbitrarily-sized byte arrays used as RAID-PIR database elements.
 */
#[derive(Clone, Default)]
pub struct RaidPirData<N: ArrayLength<u8>> {
    data: GenericArray<u8, N>,
}

impl<N: ArrayLength<u8>> RaidPirData<N> {
    /**
     * Construct a new object, cloning the given slic.
     */
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            data: GenericArray::clone_from_slice(slice),
        }
    }

    /**
     * Returns reference to data as slice.
     */
    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }
}

impl<N: ArrayLength<u8>> std::fmt::Debug for RaidPirData<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

impl<N: ArrayLength<u8>> BitXorAssign for RaidPirData<N> {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (a, b) in self.data.iter_mut().zip(rhs.into_iter()) {
            *a ^= b;
        }
    }
}

impl<N: ArrayLength<u8>> IntoIterator for RaidPirData<N> {
    type Item = u8;
    type IntoIter = generic_array::GenericArrayIter<Self::Item, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}
