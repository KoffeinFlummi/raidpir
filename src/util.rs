//! Utility functions

use std::convert::TryInto;

use bitvec::prelude::*;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;
use rayon::prelude::*;

/**
 * Generate a BitVec of random data with the given size and seed.
 *
 * Underlying BitVec element will be usize regardless of platform.
 *
 * ```
 * use bitvec::prelude::*;
 *
 * let seed: u128 = 1234;
 * let rbv = raidpir::util::rand_bitvec(seed, 5);
 *
 * assert_eq!(rbv.elements(), 1);
 * assert_eq!(rbv, bitvec![Lsb0, usize; 1, 1, 0, 1, 1]);
 * ```
 */
pub fn rand_bitvec(seed: u128, len: usize) -> BitVec<Lsb0, u8> {
    // Using ChaCha20, to be reproducible on different architectures.
    let seed_bytes: [u8; 32] = [seed.to_le_bytes(), [0; 16]].concat().try_into().unwrap();
    let mut prng = ChaChaRng::from_seed(seed_bytes);

    let mut vec = vec![0; len];
    prng.fill_bytes(&mut vec);

    let mut bv = BitVec::from_vec(vec);
    bv.resize(len, false);

    bv
}

/**
 * Xor b_slice into a_slice, using rayon for parallelization.
 *
 * Only a separate function for testing purposes.
 */
pub fn xor_into_slice(a_slice: &mut [u8], b_slice: &[u8]) {
    // When compiled with RUSTFLAGS="-C target-feature=+avx2", this will make
    // use of AVX2 on x86-64.
    //
    // For aarch64-linux-android, this should make use of NEON without any
    // further flags.
    a_slice
        .par_iter_mut()
        .zip(b_slice.par_iter())
        .with_min_len(1 << 16)
        .for_each(|(a, b)| *a ^= b);
}
