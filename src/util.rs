//! Utility functions

use bitvec::prelude::*;
use rand_chacha::ChaChaRng;
use rand::{RngCore, SeedableRng};
use rayon::prelude::*;

/**
 * Generate a BitVec of random data with the given size and seed.
 *
 * Underlying BitVec element will be usize regardless of platform.
 *
 * ```
 * use bitvec::prelude::*;
 *
 * let seed: u64 = 1234;
 * let rbv = raidpir::util::rand_bitvec(seed, 5);
 *
 * assert_eq!(rbv.elements(), 1);
 * assert_eq!(rbv, bitvec![Lsb0, usize; 0, 1, 0, 0, 1]);
 * ```
 */
pub fn rand_bitvec(seed: u64, len: usize) -> BitVec {
    // Using ChaCha20, to be reproducible on different architectures.
    let mut prng = ChaChaRng::seed_from_u64(seed);

    // BitVec, by default, works with the platform's usize type. Size depends on
    // the platform, and since RngCore doesn't provide a method for returning
    // usizes, this is implemented differently for 32 and 64 bit targets.

    let buffersize = if cfg!(target_pointer_width = "32") {
        assert!(std::mem::size_of::<&usize>() == std::mem::size_of::<&u32>());
        (len / 32) + 1
    } else if cfg!(target_pointer_width = "64") {
        assert!(std::mem::size_of::<&usize>() == std::mem::size_of::<&u64>());
        (len / 64) + 1
    } else {
        unreachable!();
    };

    let mut buffer: Vec<usize> = Vec::with_capacity(buffersize);

    for _i in 0..buffersize {
        if cfg!(target_pointer_width = "32") {
            buffer.push(prng.next_u32() as usize);
        } else if cfg!(target_pointer_width = "64") {
            buffer.push(prng.next_u64() as usize);
        } else {
            unreachable!();
        }
    }

    let mut bv = BitVec::from_vec(buffer);
    bv.resize(len, false);

    bv
}

/**
 * Xor b_slice into a_slice, using rayon for parallelization.
 *
 * Only a separate function for testing purposes.
 */
pub fn xor_into_slice(a_slice: &mut [usize], b_slice: &[usize]) {
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
